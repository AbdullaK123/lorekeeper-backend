use chrono::{DateTime, Utc};
use rig::message::Message as RigMessage;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::Json;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "user")]
    User {
        content: String,
        created_at: DateTime<Utc>,
    },

    #[serde(rename = "assistant")]
    Assistant {
        content: Option<String>,
        tool_calls: Option<Vec<ToolCall>>,
        created_at: DateTime<Utc>,
    },

    #[serde(rename = "tool")]
    Tool {
        tool_call_id: String,
        content: String,
        created_at: DateTime<Utc>,
    },
}

impl Message {
    pub fn to_rig(&self) -> Option<RigMessage> {
        match self {
            Message::User { content, .. } => Some(RigMessage::user(content.clone())),
            Message::Assistant {
                content: Some(text),
                ..
            } => Some(RigMessage::assistant(text.clone())),
            _ => None,
        }
    }
}

#[derive(FromRow, Debug, Clone, Deserialize)]
pub struct ConversationRow {
    pub id: Uuid,
    pub world_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub messages: Json<Vec<Message>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChatRequest {
    #[validate(length(min = 1, max = 10000))]
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationResponse {
    pub id: Uuid,
    pub world_id: Uuid,
    pub title: String,
    pub messages: Json<Vec<Message>>,
    pub updated_at: DateTime<Utc>,
}

impl From<ConversationRow> for ConversationResponse {
    fn from(value: ConversationRow) -> Self {
        Self {
            id: value.id,
            world_id: value.world_id,
            title: value.title,
            messages: value.messages,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationItem {
    pub id: Uuid,
    pub title: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateConversationRequest {
    pub title: String,
}

impl From<ConversationRow> for ConversationItem {
    fn from(value: ConversationRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationItemsResponse {
    pub conversations: Vec<ConversationItem>,
}
