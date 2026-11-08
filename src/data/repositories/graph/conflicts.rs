use super::helpers::{fetch_all, fetch_one};
use crate::data::models::{Conflict, Entity, FromRow};
use crate::infrastructure::InfrastructureError;
use neo4rs::{Graph, query};

#[derive(Clone)]
pub(super) struct ConflictRepository {
    graph: Graph,
}

impl ConflictRepository {
    pub(super) fn new(graph: Graph) -> Self {
        Self { graph }
    }

    // --- Conflicts ---

    #[tracing::instrument(name = "repository.graph.get_conflicts", level = "debug", skip_all)]
    pub(super) async fn get_conflicts(
        &self,
        world_id: &str,
    ) -> Result<Vec<Conflict>, InfrastructureError> {
        Ok(fetch_all::<Conflict>(
            &self.graph,
            query(
                "
                    MATCH (c:Conflict {world_id: $world_id})
                    WHERE c.resolved = false
                    RETURN c
                    ",
            )
            .param("world_id", world_id),
            "c",
        )
        .await?)
    }

    #[tracing::instrument(name = "repository.graph.list_conflicts", level = "debug", skip_all)]
    pub(super) async fn list_conflicts(
        &self,
        world_id: &str,
        limit: u32,
        cursor: Option<&str>,
        resolved: Option<bool>,
    ) -> Result<(Vec<Conflict>, Option<String>), InfrastructureError> {
        if !(1..=100).contains(&limit) {
            return Err(InfrastructureError::InvalidInput(
                "Conflict page limit must be between 1 and 100".into(),
            ));
        }
        if let Some(cursor) = cursor {
            uuid::Uuid::parse_str(cursor)
                .map_err(|_| InfrastructureError::InvalidInput("Invalid conflict cursor".into()))?;
        }
        let mut conflicts = fetch_all::<Conflict>(
            &self.graph,
            query(
                "MATCH (c:Conflict {world_id: $world_id})
                   WHERE c.id > $after_id AND ($resolved IS NULL OR c.resolved = $resolved)
                   RETURN c ORDER BY c.id LIMIT $limit",
            )
            .param("world_id", world_id)
            .param("after_id", cursor.unwrap_or_default())
            .param("resolved", resolved)
            .param("limit", i64::from(limit) + 1),
            "c",
        )
        .await?;
        let has_more = conflicts.len() > limit as usize;
        conflicts.truncate(limit as usize);
        let next_cursor = if has_more {
            conflicts.last().map(|conflict| conflict.id.clone())
        } else {
            None
        };
        Ok((conflicts, next_cursor))
    }

    #[tracing::instrument(name = "repository.graph.update_conflict", level = "debug", skip_all)]
    pub(super) async fn update_conflict(
        &self,
        id: &str,
        world_id: &str,
        conflict: &Conflict,
    ) -> Result<bool, InfrastructureError> {
        let mut result = self
            .graph
            .execute(
                query(
                    "MATCH (c:Conflict {id: $id, world_id: $world_id})
             SET c.resolved = $resolved
             RETURN c.id AS id",
                )
                .param("id", id)
                .param("world_id", world_id)
                .param("resolved", conflict.resolved),
            )
            .await?;
        Ok(result.next().await?.is_some())
    }

    /// Keep the current fact and record the writer's decision in the same query.
    /// Callers must check world ownership before resolving a conflict.
    #[tracing::instrument(name = "repository.graph.resolve_conflict", level = "debug", skip_all)]
    pub(super) async fn resolve_conflict(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<Option<Conflict>, InfrastructureError> {
        fetch_one::<Conflict>(
            &self.graph,
            query(
                "MATCH (c:Conflict {id: $id, world_id: $world_id})
                 WHERE c.decision IS NULL OR c.decision = 'keep_existing'
                 SET c.resolved = true, c.decision = 'keep_existing'
                 RETURN c",
            )
            .param("id", id)
            .param("world_id", world_id),
            "c",
        )
        .await
    }

    #[tracing::instrument(name = "repository.graph.get_conflict", level = "debug", skip_all)]
    pub(super) async fn get_conflict(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<Option<Conflict>, InfrastructureError> {
        fetch_one::<Conflict>(
            &self.graph,
            query("MATCH (c:Conflict {id: $id, world_id: $world_id}) RETURN c")
                .param("id", id)
                .param("world_id", world_id),
            "c",
        )
        .await
    }

    #[tracing::instrument(
        name = "repository.graph.get_conflicts_by_entity",
        level = "debug",
        skip_all
    )]
    pub(super) async fn get_conflicts_by_entity(
        &self,
        entity_id: &str,
        world_id: &str,
    ) -> Result<Vec<Conflict>, InfrastructureError> {
        fetch_all::<Conflict>(&self.graph,
            query(
                "MATCH (e:Entity {id: $entity_id, world_id: $world_id})-[:HAS_CONFLICT]->(c:Conflict)
             RETURN c"
            )
                .param("entity_id", entity_id)
                .param("world_id", world_id),
            "c"
        ).await
    }

