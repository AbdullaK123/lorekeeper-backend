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
