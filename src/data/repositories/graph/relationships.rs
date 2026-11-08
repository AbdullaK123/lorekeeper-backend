use super::helpers::{fetch_all, fetch_one, validate_cypher_identifier};
use crate::data::models::Relationship;
use crate::infrastructure::InfrastructureError;
use neo4rs::{Graph, query};

#[derive(Clone)]
pub(super) struct RelationshipRepository {
    graph: Graph,
}

impl RelationshipRepository {
    pub(super) fn new(graph: Graph) -> Self {
        Self { graph }
    }

    #[tracing::instrument(name = "repository.graph.get_relationship", level = "debug", skip_all)]
    pub(super) async fn get_relationship(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<Option<Relationship>, InfrastructureError> {
        fetch_one::<Relationship>(&self.graph,
            query("MATCH (a:Entity {world_id: $world_id})-[r {id: $id, world_id: $world_id}]->(b:Entity {world_id: $world_id}) RETURN r LIMIT 1")
                .param("id", id)
                .param("world_id", world_id),
            "r",
        ).await
    }

    #[tracing::instrument(
        name = "repository.graph.update_relationship",
        level = "debug",
        skip_all
    )]
    pub(super) async fn update_relationship(
        &self,
        id: &str,
        world_id: &str,
        relationship: &Relationship,
    ) -> Result<bool, InfrastructureError> {
        let mut result = self
            .graph
            .execute(
                query(
                    "MATCH ()-[r {id: $id, world_id: $world_id}]->()
             SET r.description = $description,
                 r.properties = $properties
             RETURN r.id AS id",
                )
                .param("id", id)
                .param("world_id", world_id)
                .param("description", relationship.description.clone())
                .param(
                    "properties",
                    serde_json::to_string(&relationship.properties).unwrap_or_default(),
                ),
            )
            .await?;
        Ok(result.next().await?.is_some())
    }

    #[tracing::instrument(
        name = "repository.graph.delete_relationship",
        level = "debug",
        skip_all
    )]
    pub(super) async fn delete_relationship(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<bool, InfrastructureError> {
        // same two-step pattern but with world_id
        let mut result = self
            .graph
            .execute(
                query("MATCH ()-[r {id: $id, world_id: $world_id}]->() RETURN r.id AS id")
                    .param("id", id)
                    .param("world_id", world_id),
            )
            .await?;
        let exists = result.next().await?.is_some();
        if exists {
            self.graph
                .run(
                    query("MATCH ()-[r {id: $id, world_id: $world_id}]->() DELETE r")
                        .param("id", id)
                        .param("world_id", world_id),
                )
                .await?;
        }
        Ok(exists)
    }

    // --- Cross-type queries ---

    #[tracing::instrument(
        name = "repository.graph.get_relationships_by_entity",
        level = "debug",
        skip_all
    )]
    pub(super) async fn get_relationships_by_entity(
        &self,
        entity_id: &str,
        world_id: &str,
    ) -> Result<Vec<Relationship>, InfrastructureError> {
        fetch_all::<Relationship>(
            &self.graph,
            query(
                "MATCH (e:Entity {id: $entity_id, world_id: $world_id})-[r]-()
             RETURN r",
            )
            .param("entity_id", entity_id)
            .param("world_id", world_id),
            "r",
        )
        .await
    }

    #[tracing::instrument(
        name = "repository.graph.delete_relationships_by_endpoints",
        level = "debug",
        skip_all
    )]
    pub(super) async fn delete_relationships_by_endpoints(
        &self,
        world_id: &str,
        from_id: &str,
        to_id: &str,
        rel_type: &str,
    ) -> Result<(), InfrastructureError> {
        validate_cypher_identifier(rel_type)?;

        let rel = format!(":{}", rel_type);

        let cypher = format!(
            r#"
                MATCH (a: Entity) -[r{}]-> (b: Entity)
                WHERE a.id = $from_id
                    AND b.id = $to_id
                    AND a.world_id = $world_id
                    AND b.world_id = $world_id
                DELETE r
            "#,
            rel
        );

        Ok(self
            .graph
            .run(
                query(&cypher)
                    .param("from_id", from_id)
                    .param("to_id", to_id)
                    .param("world_id", world_id),
            )
            .await?)
    }

    pub(super) async fn within_entities(
        &self,
        world_id: &str,
        ids: Vec<&str>,
    ) -> Result<Vec<Relationship>, InfrastructureError> {
        let relationships = super::helpers::fetch_all::<Relationship>(
            &self.graph,
            query(
                "MATCH (a:Entity {world_id: $world_id})-[r]->(b:Entity {world_id: $world_id})
                   WHERE a.id IN $ids AND b.id IN $ids AND r.world_id = $world_id
                   RETURN r ORDER BY r.id",
            )
            .param("world_id", world_id)
            .param("ids", ids),
            "r",
        )
        .await?;
        Ok(relationships)
    }

    pub(super) async fn upsert_in_transaction(
        &self,
        txn: &mut neo4rs::Txn,
        world_id: &str,
        r: &Relationship,
        source_message_id: &str,
    ) -> Result<(), InfrastructureError> {
        // An existing ID must still describe the same directed connection.
        let mut existing = txn
            .execute(
                query(
                    "MATCH (a)-[r]->(b) WHERE r.world_id = $world_id AND r.id = $id
                     RETURN a.id AS from_id, b.id AS to_id, type(r) AS rel_type",
                )
                .param("world_id", world_id)
                .param("id", r.id.clone()),
            )
            .await?;
        while let Some(row) = existing.next(&mut *txn).await? {
            if row.get::<String>("from_id")? != r.from_id
                || row.get::<String>("to_id")? != r.to_id
                || row.get::<String>("rel_type")? != r.rel_type
            {
                return Err(InfrastructureError::InternalError(
                    "Relationship ID already belongs to a different connection".into(),
                ));
            }
        }
        let mut saved = txn
                    .execute(
                        query(&format!(
                            "MATCH (a:Entity {{world_id: $world_id, id: $from_id}}),
                                (b:Entity {{world_id: $world_id, id: $to_id}})
                            MERGE (a)-[r:{} {{world_id: $world_id, id: $id}}]->(b)
                            ON CREATE SET r.source_conversation = $source
                            SET r.from_id = $from_id, r.to_id = $to_id,
                                r.description = $description, r.properties = $properties,
                                r.source_message_ids =
                                    CASE
                                        WHEN $source_message_id IN coalesce(r.source_message_ids, [])
                                        THEN r.source_message_ids
                                        ELSE coalesce(r.source_message_ids, []) + [$source_message_id]
                                    END
                            RETURN r.id AS id",
                            r.rel_type
                        ))
                        .param("world_id", world_id)
                        .param("id", r.id.clone())
                        .param("from_id", r.from_id.clone())
                        .param("to_id", r.to_id.clone())
                        .param("description", r.description.clone())
                        .param("properties", serde_json::to_string(&r.properties).unwrap())
                        .param("source", r.source_conversation.clone())
                        .param("source_message_id", source_message_id),
                    )
                    .await?;
        if saved.next(&mut *txn).await?.is_none() {
            return Err(InfrastructureError::InternalError(
                "Relationship endpoint not found in this world; batch rolled back".into(),
            ));
        }
        Ok(())
    }

    pub(super) async fn retract_in_transaction(
        &self,
        txn: &mut neo4rs::Txn,
        world_id: &str,
        id: &str,
    ) -> Result<(), InfrastructureError> {
        txn.run(
            query("MATCH ()-[r]->() WHERE r.world_id = $world_id AND r.id = $id DELETE r")
                .param("world_id", world_id)
                .param("id", id),
        )
        .await?;
        Ok(())
    }
}
