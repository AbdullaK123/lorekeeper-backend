use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow, Debug, Clone, Deserialize)]
pub struct World {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateWorldRequest {
    pub name: String,
    pub description: String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateWorldRequest {
    pub new_name: Option<String>,
    pub new_description: Option<String>
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

impl From<World> for WorldResponse {
    fn from(value: World) -> Self {
        WorldResponse {
            id: value.id,
            name: value.name,
            description: value.description,
            created_at: value.created_at,
            updated_at: value.updated_at
        }
    }
}