use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::Json;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow, Debug, Clone, Deserialize)]
pub struct ConversationRow {
    pub id: Uuid,
    pub world_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub messages: Json<Vec<Message>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}