use crate::data::{
    Conflict, ConflictExtraction, Entity, ExtractedConflict, ExtractedEntity,
    ExtractedRelationship, GraphExtraction, GraphRepository, Message, Relationship, name_key,
};
use crate::service::ServiceError;
use itertools::Itertools;
use rig::DynModel;
use rig::extractor::Extractor;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

/// What one `process_message` run changed in the graph.
#[derive(Debug, Default)]
pub struct ExtractionReport {
    pub entities_created: usize,
    pub entities_updated: usize,
    pub conflicts_flagged: usize,
    pub relationships_written: usize,
    /// Relationships dropped because an endpoint name didn't resolve to an entity.
    pub relationships_skipped: usize,
}

/// The result of applying one extracted entity.
pub struct AppliedEntity {
    pub entity: Entity,
    pub created: bool,
}

/// Turns a writer's message into graph writes. The only writer of lore to the graph.
/// Every step is idempotent, so a retried job can safely run from the start.
#[derive(Clone)]
pub struct ExtractionService {
    graph_repo: GraphRepository,
    entity_extractor: Arc<Extractor<GraphExtraction>>,
    conflict_extractor: Arc<Extractor<ConflictExtraction>>,
    embedding_model: DynModel<rig::operation::Embedding>,
}

impl ExtractionService {
    pub fn new(
        graph_repo: GraphRepository,
        entity_extractor: Extractor<GraphExtraction>,
        conflict_extractor: Extractor<ConflictExtraction>,
        embedding_model: DynModel<rig::operation::Embedding>,
    ) -> Self {
        Self {
            graph_repo,
            entity_extractor: Arc::new(entity_extractor),
            conflict_extractor: Arc::new(conflict_extractor),
            embedding_model,
        }
    }

    async fn embed(&self, entity: &Entity) -> Result<Vec<f64>, ServiceError> {
        let text = format!(
            "{}. {}. {}",
            entity.name,
            entity.kind.join(", "),
            entity.description
        );
        let embedding = self.embedding_model.embed_text(&text).await?;
        Ok(embedding.vec)
    }

    /// Entry point for the background job: find candidates, extract, apply entities,
    /// then apply relationships using the IDs the entities resolved to.
    ///
    /// `history` is the conversation as it was *before* this message: the caller passes
    /// it in, so the job never races the handler's saves and is self-contained for retries.
    pub async fn process_message(
        &self,
        world_id: Uuid,
        conversation_id: Uuid,
        message: &str,
        history: &[Message],
    ) -> Result<ExtractionReport, ServiceError> {
        let candidates = self.find_candidates(world_id).await?;
        let mut extraction = self.extract(message, history, &candidates).await?;
        for extracted in &mut extraction.entities {
            Self::resolve_existing(extracted, &candidates);
        }
        let mut report = ExtractionReport::default();

        // One conflict check for the whole message, before anything is saved, so it can
        // catch clashes that span entities and relationships (the Void Gate case).
        let related = self
            .find_related(world_id, &extraction, &candidates)
            .await?;
        let conflicts = if related.is_empty() {
            Vec::new()
        } else {
            self.detect_conflicts(&extraction, &related, message)
                .await?
        };
        let mut conflicting_fields: HashMap<String, Vec<String>> = HashMap::new();
        for c in &conflicts {
            let conflict = Conflict::from_extraction(c.clone(), world_id, conversation_id);
            // None = the AI named an entity that isn't in this world; skip it.
            if self.graph_repo.upsert_conflict(&conflict).await?.is_some() {
                report.conflicts_flagged += 1;
                conflicting_fields
                    .entry(c.entity_id.clone())
                    .or_default()
                    .push(c.field.clone());
            }
        }

        let mut name_to_id: HashMap<String, String> = HashMap::new();
        for e in &candidates {
            Self::index_names(&mut name_to_id, e);
        }

        for extracted in extraction.entities {
            let applied = self
                .apply_entity(world_id, extracted, &conflicting_fields)
                .await?;

            Self::index_names(&mut name_to_id, &applied.entity);

            if applied.created {
                report.entities_created += 1
            } else {
                report.entities_updated += 1
            }
        }

        let (relationships_written, relationships_skipped) = self
            .apply_relationships(
                world_id,
                conversation_id,
                extraction.relationships,
                &name_to_id,
            )
            .await?;

        for rel in extraction.retracted_relationships {
            if let (Some(a), Some(b)) = (
                name_to_id.get(&name_key(&rel.from_name)),
                name_to_id.get(&name_key(&rel.to_name)),
            ) {
                self.graph_repo
                    .delete_relationships_by_endpoints(&world_id.to_string(), a, b, &rel.rel_type)
                    .await?;
            }
        }
        report.relationships_written += relationships_written;
        report.relationships_skipped += relationships_skipped;

        Ok(report)
    }

