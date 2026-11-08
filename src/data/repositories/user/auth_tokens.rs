use chrono::{Duration, Utc};
use nanoid;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    data::{AuthTokenPurpose, AuthTokenRow},
    infrastructure::InfrastructureError,
};

#[derive(Clone, Debug)]
pub(super) struct AuthTokenRepository {
    pool: PgPool,
}

impl AuthTokenRepository {
    pub(super) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(name = "repository.auth_token.upsert", level = "debug", skip_all, fields(user_id = %user_id))]
    pub(super) async fn upsert_token(
        &self,
        user_id: Uuid,
        purpose: AuthTokenPurpose,
    ) -> Result<(AuthTokenRow, String), InfrastructureError> {
        let token = nanoid::nanoid!();

        let token_hash = hex::encode(Sha256::digest(token.as_bytes()));

        let expires_at = match purpose {
            AuthTokenPurpose::EmailVerification => Utc::now() + Duration::hours(24),
            AuthTokenPurpose::PasswordReset => Utc::now() + Duration::minutes(30),
        };

        let result = sqlx::query_as!(
            AuthTokenRow,
            r#"
            INSERT INTO auth_tokens (token_hash, user_id, purpose, expired_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, purpose)
            DO UPDATE SET
                token_hash = EXCLUDED.token_hash,
                created_at = EXCLUDED.created_at,
                expired_at = EXCLUDED.expired_at
            RETURNING 
                token_hash, user_id,
                purpose AS "purpose: AuthTokenPurpose",
                created_at, expired_at
            "#,
            token_hash,
            user_id,
            purpose as _,
            expires_at
        )
        .fetch_one(&self.pool)
        .await?;

        tracing::debug!("auth token stored");
        Ok((result, token))
    }

    #[tracing::instrument(name = "repository.auth_token.consume", level = "debug", skip_all, fields(user_id = %user_id))]
    pub(super) async fn consume_token(
        &self,
        user_id: Uuid,
        purpose: AuthTokenPurpose,
        token_hash: String,
        connection: &mut sqlx::PgConnection,
    ) -> Result<Option<AuthTokenRow>, InfrastructureError> {
        let result = sqlx::query_as!(
            AuthTokenRow,
            r#"
            DELETE FROM auth_tokens
            WHERE user_id=$1 
                AND purpose=$2
                AND token_hash=$3
                AND expired_at > NOW()
            RETURNING 
                token_hash, user_id,
                purpose AS "purpose: AuthTokenPurpose",
                created_at, expired_at
            "#,
            user_id,
            purpose as _,
            token_hash
        )
        .fetch_optional(connection)
        .await?;
        tracing::debug!(
            matched = result.is_some(),
            "auth token consume query completed; caller owns commit"
        );
        Ok(result)
    }
}
