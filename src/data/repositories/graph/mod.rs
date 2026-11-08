mod conflicts;
mod conversations;
mod entities;
mod helpers;
mod maintenance;
mod relationships;

use crate::data::models::{Conflict, ConflictDecision, Conversation, Entity, Relationship};
use crate::infrastructure::InfrastructureError;
use conflicts::ConflictRepository;
use conversations::GraphConversationRepository;
use entities::EntityRepository;
use helpers::validate_cypher_identifier;
use maintenance::GraphMaintenanceRepository;
use neo4rs::Graph;
use relationships::RelationshipRepository;

#[derive(Clone)]
pub struct GraphRepository {
    graph: Graph,
    entities: EntityRepository,
    relationships: RelationshipRepository,
    conflicts: ConflictRepository,
    conversations: GraphConversationRepository,
    maintenance: GraphMaintenanceRepository,
}

impl GraphRepository {
    pub fn new(graph: Graph) -> Self {
        Self {
            entities: EntityRepository::new(graph.clone()),
            relationships: RelationshipRepository::new(graph.clone()),
            conflicts: ConflictRepository::new(graph.clone()),
            conversations: GraphConversationRepository::new(graph.clone()),
            maintenance: GraphMaintenanceRepository::new(graph.clone()),
            graph,
        }
    }

    /// Idempotent: Memgraph skips a constraint that already exists. Run once at startup.
    pub async fn ensure_constraints(&self) -> Result<(), InfrastructureError> {
        self.maintenance.ensure_constraints().await
    }

    pub async fn update_entity(
        &self,
        id: &str,
        entity: &Entity,
    ) -> Result<bool, InfrastructureError> {
        self.entities.update_entity(id, entity).await
    }