    /// Option A: every entity in the world, so the extractor can resolve references
    /// like "the Conclave" itself. Swap for a name/alias lookup if worlds get large.
    async fn find_candidates(&self, world_id: Uuid) -> Result<Vec<Entity>, ServiceError> {
        let result = self
            .graph_repo
            .get_entities_by_world(&world_id.to_string())
            .await?;
        Ok(result)
    }

    /// Runs the entity extractor on the message, with the candidates as `existing_entities`
    /// and the last few earlier messages as context for resolving references like "his".
    async fn extract(
        &self,
        message: &str,
        history: &[Message],
        candidates: &[Entity],
    ) -> Result<GraphExtraction, ServiceError> {
        let candidates_json = serde_json::to_string_pretty(
            &candidates
                .iter()
                .map(|e| {
                    // Property labels only (no values): lets the extractor reuse a label
                    // like `departure_time` instead of inventing `launched`, so a changed
                    // value lands on the same key and the conflict check can protect it.
                    let mut property_labels: Vec<&String> = e.properties.keys().collect();
                    property_labels.sort();
                    serde_json::json!({
                        "id": e.id,
                        "name": e.name,
                        "aliases": e.aliases,
                        "kind": e.kind,
                        "property_labels": property_labels,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap_or_else(|_| "[]".into());

        let history: String = history
            .iter()
            .filter_map(|m| match m {
                Message::User { content, .. } => Some(format!("User:\n{content}\n")),
                Message::Assistant {
                    content: Some(content),
                    ..
                } => Some(format!("Assistant:\n{content}\n")),
                _ => None,
            })
            .tail(6)
            .join("\n");

        let user_prompt = format!(
            "<existing_entities>\n{}\n</existing_entities>\n\n<recent_messages>\n{history}\n</recent_messages>\n\n<user_message>\n{}\n</user_message>",
            candidates_json, message,
        );
        let result = self.entity_extractor.extract(user_prompt).await?;
        Ok(result.output)
    }

    /// New entity: upsert it. Existing entity (`existing_id` set): merge in the new
    /// information, leaving fields flagged by the conflict check at their stored values.
    async fn apply_entity(
        &self,
        world_id: Uuid,
        extracted: ExtractedEntity,
        conflicting_fields: &HashMap<String, Vec<String>>,
    ) -> Result<AppliedEntity, ServiceError> {
        let existing = match &extracted.existing_id {
            Some(id) => {
                self.graph_repo
                    .get_entity(id, &world_id.to_string())
                    .await?
            }
            None => None,
        };

        // New, or the AI gave an ID that isn't in the graph: save it as new.
        let Some(existing) = existing else {
            let entity = Entity::from_extraction(extracted, world_id);
            let embedding = self.embed(&entity).await?;
            let saved = self.graph_repo.upsert_entity(&entity, embedding).await?;
            return Ok(AppliedEntity {
                entity: saved,
                created: true,
            });
        };

        let flagged = conflicting_fields
            .get(&existing.id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let merged = Self::merge(&existing, &extracted, flagged);
        let embedding = self.embed(&merged).await?;
        let saved = self.graph_repo.upsert_entity(&merged, embedding).await?;
        Ok(AppliedEntity {
            entity: saved,
            created: false,
        })
    }

    /// Saved entities this message touches (matched entities + relationship endpoints
    /// that name a candidate), plus everything linked to them. The conflict check
    /// compares the whole extraction against this set.
    async fn find_related(
        &self,
        world_id: Uuid,
        extraction: &GraphExtraction,
        candidates: &[Entity],
    ) -> Result<Vec<Entity>, ServiceError> {
        let world = world_id.to_string();

        // 1. Which saved entities does this message touch?
        let mut ids: Vec<String> = extraction
            .entities
            .iter()
            .filter_map(|e| e.existing_id.clone())
            .collect();
        for rel in &extraction.relationships {
            for name in [&rel.from_name, &rel.to_name] {
                if let Some(c) = candidates.iter().find(|c| c.name_key == name_key(name)) {
                    ids.push(c.id.clone());
                }
            }
        }

        // 2. Load each one, plus everything linked to it.
        let mut related = Vec::new();
        for id in ids {
            if let Some(entity) = self.graph_repo.get_entity(&id, &world).await? {
                related.push(entity);
            }
            for (neighbor, _) in self
                .graph_repo
                .get_entity_neighbors(&id, &world, None, None)
                .await?
            {
                related.push(neighbor);
            }
        }

        // 3. Keep each entity once.
        let mut seen = HashSet::new();
        related.retain(|e| seen.insert(e.id.clone()));
        Ok(related)
    }

    /// Runs the conflict extractor comparing the stored entity with the new extraction.
    async fn detect_conflicts(
        &self,
        extraction: &GraphExtraction,
        related: &[Entity],
        message: &str,
    ) -> Result<Vec<ExtractedConflict>, ServiceError> {
        let extracted_json =
            serde_json::to_string_pretty(extraction).unwrap_or_else(|_| "{}".into());

        let related_json = serde_json::to_string_pretty(related).unwrap_or_else(|_| "[]".into());

        let user_prompt = format!(
            "<extracted>\n{}\n</extracted>\n\n\
             <related>\n{}\n</related>\n\n\
             <source_message>\n{}\n</source_message>",
            extracted_json, related_json, message,
        );
        let result = self.conflict_extractor.extract(user_prompt).await?;
        let conflicts = result.output.conflicts;
        Ok(conflicts)
    }

    /// Safety net for a match the extractor missed. If `extracted` has no `existing_id`
    /// but exactly one saved entity shares a name or alias with it (compared by
    /// `name_key`), link it to that entity, and keep the writer's form as an alias.
    /// "Teodor" then updates "Teodor Kost" (alias "Teodor") instead of becoming a
    /// second Teodor. Two or more matches are ambiguous, so it leaves those alone.
    fn resolve_existing(extracted: &mut ExtractedEntity, candidates: &[Entity]) {
        if extracted.existing_id.is_some() {
            return;
        }
        let keys: HashSet<String> = std::iter::once(&extracted.name)
            .chain(extracted.aliases.iter())
            .map(|n| name_key(n))
            .collect();
        let mut matches = candidates.iter().filter(|c| {
            std::iter::once(&c.name)
                .chain(c.aliases.iter())
                .any(|n| keys.contains(&name_key(n)))
        });
        let (Some(found), None) = (matches.next(), matches.next()) else {
            return;
        };
        extracted.existing_id = Some(found.id.clone());
        if name_key(&extracted.name) != found.name_key
            && !extracted.aliases.contains(&extracted.name)
        {
            extracted.aliases.push(extracted.name.clone());
        }
    }

    /// Adds an entity's name and every alias to `name_to_id`, so a relationship that
    /// uses any of them finds the right entity. A name already taken keeps its first owner.
    fn index_names(name_to_id: &mut HashMap<String, String>, entity: &Entity) {
        for n in std::iter::once(&entity.name).chain(entity.aliases.iter()) {
            name_to_id
                .entry(name_key(n))
                .or_insert_with(|| entity.id.clone());
        }
    }

    /// Pure: combines the stored entity with new information, leaving conflicting
    /// fields at their stored values. Must give the same result when applied twice.
    fn merge(
        existing: &Entity,
        extracted: &ExtractedEntity,
        conflicting_fields: &[String],
    ) -> Entity {
        let mut merged_entity = existing.clone();

        for (key, value) in &extracted.properties {
            if conflicting_fields.contains(&format!("properties.{key}")) {
                continue;
            }
            merged_entity.properties.insert(key.clone(), value.clone());
        }

        for alias in &extracted.aliases {
            if !merged_entity.aliases.contains(alias) {
                merged_entity.aliases.push(alias.clone());
            }
        }

        if !conflicting_fields.contains(&"kind".to_string()) {
            for k in &extracted.kind {
                if !merged_entity.kind.contains(k) {
                    merged_entity.kind.push(k.clone());
                }
            }
        }

        let new_text = extracted.description.trim();

        if !conflicting_fields.contains(&"description".to_string())
            && !new_text.is_empty()
            && !merged_entity.description.contains(new_text)
        {
            merged_entity.description =
                format!("{} {}", merged_entity.description.trim(), new_text)
                    .trim()
                    .to_string();
        }

        merged_entity
    }

    /// `name_to_id` is keyed by `name_key`, covering entities from this extraction and
    /// the candidates. Returns (written, skipped).
    async fn apply_relationships(
        &self,
        world_id: Uuid,
        conversation_id: Uuid,
        relationships: Vec<ExtractedRelationship>,
        name_to_id: &HashMap<String, String>,
    ) -> Result<(usize, usize), ServiceError> {
        let mut skipped: usize = 0;
        let mut written: usize = 0;

        for rel in relationships.into_iter() {
            let from_id = name_to_id.get(&name_key(&rel.from_name));
            let to_id = name_to_id.get(&name_key(&rel.to_name));
            if from_id.is_none() || to_id.is_none() {
                skipped += 1;
                continue;
            }
            let rel = Relationship::from_extraction(
                rel,
                from_id.unwrap().to_string(),
                to_id.unwrap().to_string(),
                world_id,
                conversation_id,
            );
            let result = self.graph_repo.upsert_relationship(&rel).await?;
            if result.is_some() {
                written += 1;
            } else {
                skipped += 1;
            }
        }

        Ok((written, skipped))
    }
}

#[cfg(test)]
mod tests;
