use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::FromRow;

#[derive(Clone, Deserialize, FromRow)]
pub struct LoginAttemptRow {
    pub state_hash: String,
    pub nonce: String,
    pub pkce_verifier: String,
    pub expires_at: DateTime<Utc>,
}
