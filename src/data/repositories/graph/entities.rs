use super::helpers::{fetch_all, fetch_one, validate_cypher_identifier};
use crate::data::models::{Entity, FromRow, Relationship, name_key};
use crate::infrastructure::InfrastructureError;
use neo4rs::{Graph, query};

#[derive(Clone)]
pub(super) struct EntityRepository {
    graph: Graph,
}

impl EntityRepository {
    pub(super) fn new(graph: Graph) -> Self {
        Self { graph }
    }

    #[tracing::instrument(name = "repository.graph.update_entity", level = "debug", skip_all)]
    pub(super) async fn update_entity(
        &self,
        id: &str,
        entity: &Entity,
    ) -> Result<bool, InfrastructureError> {
        let mut result = self
            .graph
            .execute(
                query(
                    "MATCH (e:Entity {id: $id, world_id: $world_id})
             SET e.name = $name,
                 e.name_key = $name_key,
                 e.aliases = $aliases,
                 e.kind = $kind,
                 e.description = $description,
                 e.properties = $properties
             RETURN e.id AS id",
                )
                .param("id", id)
                .param("world_id", entity.world_id.clone())
                .param("name", entity.name.clone())
                .param("name_key", name_key(&entity.name))
                .param("aliases", entity.aliases.clone())
                .param("kind", entity.kind.clone())
                .param("description", entity.description.clone())
                .param(
                    "properties",
                    serde_json::to_string(&entity.properties).unwrap_or_default(),
                ),
            )
            .await?;
        Ok(result.next().await?.is_some())
    }

    #[tracing::instrument(name = "repository.graph.delete_entity", level = "debug", skip_all)]
    pub(super) async fn delete_entity(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<bool, InfrastructureError> {
        let mut result = self
            .graph
            .execute(
                query("MATCH (e:Entity {id: $id, world_id: $world_id}) RETURN e.id AS id")
                    .param("id", id)
                    .param("world_id", world_id),
            )
            .await?;
        let exists = result.next().await?.is_some();
        if exists {
            self.graph
                .run(
                    // Conflicts only make sense against their entity, so they go with it.
                    query(
                        "MATCH (e:Entity {id: $id, world_id: $world_id})
                 OPTIONAL MATCH (e)-[:HAS_CONFLICT]->(c:Conflict)
                 DETACH DELETE e, c",
                    )
                    .param("id", id)
                    .param("world_id", world_id),
                )
                .await?;
        }
        Ok(exists)
    }

    #[tracing::instrument(name = "repository.graph.get_entity", level = "debug", skip_all)]
    pub(super) async fn get_entity(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<Option<Entity>, InfrastructureError> {
        fetch_one::<Entity>(
            &self.graph,
            query("MATCH (e:Entity {id: $id, world_id: $world_id}) RETURN e")
                .param("id", id)
                .param("world_id", world_id),
            "e",
        )
        .await
    }

    /// Page world entities by ID, with an optional exact kind filter.
    /// Callers must check world ownership before calling this method.
    #[tracing::instrument(name = "repository.graph.list_entities", level = "debug", skip_all)]
    pub(super) async fn list_entities(
        &self,
        world_id: &str,
        limit: u32,
        cursor: Option<&str>,
        kind: Option<&str>,
    ) -> Result<(Vec<Entity>, Option<String>), InfrastructureError> {
        if !(1..=100).contains(&limit) {
            return Err(InfrastructureError::InvalidInput(
                "Entity page limit must be between 1 and 100".into(),
            ));
        }
        if let Some(cursor) = cursor {
            uuid::Uuid::parse_str(cursor)
                .map_err(|_| InfrastructureError::InvalidInput("Invalid entity cursor".into()))?;
        }
        let mut entities = fetch_all::<Entity>(
            &self.graph,
            query(
                "MATCH (e:Entity {world_id: $world_id})
                     WHERE e.id > $after_id AND ($kind IS NULL OR $kind IN e.kind)
                     RETURN e ORDER BY e.id LIMIT $limit",
            )
            .param("world_id", world_id)
            .param("after_id", cursor.unwrap_or_default())
            .param("kind", kind.map(str::to_owned))
            .param("limit", i64::from(limit) + 1),
            "e",
        )
        .await?;
        let has_more = entities.len() > limit as usize;
        entities.truncate(limit as usize);
        let next_cursor = if has_more {
            entities.last().map(|entity| entity.id.clone())
        } else {
            None
        };
        Ok((entities, next_cursor))
    }

    #[tracing::instrument(
        name = "repository.graph.get_entities_by_world",
        level = "debug",
        skip_all
    )]
    pub(super) async fn get_entities_by_world(
        &self,
        world_id: &str,
    ) -> Result<Vec<Entity>, InfrastructureError> {
        fetch_all::<Entity>(
            &self.graph,
            query("MATCH (e:Entity {world_id: $world_id}) RETURN e ORDER BY e.name_key")
                .param("world_id", world_id),
            "e",
        )
        .await
    }

