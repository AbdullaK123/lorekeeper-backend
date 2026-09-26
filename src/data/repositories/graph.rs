use neo4rs::{query, Graph, Query};
use crate::data::models::{Conflict, Conversation, Entity, FromRow, Relationship};
use crate::infrastructure::InfrastructureError;

#[derive(Clone)]
pub struct GraphRepository {
    graph: Graph
}

impl GraphRepository {
    pub fn new(graph: Graph) -> Self {
        Self {
            graph
        }
    }

   async fn fetch_all<T: FromRow>(
        &self,
        q: Query,
        alias: &str
    )  -> Result<Vec<T>, InfrastructureError> {
       let mut result = self.graph.execute(q).await?;
       let mut rows = Vec::new();
       while let Some(row) = result.next().await? {
           rows.push(T::from_row(&row, alias)?)
       }
       Ok(rows)
    }

    async fn fetch_one<T: FromRow>(
        &self,
        q: Query,
        alias: &str
    ) -> Result<Option<T>, InfrastructureError> {
        let mut result = self.graph.execute(q).await?;
        match result.next().await? {
            Some(row) => Ok(Some(T::from_row(&row, alias)?)),
            None => Ok(None)
        }
    }

    pub async fn create_entity(
        &self,
        entity: &Entity
    ) -> Result<(), InfrastructureError> {
        self.graph.run(
            query(
               "CREATE (e:Entity {
                   id: $id,
                   name: $name,
                   aliases: $aliases,
                   kind: $kind,
                   description: $description,
                   properties: $properties
               })"
            )
                .param("id", entity.id.clone())
                .param("name", entity.name.clone())
                .param("aliases", entity.aliases.clone())
                .param("kind", entity.kind.clone())
                .param("description", entity.description.clone())
                .param(
                    "properties",
                    serde_json::to_string(&entity.properties)
                        .unwrap_or_default()
                )
        ).await?;
        Ok(())
    }

    pub async fn update_entity(
        &self,
        id: &str,
        entity: &Entity,
    ) -> Result<bool, InfrastructureError> {
        let mut result = self.graph.execute(
            query(
                "MATCH (e:Entity {id: $id})
             SET e.name = $name,
                 e.aliases = $aliases,
                 e.kind = $kind,
                 e.description = $description,
                 e.properties = $properties
             RETURN e.id AS id"
            )
                .param("id", id)
                .param("name", entity.name.clone())
                .param("aliases", entity.aliases.clone())
                .param("kind", entity.kind.clone())
                .param("description", entity.description.clone())
                .param("properties", serde_json::to_string(&entity.properties).unwrap_or_default())
        ).await?;
        Ok(result.next().await?.is_some())
    }

    pub async fn delete_entity(
        &self,
        id: &str,
    ) -> Result<bool, InfrastructureError> {
        let mut result = self.graph.execute(
            query("MATCH (e:Entity {id: $id}) RETURN e.id AS id")
                .param("id", id)
        ).await?;
        let exists = result.next().await?.is_some();
        if exists {
            self.graph.run(
                query("MATCH (e:Entity {id: $id}) DETACH DELETE e")
                    .param("id", id)
            ).await?;
        }
        Ok(exists)
    }

    pub async fn get_entity(
        &self,
        id: &str
    ) -> Result<Option<Entity>, InfrastructureError> {
        self.fetch_one::<Entity>(
            query("MATCH (e: Entity {id: $id}) RETURN e")
                .param("id", id),
            "e"
        ).await
    }

    pub async fn get_entity_neighbors(
        &self,
        id: &str,
        rel_type: Option<&str>,
        kind: Option<&str>,
    ) -> Result<Vec<(Entity, Relationship)>, InfrastructureError> {
        let rel = rel_type.map(|r| format!(":{r}")).unwrap_or_default();
        let kind_filter = kind
            .map(|_| "WHERE $kind IN m.kind")
            .unwrap_or_default();

        let cypher = format!(
            "MATCH (e:Entity {{id: $id}})-[r{rel}]-(m:Entity)
         {kind_filter}
         RETURN r, m"
        );

        let mut q = query(&cypher).param("id", id);
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

    pub async fn create_relationship(
        &self,
        relationship: &Relationship,
    ) -> Result<(), InfrastructureError> {
        let cypher = format!(
            "MATCH (a:Entity {{id: $from_id}}), (b:Entity {{id: $to_id}})
         CREATE (a)-[r:{} {{
             id: $id,
             from_id: $from_id,
             to_id: $to_id,
             description: $description,
             properties: $properties,
             source_conversation: $source_conversation
         }}]->(b)",
            relationship.rel_type
        );

        self.graph.run(
            query(&cypher)
                .param("id", relationship.id.clone())
                .param("from_id", relationship.from_id.clone())
                .param("to_id", relationship.to_id.clone())
                .param("description", relationship.description.clone())
                .param("properties", serde_json::to_string(&relationship.properties).unwrap_or_default())
                .param("source_conversation", relationship.source_conversation.clone())
        ).await?;
        Ok(())
    }

    pub async fn update_relationship(
        &self,
        id: &str,
        relationship: &Relationship,
    ) -> Result<bool, InfrastructureError> {
        let mut result = self.graph.execute(
            query(
                "MATCH ()-[r {id: $id}]-()
             SET r.description = $description,
                 r.properties = $properties
             RETURN r.id AS id"
            )
                .param("id", id)
                .param("description", relationship.description.clone())
                .param("properties", serde_json::to_string(&relationship.properties).unwrap_or_default())
        ).await?;
        Ok(result.next().await?.is_some())
    }

    pub async fn delete_relationship(
        &self,
        id: &str,
    ) -> Result<bool, InfrastructureError> {
        let mut result = self.graph.execute(
            query("MATCH ()-[r {id: $id}]-() RETURN r.id AS id")
                .param("id", id)
        ).await?;
        let exists = result.next().await?.is_some();
        if exists {
            self.graph.run(
                query("MATCH ()-[r {id: $id}]-() DELETE r")
                    .param("id", id)
            ).await?;
        }
        Ok(exists)
    }

    // --- Conversations ---

    pub async fn create_conversation(
        &self,
        conversation: &Conversation,
    ) -> Result<(), InfrastructureError> {
        self.graph.run(
            query(
                "CREATE (c:Conversation {
                    id: $id,
                    created_at: $created_at,
                    summary: $summary
                })"
            )
                .param("id", conversation.id.clone())
                .param("created_at", conversation.created_at.clone())
                .param("summary", conversation.summary.clone().unwrap_or_default())
        ).await?;
        Ok(())
    }

    pub async fn update_conversation(
        &self,
        id: &str,
        conversation: &Conversation,
    ) -> Result<bool, InfrastructureError> {
        let mut result = self.graph.execute(
            query(
                "MATCH (c:Conversation {id: $id})
                 SET c.summary = $summary
                 RETURN c.id AS id"
            )
                .param("id", id)
                .param("summary", conversation.summary.clone().unwrap_or_default())
        ).await?;
        Ok(result.next().await?.is_some())
    }

    pub async fn delete_conversation(
        &self,
        id: &str,
    ) -> Result<bool, InfrastructureError> {
        let mut result = self.graph.execute(
            query("MATCH (c:Conversation {id: $id}) RETURN c.id AS id")
                .param("id", id)
        ).await?;
        let exists = result.next().await?.is_some();
        if exists {
            self.graph.run(
                query("MATCH (c:Conversation {id: $id}) DETACH DELETE c")
                    .param("id", id)
            ).await?;
        }
        Ok(exists)
    }

    pub async fn get_conversation(
        &self,
        id: &str,
    ) -> Result<Option<Conversation>, InfrastructureError> {
        self.fetch_one::<Conversation>(
            query("MATCH (c:Conversation {id: $id}) RETURN c")
                .param("id", id),
            "c"
        ).await
    }

    // --- Conflicts ---

    pub async fn create_conflict(
        &self,
        conflict: &Conflict,
    ) -> Result<(), InfrastructureError> {
        self.graph.run(
            query(
                "MATCH (e:Entity {id: $entity_id})
                 CREATE (e)-[:HAS_CONFLICT]->(c:Conflict {
                     id: $id,
                     entity_id: $entity_id,
                     field: $field,
                     existing_value: $existing_value,
                     new_value: $new_value,
                     source_conversation: $source_conversation,
                     resolved: $resolved
                 })"
            )
                .param("id", conflict.id.clone())
                .param("entity_id", conflict.entity_id.clone())
                .param("field", conflict.field.clone())
                .param("existing_value", serde_json::to_string(&conflict.existing_value).unwrap_or_default())
                .param("new_value", serde_json::to_string(&conflict.new_value).unwrap_or_default())
                .param("source_conversation", conflict.source_conversation.clone())
                .param("resolved", conflict.resolved)
        ).await?;
        Ok(())
    }

    pub async fn update_conflict(
        &self,
        id: &str,
        conflict: &Conflict,
    ) -> Result<bool, InfrastructureError> {
        let mut result = self.graph.execute(
            query(
                "MATCH (c:Conflict {id: $id})
                 SET c.resolved = $resolved
                 RETURN c.id AS id"
            )
                .param("id", id)
                .param("resolved", conflict.resolved)
        ).await?;
        Ok(result.next().await?.is_some())
    }

    pub async fn get_conflict(
        &self,
        id: &str,
    ) -> Result<Option<Conflict>, InfrastructureError> {
        self.fetch_one::<Conflict>(
            query("MATCH (c:Conflict {id: $id}) RETURN c")
                .param("id", id),
            "c"
        ).await
    }

    pub async fn get_conflicts_by_entity(
        &self,
        entity_id: &str,
    ) -> Result<Vec<Conflict>, InfrastructureError> {
        self.fetch_all::<Conflict>(
            query(
                "MATCH (e:Entity {id: $entity_id})-[:HAS_CONFLICT]->(c:Conflict)
                 RETURN c"
            )
                .param("entity_id", entity_id),
            "c"
        ).await
    }

    // --- Cross-type queries ---

    pub async fn get_relationships_by_entity(
        &self,
        entity_id: &str,
    ) -> Result<Vec<Relationship>, InfrastructureError> {
        self.fetch_all::<Relationship>(
            query(
                "MATCH (e:Entity {id: $entity_id})-[r]-()
                 RETURN r"
            )
                .param("entity_id", entity_id),
            "r"
        ).await
    }

    pub async fn get_entities_by_conversation(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<Entity>, InfrastructureError> {
        self.fetch_all::<Entity>(
            query(
                "MATCH (e:Entity)-[r]-()
                 WHERE r.source_conversation = $conversation_id
                 RETURN DISTINCT e"
            )
                .param("conversation_id", conversation_id),
            "e"
        ).await
    }
}