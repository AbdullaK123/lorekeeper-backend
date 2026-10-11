use uuid::Uuid;
use crate::data::{ConversationItemsResponse, ConversationRepository, ConversationResponse, Message};
use crate::service::ServiceError;

#[derive(Clone)]
pub struct ConversationService {
    pub conversation_repo: ConversationRepository
}

impl ConversationService {
    pub fn new(repo: ConversationRepository) -> Self {
        Self {
            conversation_repo: repo
        }
    }

    // create a conversation thread
    pub async fn create_conversation(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        title: String
    ) -> Result<ConversationResponse, ServiceError> {
        let result = self.conversation_repo.create(
            world_id,
            user_id,
            title
        ).await?;
        Ok(result.into())
    }

    // list conversations for a world id
    pub async fn get_conversations(
        &self,
        world_id: Uuid,
        user_id: Uuid
    ) -> Result<ConversationItemsResponse, ServiceError> {
        let result = self.conversation_repo.get_by_world_id(
            world_id,
            user_id
        ).await?;
        Ok(ConversationItemsResponse {
            conversations: result
                .into_iter()
                .map(|c| c.into())
                .collect()
        })
    }

    // get a convo by id
    pub async fn get_conversation(
        &self,
        id: Uuid,
        user_id: Uuid
    ) -> Result<ConversationResponse, ServiceError> {
        let result = self.conversation_repo.get_by_id(
            id,
            user_id
        ).await?;
        match result {
            Some(conversation) => Ok(conversation.into()),
            None => Err(ServiceError::NotFound("Conversation not found".to_string()))
        }
    }

    // append a message to a convo
    pub async fn append_message(
        &self,
        id: Uuid,
        user_id: Uuid,
        message: Message
    ) -> Result<ConversationResponse, ServiceError> {
        let result = self.conversation_repo.append_message(
            id,
            user_id,
            message
        ).await?;
        match result {
            Some(conversation) => Ok(conversation.into()),
            None => Err(ServiceError::NotFound("Conversation not found".to_string()))
        }
    }

    // delete a convo
    pub async fn delete_conversation(
        &self,
        id: Uuid,
        user_id: Uuid
    ) -> Result<bool, ServiceError> {
        let result = self.conversation_repo.delete(id, user_id).await?;
        Ok(result)
    }
}
