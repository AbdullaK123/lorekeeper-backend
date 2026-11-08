mod auth_tokens;
mod sessions;

#[cfg(test)]
mod tests;

use crate::data::models::User;
use crate::data::{AuthTokenPurpose, AuthTokenRow, SessionRow};
use crate::infrastructure::InfrastructureError;
use auth_tokens::AuthTokenRepository;
use sessions::SessionRepository;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct UserRepository {
    pool: PgPool,
    auth_tokens: AuthTokenRepository,
    sessions: SessionRepository,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            auth_tokens: AuthTokenRepository::new(pool.clone()),
            sessions: SessionRepository::new(pool.clone()),
            pool,
        }
    }

    #[tracing::instrument(name = "repository.user.sign_up", level = "debug", skip_all)]
    pub async fn sign_up(
        &self,
        username: String,
        email: String,
        password_hash: String,
    ) -> Result<User, InfrastructureError> {
        let result = sqlx::query_as!(
            User,
            "
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING *
            ",
            username,
            email,
            password_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            if error.as_database_error().is_some_and(|error| {
                error.is_unique_violation() && error.constraint() == Some("users_email_key")
            }) {
                InfrastructureError::Conflict("An account with this email already exists".into())
            } else {
                error.into()
            }
        })?;

        Ok(result)
    }

    #[tracing::instrument(name = "repository.user.get_user_by_email", level = "debug", skip_all)]
    pub async fn get_user_by_email(
        &self,
        email: String,
    ) -> Result<Option<User>, InfrastructureError> {
        let result = sqlx::query_as!(
            User,
            "
            SELECT *
            FROM users
            WHERE email = $1
            ",
            email
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    #[tracing::instrument(name = "repository.user.get_user_by_id", level = "debug", skip_all)]
    pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, InfrastructureError> {
        let result = sqlx::query_as!(
            User,
            "
            SELECT  *
            FROM users
            WHERE id = $1
            ",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    #[tracing::instrument(name = "repository.user.update_password", level = "debug", skip_all, fields(user_id = %id))]
    async fn update_password(
        &self,
        id: Uuid,
        new_password_hash: String,
        connection: &mut sqlx::PgConnection,
    ) -> Result<Option<User>, InfrastructureError> {
        Ok(sqlx::query_as!(
            User,
            "
            UPDATE users
            SET password_hash = $2
            WHERE id = $1
            RETURNING *
            ",
            id,
            new_password_hash
        )
        .fetch_optional(connection)
        .await?)
    }

    #[tracing::instrument(name = "repository.user.verify_user", level = "debug", skip_all, fields(user_id = %id))]
    async fn verify_user(
        &self,
        id: Uuid,
        connection: &mut sqlx::PgConnection,
    ) -> Result<Option<User>, InfrastructureError> {
        Ok(sqlx::query_as!(
            User,
            "
            UPDATE users
            SET verified=TRUE
            WHERE id=$1
            RETURNING *
            ",
            id
        )
        .fetch_optional(connection)
        .await?)
    }

    #[tracing::instrument(name = "repository.user.reset_password_with_token", level = "info", skip_all, fields(user_id = %id))]
    pub async fn reset_password_with_token(
        &self,
        id: Uuid,
        token_hash: String,
        new_password_hash: String,
    ) -> Result<Option<User>, InfrastructureError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            tracing::error!(step = "begin", "auth transaction failed");
            InfrastructureError::Database(e)
        })?;

        let result = async {
            self.auth_tokens
                .consume_token(id, AuthTokenPurpose::PasswordReset, token_hash, &mut *tx)
                .await?
                .ok_or_else(|| {
                    tracing::info!("password reset token rejected");
                    InfrastructureError::InvalidInput("Invalid or expired reset token".into())
                })?;

            let user = self
                .update_password(id, new_password_hash, &mut *tx)
                .await?;

            if user.is_none() {
                return Ok(None);
            } else {
                self.sessions
                    .delete_all(&user.as_ref().unwrap().id, &mut *tx)
                    .await?;
                return Ok(user);
            }
        }
        .await;

        match result {
            Ok(Some(user)) => {
                tx.commit().await.map_err(|e| {
                    tracing::error!(step = "commit", "auth transaction failed");
                    InfrastructureError::Database(e)
                })?;
                tracing::info!("password reset committed");
                Ok(Some(user))
            }
            Ok(None) => {
                tracing::warn!(reason = "user_not_found", "rolling back auth transaction");
                tx.rollback().await.map_err(|e| {
                    tracing::error!(step = "rollback", "auth transaction failed");
                    InfrastructureError::Database(e)
                })?;
                tracing::debug!("auth transaction rolled back");
                Ok(None)
            }
            Err(e) => {
                tracing::warn!("rolling back auth transaction after operation failure");
                tx.rollback().await.map_err(|e| {
                    tracing::error!(step = "rollback", "auth transaction failed");
                    InfrastructureError::Database(e)
                })?;
                tracing::debug!("auth transaction rolled back");
                Err(e)
            }
        }
    }

    #[tracing::instrument(name = "repository.user.verify_email_with_token", level = "info", skip_all, fields(user_id = %id))]
    pub async fn verify_email_with_token(
        &self,
        id: Uuid,
        token_hash: String,
    ) -> Result<Option<User>, InfrastructureError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            tracing::error!(step = "begin", "auth transaction failed");
            InfrastructureError::Database(e)
        })?;

        let result = async {
            self.auth_tokens
                .consume_token(
                    id,
                    AuthTokenPurpose::EmailVerification,
                    token_hash,
                    &mut *tx,
                )
                .await?
                .ok_or_else(|| {
                    tracing::info!("email verification token rejected");
                    InfrastructureError::InvalidInput(
                        "Invalid or expired verification token".into(),
                    )
                })?;

            self.verify_user(id, &mut *tx).await
        }
        .await;

        match result {
            Ok(Some(user)) => {
                tx.commit().await.map_err(|e| {
                    tracing::error!(step = "commit", "auth transaction failed");
                    InfrastructureError::Database(e)
                })?;
                tracing::info!("email verification committed");
                Ok(Some(user))
            }
            Ok(None) => {
                tracing::warn!(reason = "user_not_found", "rolling back auth transaction");
                tx.rollback().await.map_err(|e| {
                    tracing::error!(step = "rollback", "auth transaction failed");
                    InfrastructureError::Database(e)
                })?;
                tracing::debug!("auth transaction rolled back");
                Ok(None)
            }
            Err(e) => {
                tracing::warn!("rolling back auth transaction after operation failure");
                tx.rollback().await.map_err(|e| {
                    tracing::error!(step = "rollback", "auth transaction failed");
                    InfrastructureError::Database(e)
                })?;
                tracing::debug!("auth transaction rolled back");
                Err(e)
            }
        }
    }

    #[tracing::instrument(name = "repository.user.issue_auth_token", level = "debug", skip_all, fields(user_id = %user_id))]
    pub async fn issue_auth_token(
        &self,
        user_id: Uuid,
        purpose: AuthTokenPurpose,
    ) -> Result<(AuthTokenRow, String), InfrastructureError> {
        self.auth_tokens.upsert_token(user_id, purpose).await
    }

    #[tracing::instrument(name = "repository.user.get_session", level = "debug", skip_all)]
    pub async fn get_session(
        &self,
        token_hash: &str,
    ) -> Result<Option<SessionRow>, InfrastructureError> {
        Ok(self.sessions.get(token_hash).await?)
    }

    #[tracing::instrument(name = "repository.user.create_session", level = "debug", skip_all, fields(user_id = %user_id))]
    pub async fn create_session(
        &self,
        token_hash: &str,
        user_id: &Uuid,
    ) -> Result<SessionRow, InfrastructureError> {
        Ok(self.sessions.create(token_hash, user_id).await?)
    }

    #[tracing::instrument(name = "repository.user.delete_session", level = "debug", skip_all, fields(user_id = %user_id))]
    pub async fn delete_session(
        &self,
        token_hash: &str,
        user_id: &Uuid,
    ) -> Result<bool, InfrastructureError> {
        Ok(self.sessions.delete(token_hash, user_id).await?)
    }
}
