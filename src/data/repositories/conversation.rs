use crate::app::world;
use crate::data::Message;
use crate::data::{ConversationRow, MessageRow};
use crate::infrastructure::InfrastructureError;
use serde_json;
use sqlx::PgPool;
use sqlx::types::Json;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ConversationRepository {
    pool: PgPool,
}

impl ConversationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(name = "repository.conversation.create", level = "debug", skip_all)]
    pub async fn create(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        title: String,
    ) -> Result<ConversationRow, InfrastructureError> {
        let result = sqlx::query_as!(
            ConversationRow,
            r#"
            INSERT INTO conversations (world_id, user_id, title, messages)
            VALUES ($1, $2, $3, '[]'::jsonb)
            RETURNING
                id, world_id,
                user_id, title,
                messages as "messages: Json<Vec<Message>>",
                created_at, updated_at
            "#,
            world_id,
            user_id,
            title
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }

    #[tracing::instrument(
        name = "repository.conversation.update_title",
        level = "debug",
        skip_all
    )]
    pub async fn update_title(
        &self,
        id: Uuid,
        user_id: Uuid,
        new_title: String,
    ) -> Result<Option<ConversationRow>, InfrastructureError> {
        let result = sqlx::query_as!(
            ConversationRow,
            r#"
            UPDATE conversations
            SET title = $1
            WHERE id=$2 AND user_id=$3
            RETURNING
                id, world_id,
                user_id, title,
                messages as "messages: Json<Vec<Message>>",
                created_at, updated_at
            "#,
            new_title,
            id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    #[tracing::instrument(
        name = "repository.conversation.append_messages",
        level = "debug",
        skip_all
    )]
    pub async fn append_messages(
        &self,
        id: Uuid,
        user_id: Uuid,
        messages: &[Message],
    ) -> Result<Option<ConversationRow>, InfrastructureError> {
        let message_json = serde_json::json!(messages);
        let result = sqlx::query_as!(
            ConversationRow,
            r#"
            UPDATE conversations
            SET
                messages = messages || $1::jsonb,
                updated_at = NOW()
            WHERE id=$2 AND user_id=$3
            RETURNING
                id, world_id,
                user_id, title,
                messages as "messages: Json<Vec<Message>>",
                created_at, updated_at
            "#,
            message_json,
            id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    #[tracing::instrument(name = "repository.conversation.get_by_id", level = "debug", skip_all)]
    pub async fn get_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<ConversationRow>, InfrastructureError> {
        let result = sqlx::query_as!(
            ConversationRow,
            r#"
            SELECT
                id,
                world_id,
                user_id,
                title,
                messages as "messages: Json<Vec<Message>>",
                created_at,
                updated_at
            FROM conversations
            WHERE id=$1 AND user_id=$2
            "#,
            id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    #[tracing::instrument(
        name = "repository.conversation.get_by_world_id",
        level = "debug",
        skip_all
    )]
    pub async fn get_by_world_id(
        &self,
        world_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<ConversationRow>, InfrastructureError> {
        let result = sqlx::query_as!(
            ConversationRow,
            r#"
            SELECT
                id,
                world_id,
                user_id,
                title,
                messages as "messages: Json<Vec<Message>>",
                created_at,
                updated_at
            FROM conversations
            WHERE world_id=$1 AND user_id=$2
            "#,
            world_id,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }

    #[tracing::instrument(name = "repository.conversation.delete", level = "debug", skip_all)]
    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool, InfrastructureError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM conversations WHERE id=$1 AND user_id=$2
            "#,
            id,
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    #[tracing::instrument(
        name = "repository.conversation.get_message",
        level = "debug",
        skip_all
    )]
    pub async fn get_message(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        message_id: &str,
    ) -> Result<Option<MessageRow>, InfrastructureError> {
        let result = sqlx::query_as!(
            MessageRow,
            r#"
            SELECT c.world_id, c.id AS conversation_id,
                   item.message AS "message!: Json<Message>"
            FROM conversations c
            CROSS JOIN LATERAL jsonb_array_elements(c.messages) AS item(message)
            WHERE c.world_id = $1
            AND c.user_id = $2
            AND item.message->>'id' = $3::text
            LIMIT 1
            "#,
            world_id,
            user_id,
            message_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }
}
