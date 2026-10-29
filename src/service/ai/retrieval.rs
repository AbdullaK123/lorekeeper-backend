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

/// Includes exact saved identity for both matches and their neighbors.
fn describe(entity: &Entity) -> String {
    let mut line = format!(
        "{} ({}) [ID: {}]",
        entity.name,
        entity.kind.join(", "),
        entity.id
    );
    if !entity.aliases.is_empty() {
        line.push_str(&format!(" [Aliases: {}]", entity.aliases.join(", ")));
    }
    if !entity.description.is_empty() {
        line.push_str(": ");
        line.push_str(&entity.description);
    }
    line
}

fn format_properties(properties: &HashMap<String, serde_json::Value>) -> String {
    let mut descriptions = Vec::new();
    for (key, value) in properties {
        let description = match value {
            serde_json::Value::String(text) => format!("{key}: {text}"),
            other => format!("{key}: {other}"),
        };
        descriptions.push(description);
    }
    descriptions.sort();
    descriptions.join("; ")
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
            lines.push(format!(
                "  Properties: {}",
                format_properties(&hit.entity.properties)
            ));
        }

        for (neighbor, rel) in &hit.neighbors {
            let line = if rel.from_id == hit.entity.id {
                format!("  - {} → {}", rel.rel_type, describe(neighbor))
            } else {
                format!("  - ← {} from {}", rel.rel_type, describe(neighbor))
            };
            lines.push(line);
            if !neighbor.properties.is_empty() {
                lines.push(format!(
                    "    Neighbor properties: {}",
                    format_properties(&neighbor.properties)
                ));
            }
            if !rel.description.is_empty() {
                lines.push(format!("    Relationship: {}", rel.description));
            }
            if !rel.properties.is_empty() {
                lines.push(format!(
                    "    Relationship properties: {}",
                    format_properties(&rel.properties)
                ));
            }
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
