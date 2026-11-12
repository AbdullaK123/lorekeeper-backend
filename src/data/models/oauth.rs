use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Deserialize, FromRow)]
pub struct LoginAttemptRow {
    pub state_hash: String,
    pub nonce: String,
    pub pkce_verifier: String,
    pub expires_at: DateTime<Utc>,
    pub user_id: Option<Uuid>,
}
