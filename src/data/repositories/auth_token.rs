use chrono::{Duration, Utc};
use nanoid;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    data::{AuthTokenPurpose, AuthTokenRow},
    infrastructure::InfrastructureError,
};

#[derive(Clone)]
pub struct AuthTokenRepository {
    pub pool: PgPool,
}

impl AuthTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_token(
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

        Ok((result, token))
    }

    pub async fn consume_token(
        &self,
        user_id: Uuid,
        purpose: AuthTokenPurpose,
        token_hash: String,
        connection: &mut sqlx::PgConnection,
    ) -> Result<Option<AuthTokenRow>, InfrastructureError> {
        Ok(sqlx::query_as!(
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
        .await?)
    }
}
