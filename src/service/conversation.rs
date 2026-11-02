use crate::data::{
    ConversationItemsResponse, ConversationRepository, ConversationResponse, Message,
    MessageResponse, ResearchStatus, ResearchStatusResponse, WorldRepository,
};
use crate::service::{
    ChatAgentFactory, ResearchInput, RetrievalService, ServiceError, format_hits,
};
use chrono::Utc;
use dbos::sysdb::types::WorkflowStatus;
use futures::{Stream, StreamExt};
use rig::agent::MultiTurnStreamItem;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone)]
pub struct ConversationService {
    pub conversation_repo: ConversationRepository,
    pub world_repo: WorldRepository,
    retrieval_service: RetrievalService,
    agent_factory: ChatAgentFactory,
    research_job: dbos::WorkflowRef<ResearchInput, String, ServiceError>,
    dbos_client: dbos::Client,
}

impl ConversationService {
    pub fn new(
        conversation_repo: ConversationRepository,
        world_repo: WorldRepository,
        retrieval_service: RetrievalService,
        agent_factory: ChatAgentFactory,
        research_job: dbos::WorkflowRef<ResearchInput, String, ServiceError>,
        dbos_client: dbos::Client,
    ) -> Self {
        Self {
            conversation_repo,
            world_repo,
            retrieval_service,
            agent_factory,
            research_job,
            dbos_client,
        }
    }

