use crate::data::{
    ConversationItemsResponse, ConversationRepository, ConversationResponse, Message,
    WorldRepository,
};
use crate::service::{
    ChatAgentFactory, ResearchInput, RetrievalService, ServiceError, format_hits,
};
use chrono::Utc;
use futures::{Stream, StreamExt};
use rig::agent::MultiTurnStreamItem;
use uuid::Uuid;

#[derive(Clone)]
pub struct ConversationService {
    pub conversation_repo: ConversationRepository,
    pub world_repo: WorldRepository,
    retrieval_service: RetrievalService,
    agent_factory: ChatAgentFactory,
    research_job: dbos::WorkflowRef<ResearchInput, String, ServiceError>,
}

impl ConversationService {
    pub fn new(
        conversation_repo: ConversationRepository,
        world_repo: WorldRepository,
        retrieval_service: RetrievalService,
        agent_factory: ChatAgentFactory,
        research_job: dbos::WorkflowRef<ResearchInput, String, ServiceError>,
    ) -> Self {
        Self {
            conversation_repo,
            world_repo,
            retrieval_service,
            agent_factory,
            research_job,
        }
    }

    // create a conversation thread
    pub async fn create_conversation(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        title: String,
    ) -> Result<ConversationResponse, ServiceError> {
        let world = self.world_repo.get_by_id(world_id, user_id).await?;

        if world.is_none() {
            return Err(ServiceError::NotFound("World not found".to_string()));
        }

        let result = self
            .conversation_repo
            .create(world_id, user_id, title)
            .await?;
        Ok(result.into())
    }

    // list conversations for a world id
    pub async fn get_conversations(
        &self,
        world_id: Uuid,
        user_id: Uuid,
    ) -> Result<ConversationItemsResponse, ServiceError> {
        let result = self
            .conversation_repo
            .get_by_world_id(world_id, user_id)
            .await?;
        Ok(result.into())
    }

    // get a convo by id
    pub async fn get_conversation(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<ConversationResponse, ServiceError> {
        let result = self.conversation_repo.get_by_id(id, user_id).await?;
        match result {
            Some(conversation) => Ok(conversation.into()),
            None => Err(ServiceError::NotFound("Conversation not found".to_string())),
        }
    }

    // append messages to a convo
    pub async fn append_messages(
        &self,
        id: Uuid,
        user_id: Uuid,
        messages: &[Message],
    ) -> Result<ConversationResponse, ServiceError> {
        let result = self
            .conversation_repo
            .append_messages(id, user_id, messages)
            .await?;
        match result {
            Some(conversation) => Ok(conversation.into()),
            None => Err(ServiceError::NotFound("Conversation not found".to_string())),
        }
    }

    pub async fn chat_turn(
        &self,
        id: Uuid,
        user_id: Uuid,
        content: String,
    ) -> Result<
        impl Stream<Item = Result<MultiTurnStreamItem, ServiceError>> + Send + 'static + use<>,
        ServiceError,
    > {
        let conversation = self.get_conversation(id, user_id).await?;
        let hits = self
            .retrieval_service
            .search(&content, &conversation.world_id.to_string(), 60.0)
            .await?;
        let lore = format_hits(&hits);
        let message = Message::User {
            id: Uuid::new_v4(),
            content: content.clone(),
            created_at: Utc::now(),
        };
        let history = conversation.rig_history();
        // The conversation as it was before this turn: researcher context for resolving "his", "the city", ...
        let earlier_messages: Vec<Message> = conversation.messages.0.clone();
        let agent = self
            .agent_factory
            .create_agent(&conversation.world_id.to_string(), lore);

        let service = self.clone();

        let events = async_stream::stream! {
            let mut rig_stream = agent
                .prompt(content.clone())
                .history(history)
                .max_turns(5)
                .stream();
            while let Some(item) = rig_stream.next().await {

                let message = message.clone();

                match item {
                    Ok(item) => {
                        if let MultiTurnStreamItem::FinalResponse(resp) = &item {
                            let reply = Message::Assistant {
                                id: Uuid::new_v4(),
                                content: Some(resp.output().to_string()),
                                tool_calls: None,
                                created_at: Utc::now()
                            };
                            let svc = &service;
                            let messages = vec![message.clone(), reply];
                            if let Err(e) = svc.append_messages(id, user_id, &messages).await {
                                tracing::error!("saving chat turn failed: {e}");
                                yield Err(e.into());
                                break;
                            }

                            let input = ResearchInput {
                                world_id: conversation.world_id,
                                conversation_id: id,
                                source_message_id: message.id(),
                                message: content.clone(),
                                history: earlier_messages.clone()
                            };

                            let workflow_id = message.id().to_string();

                            if let Err(error) = service.research_job
                                .start_with(input, dbos::StartOptions {
                                    workflow_id: Some(&workflow_id),
                                    ..Default::default()
                                })
                                .await
                            {
                                yield Err(ServiceError::InternalError(error.to_string()));
                                break;
                            }

                        }
                        yield Ok(item)
                    },
                    Err(e) => {
                        yield Err(e.into());
                        break;
                    }
                }
            }
        };
        Ok(events)
    }

    // delete a convo
    pub async fn delete_conversation(&self, id: Uuid, user_id: Uuid) -> Result<bool, ServiceError> {
        let result = self.conversation_repo.delete(id, user_id).await?;
        Ok(result)
    }
}