    /// Entities whose name or any alias matches one of `names` (compared via `name_key`).
    /// This is how extraction finds `existing_entities` for a message.
    #[tracing::instrument(
        name = "repository.graph.find_entities_by_names",
        level = "debug",
        skip_all
    )]
    pub(super) async fn find_entities_by_names(
        &self,
        world_id: &str,
        names: &[String],
    ) -> Result<Vec<Entity>, InfrastructureError> {
        let keys: Vec<String> = names.iter().map(|n| name_key(n)).collect();
        fetch_all::<Entity>(
            &self.graph,
            query(
                "MATCH (e:Entity {world_id: $world_id})
             WHERE e.name_key IN $keys
                OR any(a IN e.aliases WHERE toLower(trim(a)) IN $keys)
             RETURN e",
            )
            .param("world_id", world_id)
            .param("keys", keys),
            "e",
        )
        .await
    }

    #[tracing::instrument(
        name = "repository.graph.get_entity_neighbors",
        level = "debug",
        skip_all
    )]
    pub(super) async fn get_entity_neighbors(
        &self,
        id: &str,
        world_id: &str,
        rel_type: Option<&str>,
        kind: Option<&str>,
    ) -> Result<Vec<(Entity, Relationship)>, InfrastructureError> {
        if let Some(rel_type) = rel_type {
            validate_cypher_identifier(rel_type)?;
        }

        let rel = rel_type.map(|r| format!(":{r}")).unwrap_or_default();
        let kind_filter = kind.map(|_| "WHERE $kind IN m.kind").unwrap_or_default();

        let cypher = format!(
            "MATCH (e:Entity {{id: $id, world_id: $world_id}})-[r{rel}]-(m:Entity)
         {kind_filter}
         RETURN r, m"
        );

        let mut q = query(&cypher).param("id", id).param("world_id", world_id);
        if let Some(k) = kind {
            q = q.param("kind", k);
        }

        let mut result = self.graph.execute(q).await?;
        let mut items = Vec::new();
        while let Some(row) = result.next().await? {
            let entity = Entity::from_row(&row, "m")?;
            let relationship = Relationship::from_row(&row, "r")?;
            items.push((entity, relationship));
        }
        Ok(items)
    }

    /// Page connecting relationships by ID and return their neighboring entities.
    /// The cursor is the last returned relationship ID; callers must check world ownership.
    #[tracing::instrument(
        name = "repository.graph.get_entity_neighbors_page",
        level = "debug",
        skip_all
    )]
    pub(super) async fn get_entity_neighbors_page(
        &self,
        id: &str,
        world_id: &str,
        rel_type: Option<&str>,
        kind: Option<&str>,
        limit: u32,
        cursor: Option<&str>,
    ) -> Result<(Vec<(Entity, Relationship)>, Option<String>), InfrastructureError> {
        if !(1..=100).contains(&limit) {
            return Err(InfrastructureError::InvalidInput(
                "Neighbor page limit must be between 1 and 100".into(),
            ));
        }
        if let Some(cursor) = cursor {
            uuid::Uuid::parse_str(cursor)
                .map_err(|_| InfrastructureError::InvalidInput("Invalid neighbor cursor".into()))?;
        }
        let mut rows = self.graph.execute(
            query(
                "MATCH (e:Entity {id: $id, world_id: $world_id})-[r]-(m:Entity {world_id: $world_id})
                 WHERE r.world_id = $world_id AND r.id > $after_id
                   AND ($rel_type IS NULL OR type(r) = $rel_type)
                   AND ($kind IS NULL OR $kind IN m.kind)
                 RETURN DISTINCT r, m ORDER BY r.id LIMIT $limit",
            )
            .param("id", id)
            .param("world_id", world_id)
            .param("after_id", cursor.unwrap_or_default())
            .param("rel_type", rel_type.map(str::to_owned))
            .param("kind", kind.map(str::to_owned))
            .param("limit", i64::from(limit) + 1),
        ).await?;
        let mut items = Vec::new();
        while let Some(row) = rows.next().await? {
            items.push((
                Entity::from_row(&row, "m")?,
                Relationship::from_row(&row, "r")?,
            ));
        }
        let has_more = items.len() > limit as usize;
        items.truncate(limit as usize);
        let next_cursor = if has_more {
            items
                .last()
                .map(|(_, relationship)| relationship.id.clone())
        } else {
            None
        };
        Ok((items, next_cursor))
    }

    #[tracing::instrument(
        name = "repository.graph.get_entities_by_conversation",
        level = "debug",
        skip_all
    )]
    pub(super) async fn get_entities_by_conversation(
        &self,
        conversation_id: &str,
        world_id: &str,
    ) -> Result<Vec<Entity>, InfrastructureError> {
        fetch_all::<Entity>(
            &self.graph,
            query(
                "MATCH (e:Entity {world_id: $world_id})-[r]-()
             WHERE r.source_conversation = $conversation_id
             RETURN DISTINCT e",
            )
            .param("conversation_id", conversation_id)
            .param("world_id", world_id),
            "e",
        )
        .await
    }

    #[tracing::instrument(
        name = "repository.graph.vector_search_entities",
        level = "debug",
        skip_all
    )]
    pub(super) async fn vector_search_entities(
        &self,
        world_id: &str,
        query_embedding: Vec<f64>,
        limit: u32,
    ) -> Result<Vec<Entity>, InfrastructureError> {
        let results = fetch_all::<Entity>(&self.graph,
                query(
                    r#"
                CALL vector_search.search("entity_embedding", $candidates, $embedding) YIELD node, similarity
                WHERE node.world_id = $world_id
                RETURN node, similarity
                ORDER BY similarity DESC
                LIMIT $limit
                "#,
                )
                .param("world_id", world_id)
                .param("embedding", query_embedding)
                .param("candidates", i64::from(limit.max(50)))
                .param("limit", i64::from(limit)),
                "node",
            )
            .await?;
        Ok(results)
    }

    #[tracing::instrument(
        name = "repository.graph.fts_search_entities",
        level = "debug",
        skip_all
    )]
    pub(super) async fn fts_search_entities(
        &self,
        world_id: &str,
        query_text: &str,
        limit: u32,
    ) -> Result<Vec<Entity>, InfrastructureError> {
        let cleaned: String = query_text
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { ' ' })
            .collect();

        if cleaned.trim().is_empty() {
            return Ok(Vec::new());
        }

        let results = fetch_all::<Entity>(
            &self.graph,
            query(
                r#"
                    CALL text_search.search_all("entity_text", $text) YIELD node, score
                    WITH node, score
                    WHERE node.world_id = $world_id
                    RETURN node
                    ORDER BY score DESC
                    LIMIT $limit
                "#,
            )
            .param("world_id", world_id)
            .param("text", cleaned)
            .param("limit", i64::from(limit)),
            "node",
        )
        .await?;
        Ok(results)
    }

    pub(super) async fn graph_page(
        &self,
        world_id: &str,
        limit: u32,
        cursor: Option<&str>,
    ) -> Result<(Vec<Entity>, Option<String>), InfrastructureError> {
        if !(1..=100).contains(&limit) {
            return Err(InfrastructureError::InvalidInput(
                "Graph page limit must be between 1 and 100".into(),
            ));
        }
        if let Some(cursor) = cursor {
            uuid::Uuid::parse_str(cursor)
                .map_err(|_| InfrastructureError::InvalidInput("Invalid graph cursor".into()))?;
        }
        let mut entities = fetch_all::<Entity>(
            &self.graph,
            query(
                "MATCH (e:Entity {world_id: $world_id})
                   WHERE e.id > $after_id
                   RETURN e ORDER BY e.id LIMIT $limit",
            )
            .param("world_id", world_id)
            .param("after_id", cursor.unwrap_or_default())
            .param("limit", i64::from(limit) + 1),
            "e",
        )
        .await?;
        let has_more = entities.len() > limit as usize;
        entities.truncate(limit as usize);
        let next_cursor = if has_more {
            entities.last().map(|entity| entity.id.clone())
        } else {
            None
        };
        Ok((entities, next_cursor))
    }

    pub(super) async fn upsert_in_transaction(
        &self,
        txn: &mut neo4rs::Txn,
        world_id: &str,
        e: &Entity,
        embedding: &[f64],
        source_message_id: &str,
    ) -> Result<(), InfrastructureError> {
        txn.run(
            query(
                "MERGE (e:Entity {world_id: $world_id, id: $id})
                     SET e.name = $name, e.name_key = $name_key, e.aliases = $aliases,
                         e.kind = $kind, e.description = $description,
                         e.properties = $properties, e.embedding = $embedding,
                         e.source_message_ids =
                            CASE
                                WHEN $source_message_id IN coalesce(e.source_message_ids, [])
                                THEN e.source_message_ids
                                ELSE coalesce(e.source_message_ids, []) + [$source_message_id]
                            END
                         ",
            )
            .param("world_id", world_id)
            .param("id", e.id.clone())
            .param("name", e.name.clone())
            .param("name_key", name_key(&e.name))
            .param("aliases", e.aliases.clone())
            .param("kind", e.kind.clone())
            .param("description", e.description.clone())
            .param("properties", serde_json::to_string(&e.properties).unwrap())
            .param("embedding", embedding.to_vec())
            .param("source_message_id", source_message_id),
        )
        .await?;
        Ok(())
    }

    pub(super) async fn save_accepted_conflict(
        &self,
        txn: &mut neo4rs::Txn,
        world_id: &str,
        entity: Entity,
        new_description: &str,
        new_embedding: &[f64],
    ) -> Result<(), InfrastructureError> {
        let properties = serde_json::to_string(&entity.properties)
            .map_err(|error| InfrastructureError::InternalError(error.to_string()))?;
        txn.run(
            query(
                "MATCH (e:Entity {id: $id, world_id: $world_id})
                       SET e.properties = $properties, e.source_message_ids = $sources,
                           e.description = $description, e.embedding = $embedding",
            )
            .param("id", entity.id)
            .param("world_id", world_id)
            .param("properties", properties)
            .param("description", new_description)
            .param("embedding", new_embedding.to_vec())
            .param("sources", entity.source_message_ids),
        )
        .await?;
        Ok(())
    }
}
