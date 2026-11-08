use crate::infrastructure::InfrastructureError;
use neo4rs::{Graph, query};

#[derive(Clone)]
pub(super) struct GraphMaintenanceRepository {
    graph: Graph,
}

impl GraphMaintenanceRepository {
    pub(super) fn new(graph: Graph) -> Self {
        Self { graph }
    }

    /// Idempotent: Memgraph skips a constraint that already exists. Run once at startup.
    #[tracing::instrument(
        name = "repository.graph.ensure_constraints",
        level = "debug",
        skip_all
    )]
    pub(super) async fn ensure_constraints(&self) -> Result<(), InfrastructureError> {
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

    #[tracing::instrument(name = "repository.graph.delete_graph", level = "debug", skip_all)]
    pub(super) async fn delete_graph(&self, world_id: &str) -> Result<(), InfrastructureError> {
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