    // create a conversation thread
    #[tracing::instrument(name = "conversation.create_conversation", level = "info", skip_all, fields(world_id = %world_id, user_id = %user_id))]
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
    #[tracing::instrument(name = "conversation.get_conversations", level = "info", skip_all, fields(world_id = %world_id, user_id = %user_id))]
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
    #[tracing::instrument(name = "conversation.get_conversation", level = "info", skip_all, fields(id = %id, user_id = %user_id))]
    pub async fn get_conversation(
        &self,
        world_id: Uuid,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<ConversationResponse, ServiceError> {
        let result = self.conversation_repo.get_by_id(id, user_id).await?;
        match result {
            Some(conversation) if conversation.world_id == world_id => Ok(conversation.into()),
            _ => Err(ServiceError::NotFound("Conversation not found".to_string())),
        }
    }

    // append messages to a convo
    #[tracing::instrument(name = "conversation.append_messages", level = "info", skip_all, fields(id = %id, user_id = %user_id))]
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

    #[tracing::instrument(name = "conversation.chat_turn", level = "info", skip_all, fields(id = %id, user_id = %user_id))]
    pub async fn chat_turn(
        &self,
        world_id: Uuid,
        id: Uuid,
        user_id: Uuid,
        content: String,
    ) -> Result<
        impl Stream<Item = Result<MultiTurnStreamItem, ServiceError>> + Send + 'static + use<>,
        ServiceError,
    > {
        let conversation = self.get_conversation(world_id, id, user_id).await?;
        let world_id = conversation.world_id;
        let world_key = world_id.to_string();
        let hits = self
            .retrieval_service
            .search(&content, &world_key, 60.0)
            .await?;
        let lore = format_hits(&hits);
        let message = Message::User {
            id: Uuid::new_v4(),
            content: content.clone(),
            created_at: Utc::now(),
        };
        let history = conversation.rig_history();
        // The conversation as it was before this turn: researcher context for resolving "his", "the city", ...
        let earlier_messages = conversation.messages.0;
        let agent = self.agent_factory.create_agent(&world_key, lore);

        let service = self.clone();

        let events = async_stream::stream! {
            let started = std::time::Instant::now();
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
                                id: Uuid::new_v4(),
                                content: Some(resp.output().to_string()),
                                tool_calls: None,
                                created_at: Utc::now()
                            };
                            let source_message_id = message.id();
                            let messages = [message, reply];
                            if let Err(e) = service.append_messages(id, user_id, &messages).await {
                                tracing::error!(conversation_id = %id, message_id = %source_message_id, "saving chat turn failed");
                                yield Err(e.into());
                                break;
                            }

                            tracing::info!(conversation_id = %id, message_id = %source_message_id, "chat turn saved");

                            let input = ResearchInput {
                                world_id,
                                conversation_id: id,
                                source_message_id,
                                message: content,
                                history: earlier_messages
                            };

                            let workflow_id = source_message_id.to_string();

                            if let Err(error) = service.research_job
                                .start_with(input, dbos::StartOptions {
                                    workflow_id: Some(&workflow_id),
                                    ..Default::default()
                                })
                                .await
                            {
                                tracing::error!(conversation_id = %id, workflow_id = %workflow_id, "research workflow submission failed");
                                yield Err(ServiceError::InternalError(error.to_string()));
                                break;
                            }
                            tracing::info!(conversation_id = %id, workflow_id = %workflow_id,
                                elapsed_ms = started.elapsed().as_millis() as u64, "research workflow submitted");
                            yield Ok(item);
                            break;
                        }
                        yield Ok(item)
                    },
                    Err(e) => {
                        tracing::error!(conversation_id = %id, "chat model stream failed");
                        yield Err(e.into());
                        break;
                    }
                }
            }
        };
        Ok(events)
    }

    // delete a convo
    #[tracing::instrument(name = "conversation.delete_conversation", level = "info", skip_all, fields(id = %id, user_id = %user_id))]
    pub async fn delete_conversation(
        &self,
        world_id: Uuid,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, ServiceError> {
        self.get_conversation(world_id, id, user_id).await?;
        let result = self.conversation_repo.delete(id, user_id).await?;
        Ok(result)
    }

    #[tracing::instrument(name = "conversation.get_message", level = "info", skip_all, fields(world_id = %world_id, user_id = %user_id, message_id = %message_id))]
    pub async fn get_message(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        message_id: Uuid,
    ) -> Result<MessageResponse, ServiceError> {
        let result = self
            .conversation_repo
            .get_message(world_id, user_id, &message_id.to_string())
            .await?;
        match result {
            Some(row) => Ok(row.into()),
            None => Err(ServiceError::NotFound("Message not found".to_string())),
        }
    }

    #[tracing::instrument(name = "conversation.get_researcher_status", level = "debug", skip_all, fields(world_id = %world_id, user_id = %user_id, message_id = %message_id))]
    pub async fn get_researcher_status(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        message_id: Uuid,
    ) -> Result<ResearchStatusResponse, ServiceError> {
        self.get_message(world_id, user_id, message_id).await?;
        let workflow_id = self.current_research_workflow(message_id).await?;
        let status = self.dbos_client.workflow_status(&workflow_id).await?;
        match status {
            Some(status) => {
                tracing::debug!(workflow_status = %status.as_str(), "research status loaded");
                Ok(ResearchStatusResponse {
                    status: ResearchStatus::from_workflow_status(status),
                })
            }
            None => {
                tracing::debug!("message has no submitted research workflow");
                Ok(ResearchStatusResponse {
                    status: ResearchStatus::NotSubmitted,
                })
            }
        }
    }

    async fn current_research_workflow(&self, message_id: Uuid) -> Result<String, ServiceError> {
        Ok(self
            .conversation_repo
            .get_research_workflow(message_id)
            .await?
            .unwrap_or_else(|| message_id.to_string()))
    }

    #[tracing::instrument(name = "conversation.retry_research", skip_all, fields(%world_id, %user_id, %message_id))]
    pub async fn retry_research_run(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        message_id: Uuid,
    ) -> Result<ResearchStatusResponse, ServiceError> {
        let saved = self.get_message(world_id, user_id, message_id).await?;
        let content = match saved.message {
            Message::User { content, .. } => content,
            _ => {
                return Err(ServiceError::InvalidInput(
                    "Research requires a user message".into(),
                ));
            }
        };
        // Serialize retry requests for this message across application instances.
        let lock = self
            .conversation_repo
            .lock_research_retry(message_id)
            .await?;
        self.get_message(world_id, user_id, message_id).await?;
        let workflow_id = self.current_research_workflow(message_id).await?;

        let status = self.dbos_client.workflow_status(&workflow_id).await?;
        let active_id = match status {
            Some(WorkflowStatus::Error) => {
                let handle = self
                    .dbos_client
                    .fork::<String, ServiceError>(&workflow_id, dbos::ForkFrom::LastFailure)
                    .await?;
                handle.workflow_id().to_owned()
            }
            Some(WorkflowStatus::MaxRecoveryAttemptsExceeded) => {
                self.dbos_client
                    .resume::<String, ServiceError>(&workflow_id)
                    .await?;
                workflow_id
            }
            None => {
                let history = self
                    .conversation_repo
                    .get_by_id(saved.conversation_id, user_id)
                    .await?
                    .ok_or_else(|| ServiceError::NotFound("Conversation not found".into()))?;
                let source_index = history
                    .messages
                    .0
                    .iter()
                    .position(|message| message.id() == message_id)
                    .ok_or_else(|| ServiceError::NotFound("Message not found".into()))?;
                let input = ResearchInput {
                    world_id,
                    conversation_id: saved.conversation_id,
                    source_message_id: message_id,
                    message: content,
                    history: history.messages.0.into_iter().take(source_index).collect(),
                };
                self.research_job
                    .start_with(
                        input,
                        dbos::StartOptions {
                            workflow_id: Some(&workflow_id),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|error| ServiceError::InternalError(error.to_string()))?;
                workflow_id
            }
            Some(_) => {
                return Err(ServiceError::Conflict(
                    "Research cannot be retried in its current state".into(),
                ));
            }
        };
        self.conversation_repo
            .set_research_workflow(message_id, &active_id)
            .await?;
        lock.commit()
            .await
            .map_err(|error| ServiceError::InternalError(error.to_string()))?;
        tracing::info!(workflow_id = %active_id, "research retry submitted");
        self.get_researcher_status(world_id, user_id, message_id)
            .await
    }
}
