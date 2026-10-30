use crate::data::models::{Conflict, Conversation, Entity, FromRow, Relationship, name_key};
use crate::infrastructure::InfrastructureError;
use neo4rs::{Graph, Query, query};

#[derive(Clone)]
pub struct GraphRepository {
    graph: Graph,
}

impl GraphRepository {
    pub fn new(graph: Graph) -> Self {
        Self { graph }
    }

    fn validate_cypher_identifier(value: &str) -> Result<(), InfrastructureError> {
        if value.is_empty()
            || !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            || value.chars().next().unwrap().is_ascii_digit()
        {
            return Err(InfrastructureError::CypherInjection(value.to_string()));
        }
        Ok(())
    }

    async fn fetch_all<T: FromRow>(
        &self,
        q: Query,
        alias: &str,
    ) -> Result<Vec<T>, InfrastructureError> {
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
        alias: &str,
    ) -> Result<Option<T>, InfrastructureError> {
        let mut result = self.graph.execute(q).await?;
        match result.next().await? {
            Some(row) => Ok(Some(T::from_row(&row, alias)?)),
            None => Ok(None),
        }
    }

    /// Idempotent: Memgraph skips a constraint that already exists. Run once at startup.
    pub async fn ensure_constraints(&self) -> Result<(), InfrastructureError> {
        self.graph
            .run(query(
                "CREATE CONSTRAINT ON (e:Entity) ASSERT e.world_id, e.id IS UNIQUE",
            ))
            .await?;
        self.graph
            .run(query(
                "CREATE CONSTRAINT ON (e:Entity) ASSERT e.world_id, e.name_key IS UNIQUE",
            ))
            .await?;
        self.graph
            .run(query(
                r#"CREATE VECTOR INDEX entity_embedding ON :Entity(embedding)
                WITH CONFIG {"dimension": 1536, "capacity": 100000, "metric": "cos"}
                "#,
            ))
            .await?;
        self.graph
            .run(query(
                r#"CREATE TEXT INDEX entity_text ON :Entity(name, description)"#,
            ))
            .await?;
        Ok(())
    }

