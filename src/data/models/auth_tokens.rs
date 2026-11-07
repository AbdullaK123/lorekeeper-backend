use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx;
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum AuthTokenPurpose {
    PasswordReset,
    EmailVerification,
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct AuthTokenRow {
    pub token_hash: String,
    pub user_id: Uuid,
    pub purpose: AuthTokenPurpose,
    pub created_at: DateTime<Utc>,
    pub expired_at: DateTime<Utc>,
}
