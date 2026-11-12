use sqlx::PgPool;
use uuid::Uuid;

use crate::{data::LoginAttemptRow, infrastructure::InfrastructureError};

#[derive(Clone, Debug)]
pub struct LoginAttemptsRepository {
    pool: PgPool,
}

impl LoginAttemptsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(name = "repository.login_attempt.create", level = "debug", skip_all)]
    pub async fn create(
        &self,
        state_hash: String,
        nonce: String,
        pkce_verifier: String,
        user_id: Option<Uuid>,
    ) -> Result<LoginAttemptRow, InfrastructureError> {
        let result = sqlx::query_as!(
            LoginAttemptRow,
            "
            INSERT INTO google_login_attempts (state_hash, nonce, pkce_verifier, user_id) 
            VALUES ($1, $2, $3, $4)
            RETURNING *
            ",
            state_hash,
            nonce,
            pkce_verifier,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            tracing::error!(operation = "insert", "OAuth login attempt query failed");
            error
        })?;
        tracing::debug!("OAuth login attempt insert completed");
        Ok(result)
    }

    #[tracing::instrument(name = "repository.login_attempt.consume", level = "debug", skip_all)]
    pub async fn consume(
        &self,
        state_hash: String,
        user_id: Option<Uuid>,
    ) -> Result<Option<LoginAttemptRow>, InfrastructureError> {
        let result = sqlx::query_as!(
            LoginAttemptRow,
            "
            DELETE FROM google_login_attempts
            WHERE state_hash=$1 
                AND (user_id IS NULL OR user_id=$2::uuid) 
                AND expires_at > NOW()
            RETURNING *
            ",
            state_hash,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            tracing::error!(operation = "consume", "OAuth login attempt query failed");
            error
        })?;
        tracing::debug!(
            matched = result.is_some(),
            "OAuth login attempt consume completed"
        );
        Ok(result)
    }
}