    pub async fn update_entity(
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

    pub async fn delete_entity(
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

    pub async fn get_entity(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<Option<Entity>, InfrastructureError> {
        self.fetch_one::<Entity>(
            query("MATCH (e:Entity {id: $id, world_id: $world_id}) RETURN e")
                .param("id", id)
                .param("world_id", world_id),
            "e",
        )
        .await
    }

    pub async fn get_entities_by_world(
        &self,
        world_id: &str,
    ) -> Result<Vec<Entity>, InfrastructureError> {
        self.fetch_all::<Entity>(
            query("MATCH (e:Entity {world_id: $world_id}) RETURN e ORDER BY e.name_key")
                .param("world_id", world_id),
            "e",
        )
        .await
    }

    /// Entities whose name or any alias matches one of `names` (compared via `name_key`).
    /// This is how extraction finds `existing_entities` for a message.
    pub async fn find_entities_by_names(
        &self,
        world_id: &str,
        names: &[String],
    ) -> Result<Vec<Entity>, InfrastructureError> {
        let keys: Vec<String> = names.iter().map(|n| name_key(n)).collect();
        self.fetch_all::<Entity>(
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

    pub async fn get_entity_neighbors(
        &self,
        id: &str,
        world_id: &str,
        rel_type: Option<&str>,
        kind: Option<&str>,
    ) -> Result<Vec<(Entity, Relationship)>, InfrastructureError> {
        if let Some(rel_type) = rel_type {
            Self::validate_cypher_identifier(rel_type)?;
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

    pub async fn update_relationship(
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

    pub async fn delete_relationship(
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

    // --- Conversations ---

    pub async fn create_conversation(
        &self,
        conversation: &Conversation,
    ) -> Result<(), InfrastructureError> {
        self.graph
            .run(
                query(
                    "CREATE (c:Conversation {
                    id: $id,
                    world_id: $world_id,
                    created_at: $created_at,
                    summary: $summary
                })",
                )
                .param("id", conversation.id.clone())
                .param("world_id", conversation.world_id.clone())
                .param("created_at", conversation.created_at.clone())
                .param("summary", conversation.summary.clone().unwrap_or_default()),
            )
            .await?;
        Ok(())
    }

    pub async fn update_conversation(
        &self,
        id: &str,
        world_id: &str,
        conversation: &Conversation,
    ) -> Result<bool, InfrastructureError> {
        let mut result = self
            .graph
            .execute(
                query(
                    "MATCH (c:Conversation {id: $id, world_id: $world_id})
             SET c.summary = $summary
             RETURN c.id AS id",
                )
                .param("id", id)
                .param("world_id", world_id)
                .param("summary", conversation.summary.clone().unwrap_or_default()),
            )
            .await?;
        Ok(result.next().await?.is_some())
    }

    pub async fn delete_conversation(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<bool, InfrastructureError> {
        let mut result = self
            .graph
            .execute(
                query("MATCH (c:Conversation {id: $id, world_id: $world_id}) RETURN c.id AS id")
                    .param("id", id)
                    .param("world_id", world_id),
            )
            .await?;
        let exists = result.next().await?.is_some();
        if exists {
            self.graph
                .run(
                    query("MATCH (c:Conversation {id: $id, world_id: $world_id}) DETACH DELETE c")
                        .param("id", id)
                        .param("world_id", world_id),
                )
                .await?;
        }
        Ok(exists)
    }

    pub async fn get_conversation(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<Option<Conversation>, InfrastructureError> {
        self.fetch_one::<Conversation>(
            query("MATCH (c:Conversation {id: $id, world_id: $world_id}) RETURN c")
                .param("id", id)
                .param("world_id", world_id),
            "c",
        )
        .await
    }

    // --- Conflicts ---

    pub async fn get_conflicts(
        &self,
        world_id: &str,
    ) -> Result<Vec<Conflict>, InfrastructureError> {
        Ok(self
            .fetch_all::<Conflict>(
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

    pub async fn update_conflict(
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

    pub async fn get_conflict(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<Option<Conflict>, InfrastructureError> {
        self.fetch_one::<Conflict>(
            query("MATCH (c:Conflict {id: $id, world_id: $world_id}) RETURN c")
                .param("id", id)
                .param("world_id", world_id),
            "c",
        )
        .await
    }

    pub async fn get_conflicts_by_entity(
        &self,
        entity_id: &str,
        world_id: &str,
    ) -> Result<Vec<Conflict>, InfrastructureError> {
        self.fetch_all::<Conflict>(
            query(
                "MATCH (e:Entity {id: $entity_id, world_id: $world_id})-[:HAS_CONFLICT]->(c:Conflict)
             RETURN c"
            )
                .param("entity_id", entity_id)
                .param("world_id", world_id),
            "c"
        ).await
    }

    // --- Cross-type queries ---

    pub async fn get_relationships_by_entity(
        &self,
        entity_id: &str,
        world_id: &str,
    ) -> Result<Vec<Relationship>, InfrastructureError> {
        self.fetch_all::<Relationship>(
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

    pub async fn get_entities_by_conversation(
        &self,
        conversation_id: &str,
        world_id: &str,
    ) -> Result<Vec<Entity>, InfrastructureError> {
        self.fetch_all::<Entity>(
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

    /// Exact-ID writes. Any failed validation or write rolls back the entire batch.
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
            Self::validate_cypher_identifier(&r.rel_type)?;
            if r.world_id != world_id || retracted_ids.contains(&r.id) {
                return Err(InfrastructureError::InternalError(
                    "Invalid relationship batch".into(),
                ));
            }
        }
        let mut txn = self.graph.start_txn().await?;
        let result: Result<(), InfrastructureError> = async {
            for (e, embedding) in entities {
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
                    .param("embedding", embedding.clone())
                    .param("source_message_id", source_message_id),
                )
                .await?;
            }
            for r in relationships {
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
                while let Some(row) = existing.next(&mut txn).await? {
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
                if saved.next(&mut txn).await?.is_none() {
                    return Err(InfrastructureError::InternalError(
                        "Relationship endpoint not found in this world; batch rolled back".into(),
                    ));
                }
            }
            for id in retracted_ids {
                txn.run(
                    query("MATCH ()-[r]->() WHERE r.world_id = $world_id AND r.id = $id DELETE r")
                        .param("world_id", world_id)
                        .param("id", id.clone()),
                )
                .await?;
            }
            Ok(())
        }
        .await;
        match result {
            Ok(()) => {
                txn.commit().await?;
                Ok(())
            }
            Err(error) => {
                txn.rollback().await?;
                Err(error)
            }
        }
    }

    // One open conflict per entity field: a retry, or a later message contradicting
    // the same field, updates that conflict instead of adding another.
    // Returns None when the entity doesn't exist in this world.
    pub async fn upsert_conflict(
        &self,
        conflict: &Conflict,
        source_message_id: &str,
    ) -> Result<Option<Conflict>, InfrastructureError> {
        self.fetch_one::<Conflict>(
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

    pub async fn vector_search_entities(
        &self,
        world_id: &str,
        query_embedding: Vec<f64>,
    ) -> Result<Vec<Entity>, InfrastructureError> {
        let results = self
            .fetch_all::<Entity>(
                query(
                    r#"
                CALL vector_search.search("entity_embedding", 50, $embedding) YIELD node, similarity
                WHERE node.world_id = $world_id
                RETURN node, similarity
                ORDER BY similarity DESC
                LIMIT 10
                "#,
                )
                .param("world_id", world_id)
                .param("embedding", query_embedding),
                "node",
            )
            .await?;
        Ok(results)
    }

    pub async fn fts_search_entities(
        &self,
        world_id: &str,
        query_text: &str,
    ) -> Result<Vec<Entity>, InfrastructureError> {
        let cleaned: String = query_text
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { ' ' })
            .collect();

        if cleaned.trim().is_empty() {
            return Ok(Vec::new());
        }

        let results = self
            .fetch_all::<Entity>(
                query(
                    r#"
                    CALL text_search.search_all("entity_text", $text) YIELD node, score
                    WITH node, score
                    WHERE node.world_id = $world_id
                    RETURN node
                    ORDER BY score DESC
                    LIMIT 10
                "#,
                )
                .param("world_id", world_id)
                .param("text", cleaned),
                "node",
            )
            .await?;
        Ok(results)
    }

    pub async fn delete_relationships_by_endpoints(
        &self,
        world_id: &str,
        from_id: &str,
        to_id: &str,
        rel_type: &str,
    ) -> Result<(), InfrastructureError> {
        Self::validate_cypher_identifier(rel_type)?;

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

    pub async fn delete_graph(&self, world_id: &str) -> Result<(), InfrastructureError> {
        Ok(self
            .graph
            .run(
                query(
                    "
                    MATCH (e { world_id: $world_id })
                    DETACH DELETE e
                ",
                )
                .param("world_id", world_id),
            )
            .await?)
    }
}
