use sqlx::PgPool;

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
    ) -> Result<LoginAttemptRow, InfrastructureError> {
        let result = sqlx::query_as!(
            LoginAttemptRow,
            "
            INSERT INTO google_login_attempts (state_hash, nonce, pkce_verifier) 
            VALUES ($1, $2, $3)
            RETURNING *
            ",
            state_hash,
            nonce,
            pkce_verifier
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
    ) -> Result<Option<LoginAttemptRow>, InfrastructureError> {
        let result = sqlx::query_as!(
            LoginAttemptRow,
            "
            DELETE FROM google_login_attempts
            WHERE state_hash=$1 AND expires_at > NOW()
            RETURNING *
            ",
            state_hash
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
