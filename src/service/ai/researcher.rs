use crate::{
    data::{GraphRepository, Message},
    service::{FlagConflict, ReadEntity, RetrievalService, SearchWorld, ServiceError, UpdateGraph},
};
use rig::{
    Agent, AgentBuilder, DynModel,
    operation::{Completion, Embedding},
};
use uuid::Uuid;

/// Writes lore after the conversation turn is saved. Chat agents remain read-only.
#[derive(Clone)]
pub struct ResearcherService {
    model: DynModel<Completion>,
    preamble: String,
    graph_repo: GraphRepository,
    retrieval_service: RetrievalService,
    embedding_model: DynModel<Embedding>,
}

impl ResearcherService {
    pub fn new(
        model: DynModel<Completion>,
        preamble: String,
        graph_repo: GraphRepository,
        retrieval_service: RetrievalService,
        embedding_model: DynModel<Embedding>,
    ) -> Self {
        Self {
            model,
            preamble,
            graph_repo,
            retrieval_service,
            embedding_model,
        }
    }

    pub(crate) fn create_agent(
        &self,
        world_id: String,
        conversation_id: String,
        source_message_id: String,
    ) -> Agent {
        AgentBuilder::new(self.model.clone())
            .preamble(&self.preamble)
            .tool(SearchWorld {
                world_id: world_id.clone(),
                retrieval_service: self.retrieval_service.clone(),
            })
            .tool(ReadEntity {
                world_id: world_id.clone(),
                graph_repo: self.graph_repo.clone(),
            })
            .tool(FlagConflict {
                world_id: world_id.clone(),
                conversation_id: conversation_id.clone(),
                source_message_id: source_message_id.clone(),
                graph_repo: self.graph_repo.clone(),
            })
            .tool(UpdateGraph {
                world_id: world_id.clone(),
                conversation_id: conversation_id.clone(),
                source_message_id: source_message_id.clone(),
                graph_repo: self.graph_repo.clone(),
                embedding_model: self.embedding_model.clone(),
            })
            .build()
    }

    pub async fn process_message(
        &self,
        world_id: Uuid,
        conversation_id: Uuid,
        source_message_id: Uuid,
        message: &str,
        history: &[Message],
    ) -> Result<String, ServiceError> {
        let world_id = world_id.to_string();
        let conversation_id = conversation_id.to_string();
        let source_message_id = source_message_id.to_string();
        let agent = self.create_agent(
            world_id.clone(),
            conversation_id.clone(),
            source_message_id.clone(),
        );
        let input = serde_json::json!({
            "world_id": world_id, "message": message, "recent_messages": history,
            "source_conversation": conversation_id,
        });
        let response = agent.prompt(input.to_string()).max_turns(12).await?;
        Ok(response.output().to_string())
    }
}
