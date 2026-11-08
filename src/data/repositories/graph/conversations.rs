use super::helpers::fetch_one;
use crate::data::models::Conversation;
use crate::infrastructure::InfrastructureError;
use neo4rs::{Graph, query};

#[derive(Clone)]
pub(super) struct GraphConversationRepository {
    graph: Graph,
}

impl GraphConversationRepository {
    pub(super) fn new(graph: Graph) -> Self {
        Self { graph }
    }

    // --- Conversations ---

    #[tracing::instrument(
        name = "repository.graph.create_conversation",
        level = "debug",
        skip_all
    )]
    pub(super) async fn create_conversation(
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

    #[tracing::instrument(
        name = "repository.graph.update_conversation",
        level = "debug",
        skip_all
    )]
    pub(super) async fn update_conversation(
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

    #[tracing::instrument(
        name = "repository.graph.delete_conversation",
        level = "debug",
        skip_all
    )]
    pub(super) async fn delete_conversation(
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

    #[tracing::instrument(name = "repository.graph.get_conversation", level = "debug", skip_all)]
    pub(super) async fn get_conversation(
        &self,
        id: &str,
        world_id: &str,
    ) -> Result<Option<Conversation>, InfrastructureError> {
        fetch_one::<Conversation>(
            &self.graph,
            query("MATCH (c:Conversation {id: $id, world_id: $world_id}) RETURN c")
                .param("id", id)
                .param("world_id", world_id),
            "c",
        )
        .await
    }
}
