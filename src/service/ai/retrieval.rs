use crate::{
    data::{Entity, GraphRepository, Relationship},
    service::ServiceError,
};
use rig::{DynModel, operation::Embedding};
use std::collections::{HashMap, HashSet};

pub struct SearchHit {
    pub entity: Entity,
    pub neighbors: Vec<(Entity, Relationship)>,
}

/// "Kael (character): A warrior from the Ashlands."
fn describe(entity: &Entity) -> String {
    let mut line = format!("{} ({})", entity.name, entity.kind.join(", "));
    if !entity.description.is_empty() {
        line.push_str(": ");
        line.push_str(&entity.description);
    }
    line
}

/// Renders search hits as the plain-text lore block the chat agent reads.
/// Arrows always read in the relationship's real direction, whichever side the match is on.
pub fn format_hits(hits: &[SearchHit]) -> String {
    if hits.is_empty() {
        return "No matching lore found in this world.".to_string();
    }

    let mut blocks = Vec::new();
    for hit in hits {
        let mut lines = vec![describe(&hit.entity)];

        if !hit.entity.properties.is_empty() {
            let mut props: Vec<String> = hit
                .entity
                .properties
                .iter()
                .map(|(key, value)| match value {
                    serde_json::Value::String(s) => format!("{key}: {s}"),
                    other => format!("{key}: {other}"),
                })
                .collect();
            props.sort();
            lines.push(format!("  Properties: {}", props.join("; ")));
        }

        for (neighbor, rel) in &hit.neighbors {
            let line = if rel.from_id == hit.entity.id {
                format!("  - {} → {}", rel.rel_type, describe(neighbor))
            } else {
                format!("  - ← {} from {}", rel.rel_type, describe(neighbor))
            };
            lines.push(line);
        }

        blocks.push(lines.join("\n"));
    }
    blocks.join("\n\n")
}

#[derive(Clone)]
pub struct RetrievalService {
    graph: GraphRepository,
    embedder: DynModel<Embedding>,
}

impl RetrievalService {
    pub fn new(graph: GraphRepository, embedder: DynModel<Embedding>) -> Self {
        Self { graph, embedder }
    }

    fn rrf(lists: &[Vec<Entity>], k: f64) -> Vec<Entity> {
        let mut scoreboard: HashMap<String, f64> = HashMap::new();
        for list in lists {
            for (i, entity) in list.iter().enumerate() {
                *scoreboard.entry(entity.id.clone()).or_insert(0.0) += 1.0 / (k + i as f64 + 1.0);
            }
        }
        let mut entities: Vec<Entity> = lists.concat();
        let mut seen = HashSet::new();
        entities.retain(|e| seen.insert(e.id.clone()));
        entities.sort_by(|a, b| scoreboard[&b.id].total_cmp(&scoreboard[&a.id]));
        entities
    }

    pub async fn search(
        &self,
        query: &str,
        world_id: &str,
        k: f64,
    ) -> Result<Vec<SearchHit>, ServiceError> {
        let query_embedding = self.embedder.embed_text(query).await?.vec;
        let fts_search_results = self.graph.fts_search_entities(world_id, query).await?;
        let vector_search_results = self
            .graph
            .vector_search_entities(world_id, query_embedding)
            .await?;
        let rrf_results = Self::rrf(&[fts_search_results, vector_search_results], k);

        let mut hits = Vec::new();
        for entity in rrf_results.into_iter().take(5) {
            let neighbors = self
                .graph
                .get_entity_neighbors(&entity.id, world_id, None, None)
                .await?;
            hits.push(SearchHit { entity, neighbors });
        }
        Ok(hits)
    }
}

#[cfg(test)]
mod tests;
