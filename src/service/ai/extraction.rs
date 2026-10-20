use crate::data::{
    Conflict, ConflictExtraction, Entity, ExtractedConflict, ExtractedEntity,
    ExtractedRelationship, GraphExtraction, GraphRepository, Relationship, name_key,
};
use crate::service::{ServiceError, relationships};
use rig::DynModel;
use rig::extractor::Extractor;
use serde_json::json;
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
    pub conflicts_flagged: usize,
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
    pub async fn process_message(
        &self,
        world_id: Uuid,
        conversation_id: Uuid,
        message: &str,
    ) -> Result<ExtractionReport, ServiceError> {
        let candidates = self.find_candidates(world_id).await?;
        let extraction = self.extract(message, &candidates).await?;
        let mut report = ExtractionReport::default();

        let mut name_to_id: HashMap<String, String> = candidates
            .iter()
            .map(|e| (e.name_key.clone(), e.id.clone()))
            .collect();

        for extracted in extraction.entities {
            let applied = self
                .apply_entity(world_id, conversation_id, message, extracted)
                .await?;

            name_to_id.insert(applied.entity.name_key.clone(), applied.entity.id.clone());

            if applied.created {
                report.entities_created += 1
            } else {
                report.entities_updated += 1
            }
            report.conflicts_flagged += applied.conflicts_flagged;
        }

        let (relationships_written, relationships_skipped) = self
            .apply_relationships(
                world_id,
                conversation_id,
                extraction.relationships,
                &name_to_id,
            )
            .await?;

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

    /// Runs the entity extractor on the message, with the candidates as `existing_entities`.
    async fn extract(
        &self,
        message: &str,
        candidates: &[Entity],
    ) -> Result<GraphExtraction, ServiceError> {
        let candidates_json = serde_json::to_string_pretty(
            &candidates
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "name": e.name,
                        "aliases": e.aliases,
                        "kind": e.kind,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap_or_else(|_| "[]".into());

        let user_prompt = format!(
            "<existing_entities>\n{}\n</existing_entities>\n\n<user_message>\n{}\n</user_message>",
            candidates_json, message,
        );
        let result = self.entity_extractor.extract(user_prompt).await?;
        Ok(result.output)
    }

    /// New entity: upsert it. Existing entity (`existing_id` set): load it, flag
    /// conflicts, merge only the non-conflicting fields, then upsert.
    async fn apply_entity(
        &self,
        world_id: Uuid,
        conversation_id: Uuid,
        message: &str,
        extracted: ExtractedEntity,
    ) -> Result<AppliedEntity, ServiceError> {
        if extracted.existing_id.is_none() {
            let entity = Entity::from_extraction(extracted, world_id);
            let embedding = self.embed(&entity).await?;
            let saved = self.graph_repo.upsert_entity(&entity, embedding).await?;

            return Ok(AppliedEntity {
                entity: saved,
                created: true,
                conflicts_flagged: 0,
            });
        }

        let existing_id = extracted.existing_id.clone().unwrap();

        let existing = self
            .graph_repo
            .get_entity(&existing_id, &world_id.to_string())
            .await?;

        if let Some(existing) = existing {
            let conflicts = self
                .detect_conflicts(&existing, &extracted, message)
                .await?;

            for c in &conflicts {
                let conflict = Conflict::from_extraction(c.clone(), world_id, conversation_id);

                self.graph_repo.upsert_conflict(&conflict).await?;
            }

            let conflicting_fields: Vec<String> =
                conflicts.iter().map(|c| c.field.clone()).collect();
            let merged = Self::merge(&existing, &extracted, &conflicting_fields);
            let embedding = self.embed(&merged).await?;
            let saved = self.graph_repo.upsert_entity(&merged, embedding).await?;

            Ok(AppliedEntity {
                entity: saved,
                created: false,
                conflicts_flagged: conflicts.len(),
            })
        } else {
            let entity = Entity::from_extraction(extracted, world_id);
            let embedding = self.embed(&entity).await?;
            let saved = self.graph_repo.upsert_entity(&entity, embedding).await?;

            return Ok(AppliedEntity {
                entity: saved,
                created: true,
                conflicts_flagged: 0,
            });
        }
    }

    /// Runs the conflict extractor comparing the stored entity with the new extraction.
    async fn detect_conflicts(
        &self,
        existing: &Entity,
        extracted: &ExtractedEntity,
        message: &str,
    ) -> Result<Vec<ExtractedConflict>, ServiceError> {
        let existing_entity_json =
            serde_json::to_string_pretty(existing).unwrap_or_else(|_| "{}".into());

        let extracted_json =
            serde_json::to_string_pretty(extracted).unwrap_or_else(|_| "{}".into());

        let user_prompt = format!(
            "<existing_entity>\n{}\n</existing_entity>\n\n\
             <extracted_entity>\n{}\n</extracted_entity>\n\n\
             <source_message>\n{}\n</source_message>",
            existing_entity_json, extracted_json, message,
        );
        let result = self.conflict_extractor.extract(user_prompt).await?;
        let conflicts = result.output.conflicts;
        Ok(conflicts)
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
