use sqlx::FromRow;
use serde::{Deserialize};
use chrono::prelude::*;
use uuid::Uuid;

#[derive(FromRow, Debug, Clone, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            created_at: value.created_at,
            updated_at: value.updated_at
        }
    }
}