use neo4rs::{query, Graph, Query};
use crate::data::models::{Entity, FromRow, Relationship};
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

    // create a relationship between entities

    // update a relationship between entities

    // delete a relationship between entities

    // create a convo

    // update a convo

    // delete a convo

    // get a convo by id

    // create a conflict

    // update a conflict

    // get a conflict by id

    // get conflicts by entity

    // get relationships by entity

    // get entities by conversation
}