    // One open conflict per entity field: a retry, or a later message contradicting
    // the same field, updates that conflict instead of adding another.
    // Returns None when the entity doesn't exist in this world.
    #[tracing::instrument(name = "repository.graph.upsert_conflict", level = "debug", skip_all)]
    pub(super) async fn upsert_conflict(
        &self,
        conflict: &Conflict,
        source_message_id: &str,
    ) -> Result<Option<Conflict>, InfrastructureError> {
        fetch_one::<Conflict>(
            &self.graph,
            query(
                "
                MATCH (e:Entity {id: $entity_id, world_id: $world_id})
                MERGE (e)-[:HAS_CONFLICT]->(c:Conflict {
                    world_id: $world_id,
                    entity_id: $entity_id,
                    field: $field,
                    resolved: false
                })
                ON CREATE SET
                    c.id                  = $id,
                    c.existing_value      = $existing_value,
                    c.new_value           = $new_value,
                    c.severity            = $severity,
                    c.explanation         = $explanation,
                    c.source_conversation = $source_conversation
                ON MATCH SET
                    c.new_value           = $new_value,
                    c.severity            = $severity,
                    c.explanation         = $explanation,
                    c.source_conversation = $source_conversation
                SET c.source_message_ids = CASE
                    WHEN $source_message_id IN coalesce(c.source_message_ids, [])
                    THEN c.source_message_ids
                    ELSE coalesce(c.source_message_ids, []) + [$source_message_id]
                END
                RETURN c
                ",
            )
            .param("source_message_id", source_message_id)
            .param("id", conflict.id.clone())
            .param("world_id", conflict.world_id.clone())
            .param("entity_id", conflict.entity_id.clone())
            .param("field", conflict.field.clone())
            .param(
                "existing_value",
                serde_json::to_string(&conflict.existing_value).unwrap_or_default(),
            )
            .param(
                "new_value",
                serde_json::to_string(&conflict.new_value).unwrap_or_default(),
            )
            .param("severity", conflict.severity.clone())
            .param("explanation", conflict.explanation.clone())
            .param("source_conversation", conflict.source_conversation.clone()),
            "c",
        )
        .await
    }

    pub(super) async fn acceptance_target(
        &self,
        txn: &mut neo4rs::Txn,
        conflict_id: &str,
        world_id: &str,
    ) -> Result<Option<(Conflict, Entity)>, InfrastructureError> {
        let mut rows = txn.execute(
                query("MATCH (c:Conflict {id: $id, world_id: $world_id}), (e:Entity {world_id: $world_id})
                       WHERE e.id = c.entity_id RETURN c, e")
                    .param("id", conflict_id)
                    .param("world_id", world_id),
            ).await?;
        let Some(row) = rows.next(&mut *txn).await? else {
            return Ok(None);
        };
        Ok(Some((
            Conflict::from_row(&row, "c")?,
            Entity::from_row(&row, "e")?,
        )))
    }

    pub(super) async fn mark_accepted(
        &self,
        txn: &mut neo4rs::Txn,
        conflict_id: &str,
        world_id: &str,
    ) -> Result<Conflict, InfrastructureError> {
        let mut saved = txn
            .execute(
                query(
                    "MATCH (c:Conflict {id: $id, world_id: $world_id})
                       SET c.resolved = true, c.decision = 'accept_new' RETURN c",
                )
                .param("id", conflict_id)
                .param("world_id", world_id),
            )
            .await?;
        let row = saved.next(&mut *txn).await?.ok_or_else(|| {
            InfrastructureError::InternalError("Conflict disappeared during resolution".into())
        })?;
        Ok(Conflict::from_row(&row, "c")?)
    }
}
