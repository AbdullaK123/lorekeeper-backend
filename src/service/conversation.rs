use crate::data::{
    ConversationItemsResponse, ConversationRepository, ConversationResponse, Message,
    WorldRepository,
};
use crate::service::{
    ChatAgentFactory, ExtractionService, RetrievalService, ServiceError, format_hits,
};
use chrono::Utc;
use futures::{Stream, StreamExt};
use rig::agent::MultiTurnStreamItem;
use rig::message::Message as RigMessage;
use uuid::Uuid;

#[derive(Clone)]
pub struct ConversationService {
    pub conversation_repo: ConversationRepository,
    pub world_repo: WorldRepository,
    extraction_service: ExtractionService,
    retrieval_service: RetrievalService,
    agent_factory: ChatAgentFactory,
}

impl ConversationService {
    pub fn new(
        conversation_repo: ConversationRepository,
        world_repo: WorldRepository,
        extraction_service: ExtractionService,
        retrieval_service: RetrievalService,
        agent_factory: ChatAgentFactory,
    ) -> Self {
        Self {
            conversation_repo,
            world_repo,
            extraction_service,
            retrieval_service,
            agent_factory,
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
        Ok(ConversationItemsResponse {
            conversations: result.into_iter().map(|c| c.into()).collect(),
        })
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
            content: content.clone(),
            created_at: Utc::now(),
        };
        let history: Vec<RigMessage> = conversation
            .messages
            .iter()
            .filter_map(Message::to_rig)
            .collect();
        // The conversation as it was before this turn: extraction's context for resolving "his", "the city", ...
        let earlier_messages: Vec<Message> = conversation.messages.0.clone();
        let agent = self
            .agent_factory
            .create_agent(conversation.world_id.to_string(), lore);

        let service = self.clone();
        let events = async_stream::stream! {
            let mut rig_stream = agent
                .prompt(content.clone())
                .history(history)
                .max_turns(5)
                .stream();
            while let Some(item) = rig_stream.next().await {
                match item {
                    Ok(item) => {
                        if let MultiTurnStreamItem::FinalResponse(resp) = &item {
                            let reply = Message::Assistant {
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
                            let extraction = service.extraction_service.clone();
                            let world_id = conversation.world_id;
                            let text = content.clone();
                            let earlier = earlier_messages.clone();
                            tokio::spawn(async move {
                                if let Err(e) = extraction.process_message(world_id, id, &text, &earlier).await {
                                    tracing::error!("extraction failed for conversation {id}: {e}");
                                }
                            });
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