    pub async fn delete_entity(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<bool, InfrastructureError> {
        self.entities.delete_entity(id, world_id).await
    }

    pub async fn get_entity(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<Option<Entity>, InfrastructureError> {
        self.entities.get_entity(id, world_id).await
    }

    /// Page entities by ID, then load directed relationships within that page.
    /// The cursor is the last returned entity ID; callers must check world ownership.
    #[tracing::instrument(name = "repository.graph.get_graph", level = "debug", skip_all)]
    pub async fn get_graph(
        &self,
        world_id: &str,
        limit: u32,
        cursor: Option<&str>,
    ) -> Result<(Vec<Entity>, Vec<Relationship>, Option<String>), InfrastructureError> {
        let (entities, next_cursor) = self.entities.graph_page(world_id, limit, cursor).await?;
        if entities.is_empty() {
            return Ok((entities, Vec::new(), next_cursor));
        }
        let ids = entities.iter().map(|entity| entity.id.as_str()).collect();
        let relationships = self.relationships.within_entities(world_id, ids).await?;
        Ok((entities, relationships, next_cursor))
    }

    /// Page world entities by ID, with an optional exact kind filter.
    /// Callers must check world ownership before calling this method.
    pub async fn list_entities(
        &self,
        world_id: &str,
        limit: u32,
        cursor: Option<&str>,
        kind: Option<&str>,
    ) -> Result<(Vec<Entity>, Option<String>), InfrastructureError> {
        self.entities
            .list_entities(world_id, limit, cursor, kind)
            .await
    }

    pub async fn get_entities_by_world(
        &self,
        world_id: &str,
    ) -> Result<Vec<Entity>, InfrastructureError> {
        self.entities.get_entities_by_world(world_id).await
    }

    /// Entities whose name or any alias matches one of `names` (compared via `name_key`).
    /// This is how extraction finds `existing_entities` for a message.
    pub async fn find_entities_by_names(
        &self,
        world_id: &str,
        names: &[String],
    ) -> Result<Vec<Entity>, InfrastructureError> {
        self.entities.find_entities_by_names(world_id, names).await
    }

    pub async fn get_entity_neighbors(
        &self,
        id: &str,
        world_id: &str,
        rel_type: Option<&str>,
        kind: Option<&str>,
    ) -> Result<Vec<(Entity, Relationship)>, InfrastructureError> {
        self.entities
            .get_entity_neighbors(id, world_id, rel_type, kind)
            .await
    }

    /// Page connecting relationships by ID and return their neighboring entities.
    /// The cursor is the last returned relationship ID; callers must check world ownership.
    pub async fn get_entity_neighbors_page(
        &self,
        id: &str,
        world_id: &str,
        rel_type: Option<&str>,
        kind: Option<&str>,
        limit: u32,
        cursor: Option<&str>,
    ) -> Result<(Vec<(Entity, Relationship)>, Option<String>), InfrastructureError> {
        self.entities
            .get_entity_neighbors_page(id, world_id, rel_type, kind, limit, cursor)
            .await
    }

    pub async fn get_relationship(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<Option<Relationship>, InfrastructureError> {
        self.relationships.get_relationship(id, world_id).await
    }

    pub async fn update_relationship(
        &self,
        id: &str,
        world_id: &str,
        relationship: &Relationship,
    ) -> Result<bool, InfrastructureError> {
        self.relationships
            .update_relationship(id, world_id, relationship)
            .await
    }

    pub async fn delete_relationship(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<bool, InfrastructureError> {
        self.relationships.delete_relationship(id, world_id).await
    }

    pub async fn create_conversation(
        &self,
        conversation: &Conversation,
    ) -> Result<(), InfrastructureError> {
        self.conversations.create_conversation(conversation).await
    }

    pub async fn update_conversation(
        &self,
        id: &str,
        world_id: &str,
        conversation: &Conversation,
    ) -> Result<bool, InfrastructureError> {
        self.conversations
            .update_conversation(id, world_id, conversation)
            .await
    }

    pub async fn delete_conversation(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<bool, InfrastructureError> {
        self.conversations.delete_conversation(id, world_id).await
    }

    pub async fn get_conversation(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<Option<Conversation>, InfrastructureError> {
        self.conversations.get_conversation(id, world_id).await
    }

    pub async fn get_conflicts(
        &self,
        world_id: &str,
    ) -> Result<Vec<Conflict>, InfrastructureError> {
        self.conflicts.get_conflicts(world_id).await
    }

    pub async fn list_conflicts(
        &self,
        world_id: &str,
        limit: u32,
        cursor: Option<&str>,
        resolved: Option<bool>,
    ) -> Result<(Vec<Conflict>, Option<String>), InfrastructureError> {
        self.conflicts
            .list_conflicts(world_id, limit, cursor, resolved)
            .await
    }

    pub async fn update_conflict(
        &self,
        id: &str,
        world_id: &str,
        conflict: &Conflict,
    ) -> Result<bool, InfrastructureError> {
        self.conflicts.update_conflict(id, world_id, conflict).await
    }

    /// Keep the current fact and record the writer's decision in the same query.
    /// Callers must check world ownership before resolving a conflict.
    pub async fn resolve_conflict(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<Option<Conflict>, InfrastructureError> {
        self.conflicts.resolve_conflict(id, world_id).await
    }

    /// Accept a saved claim targeting an exact, existing Entity.properties key.
    /// Entity changes and conflict resolution commit together. Callers check ownership.
    #[tracing::instrument(name = "repository.graph.accept_conflict", level = "debug", skip_all)]
    pub async fn accept_conflict(
        &self,
        conflict_id: &str,
        world_id: &str,
        new_description: &str,
        new_embedding: &[f64],
    ) -> Result<Option<Conflict>, InfrastructureError> {
        let mut txn = self.graph.start_txn().await?;
        let result: Result<Option<Conflict>, InfrastructureError> = async {
            let Some((conflict, mut entity)) = self
                .conflicts
                .acceptance_target(&mut txn, conflict_id, world_id)
                .await?
            else {
                return Ok(None);
            };
            if conflict.resolved {
                return if matches!(conflict.decision, Some(ConflictDecision::AcceptNew)) {
                    Ok(Some(conflict))
                } else {
                    Err(InfrastructureError::Conflict(
                        "Conflict already resolved with another decision".into(),
                    ))
                };
            }
            let property = entity.properties.get_mut(&conflict.field).ok_or_else(|| {
                InfrastructureError::InvalidInput(
                    "Accept-new requires an exact, existing entity property key".into(),
                )
            })?;
            *property = conflict.new_value.clone();
            for source in &conflict.source_message_ids {
                if !entity.source_message_ids.contains(source) {
                    entity.source_message_ids.push(source.clone());
                }
            }
            self.entities
                .save_accepted_conflict(&mut txn, world_id, entity, new_description, new_embedding)
                .await?;
            let conflict = self
                .conflicts
                .mark_accepted(&mut txn, conflict_id, world_id)
                .await?;
            Ok(Some(conflict))
        }
        .await;
        match result {
            Ok(conflict) => {
                txn.commit().await?;
                tracing::info!(
                    world_id,
                    conflict_id,
                    found = conflict.is_some(),
                    "accept-new transaction committed"
                );
                Ok(conflict)
            }
            Err(error) => {
                tracing::warn!(
                    world_id,
                    conflict_id,
                    "accept-new transaction failed; rolling back"
                );
                txn.rollback().await?;
                Err(error)
            }
        }
    }

    pub async fn get_conflict(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<Option<Conflict>, InfrastructureError> {
        self.conflicts.get_conflict(id, world_id).await
    }

    pub async fn get_conflicts_by_entity(
        &self,
        entity_id: &str,
        world_id: &str,
    ) -> Result<Vec<Conflict>, InfrastructureError> {
        self.conflicts
            .get_conflicts_by_entity(entity_id, world_id)
            .await
    }

    pub async fn get_relationships_by_entity(
        &self,
        entity_id: &str,
        world_id: &str,
    ) -> Result<Vec<Relationship>, InfrastructureError> {
        self.relationships
            .get_relationships_by_entity(entity_id, world_id)
            .await
    }

    pub async fn get_entities_by_conversation(
        &self,
        conversation_id: &str,
        world_id: &str,
    ) -> Result<Vec<Entity>, InfrastructureError> {
        self.entities
            .get_entities_by_conversation(conversation_id, world_id)
            .await
    }

    /// Exact-ID writes. Any failed validation or write rolls back the entire batch.
    #[tracing::instrument(
        name = "repository.graph.update_graph_batch",
        level = "debug",
        skip_all
    )]
    pub async fn update_graph_batch(
        &self,
        world_id: &str,
        entities: &[(Entity, Vec<f64>)],
        relationships: &[Relationship],
        retracted_ids: &[String],
        source_message_id: &str,
    ) -> Result<(), InfrastructureError> {
        for (e, _) in entities {
            if e.world_id != world_id {
                return Err(InfrastructureError::InternalError(
                    "Entity is outside this world".into(),
                ));
            }
        }
        for r in relationships {
            validate_cypher_identifier(&r.rel_type)?;
            if r.world_id != world_id || retracted_ids.contains(&r.id) {
                return Err(InfrastructureError::InternalError(
                    "Invalid relationship batch".into(),
                ));
            }
        }
        let mut txn = self.graph.start_txn().await?;
        let result: Result<(), InfrastructureError> = async {
            for (e, embedding) in entities {
                self.entities
                    .upsert_in_transaction(&mut txn, world_id, e, embedding, source_message_id)
                    .await?;
            }
            for r in relationships {
                self.relationships
                    .upsert_in_transaction(&mut txn, world_id, r, source_message_id)
                    .await?;
            }
            for id in retracted_ids {
                self.relationships
                    .retract_in_transaction(&mut txn, world_id, id)
                    .await?;
            }
            Ok(())
        }
        .await;
        match result {
            Ok(()) => {
                txn.commit().await?;
                tracing::info!(
                    world_id,
                    entities = entities.len(),
                    relationships = relationships.len(),
                    retractions = retracted_ids.len(),
                    "graph batch committed"
                );
                Ok(())
            }
            Err(error) => {
                tracing::warn!(world_id, "graph batch failed; rolling back");
                txn.rollback().await?;
                Err(error)
            }
        }
    }

    pub async fn upsert_conflict(
        &self,
        conflict: &Conflict,
        source_message_id: &str,
    ) -> Result<Option<Conflict>, InfrastructureError> {
        self.conflicts
            .upsert_conflict(conflict, source_message_id)
            .await
    }

    pub async fn vector_search_entities(
        &self,
        world_id: &str,
        query_embedding: Vec<f64>,
        limit: u32,
    ) -> Result<Vec<Entity>, InfrastructureError> {
        self.entities
            .vector_search_entities(world_id, query_embedding, limit)
            .await
    }

    pub async fn fts_search_entities(
        &self,
        world_id: &str,
        query_text: &str,
        limit: u32,
    ) -> Result<Vec<Entity>, InfrastructureError> {
        self.entities
            .fts_search_entities(world_id, query_text, limit)
            .await
    }

    pub async fn delete_relationships_by_endpoints(
        &self,
        world_id: &str,
        from_id: &str,
        to_id: &str,
        rel_type: &str,
    ) -> Result<(), InfrastructureError> {
        self.relationships
            .delete_relationships_by_endpoints(world_id, from_id, to_id, rel_type)
            .await
    }

    pub async fn delete_graph(&self, world_id: &str) -> Result<(), InfrastructureError> {
        self.maintenance.delete_graph(world_id).await
    }
}
