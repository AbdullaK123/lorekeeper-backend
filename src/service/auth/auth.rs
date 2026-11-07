use crate::data::{
    AuthTokenPurpose, AuthTokenRepository, SignupRequest, User, UserRepository, UserResponse,
};
use crate::infrastructure::InfrastructureError;
use crate::service::{EmailService, ServiceError};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tokio;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthService {
    pool: PgPool,
    user_repo: UserRepository,
    auth_token_repo: AuthTokenRepository,
    email_service: EmailService,
    api_base_url: String,
}

impl AuthService {
    pub fn new(
        pool: PgPool,
        user_repo: UserRepository,
        auth_token_repo: AuthTokenRepository,
        resend_api_key: &str,
        api_base_url: &str,
    ) -> Self {
        Self {
            pool,
            user_repo,
            auth_token_repo,
            email_service: EmailService::new(resend_api_key),
            api_base_url: api_base_url.trim_end_matches('/').to_owned(),
        }
    }

    async fn hash_password(password: String) -> Result<String, ServiceError> {
        let hash = tokio::task::spawn_blocking(move || {
            Argon2::default()
                .hash_password(password.as_bytes())
                .map(|h| h.to_string())
        })
        .await
        .map_err(|e| ServiceError::InternalError(e.to_string()))?
        .map_err(|e| ServiceError::InternalError(e.to_string()))?;
        Ok(hash)
    }

    #[tracing::instrument(name = "auth.authenticate_user", level = "info", skip_all)]
    pub async fn authenticate_user(
        &self,
        email: String,
        password: String,
    ) -> Result<User, ServiceError> {
        let user = self.user_repo.get_user_by_email(email).await?;
        if user.is_none() {
            tracing::info!("authentication rejected");
            return Err(ServiceError::AuthError);
        }
        let unwrapped_user = user.unwrap();
        if unwrapped_user.password_hash.is_none() {
            tracing::info!("authentication rejected");
            return Err(ServiceError::AuthError);
        }
        let password_hash = PasswordHash::new(unwrapped_user.password_hash.as_deref().unwrap())
            .map_err(|e| ServiceError::InternalError(e.to_string()))?;
        let password_valid = Argon2::default()
            .verify_password(password.as_bytes(), &password_hash)
            .is_ok();
        if !password_valid {
            tracing::info!("authentication rejected");
            return Err(ServiceError::AuthError);
        }
        tracing::info!(user_id = %unwrapped_user.id, "authentication succeeded");
        Ok(unwrapped_user)
    }

    #[tracing::instrument(name = "auth.signup", level = "info", skip_all)]
    pub async fn signup(&self, payload: SignupRequest) -> Result<UserResponse, ServiceError> {
        let password_hash = Self::hash_password(payload.password.into_inner()).await?;
        let user = self
            .user_repo
            .sign_up(payload.username.into_inner(), payload.email, password_hash)
            .await?;
        tracing::info!(user_id = %user.id, "account created");
        Ok(user.into())
    }

    #[tracing::instrument(name = "auth.get_profile", level = "debug", skip_all, fields(user_id = %user_id))]
    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserResponse, ServiceError> {
        let user = self.user_repo.get_user_by_id(user_id).await?;
        match user {
            Some(user) => Ok(user.into()),
            None => Err(ServiceError::AuthError),
        }
    }

    pub async fn send_password_reset_email(&self, user_id: Uuid) -> Result<(), ServiceError> {
        let user = self.user_repo.get_user_by_id(user_id).await?;
        if user.is_none() {
            return Err(ServiceError::NotFound("User not found".to_string()));
        }
        let (_, token) = self
            .auth_token_repo
            .upsert_token(user_id, AuthTokenPurpose::PasswordReset)
            .await?;

        let reset_url = format!(
            "{}/auth/password_reset?user_id={user_id}&token={}",
            self.api_base_url, token
        );

        self.email_service
            .send_password_reset_email(&user.unwrap().email, &reset_url)
            .await?;

        Ok(())
    }

    pub async fn send_verification_email(&self, user_id: Uuid) -> Result<(), ServiceError> {
        let user = self.user_repo.get_user_by_id(user_id).await?;
        if user.is_none() {
            return Err(ServiceError::NotFound("User not found".to_string()));
        }
        let user = user.unwrap();
        if user.verified {
            return Err(ServiceError::Conflict(
                "User is already verified".to_string(),
            ));
        }
        let (_, token) = self
            .auth_token_repo
            .upsert_token(user_id, AuthTokenPurpose::EmailVerification)
            .await?;
        let verification_url = format!(
            "{}/auth/email_verification?user_id={user_id}&token={}",
            self.api_base_url, token
        );
        self.email_service
            .send_verification_email(&user.email, &verification_url)
            .await?;
        Ok(())
    }

    pub async fn reset_password(
        &self,
        id: Uuid,
        token: String,
        new_password: String,
    ) -> Result<UserResponse, ServiceError> {
        let new_password_hash = Self::hash_password(new_password).await?;
        let token_hash = hex::encode(Sha256::digest(token.as_bytes()));

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::InternalError(e.to_string()))?;

        let result = async {
            self.auth_token_repo
                .consume_token(id, AuthTokenPurpose::PasswordReset, token_hash, &mut *tx)
                .await?
                .ok_or_else(|| {
                    InfrastructureError::InvalidInput("Invalid or expired reset token".into())
                })?;

            let new_user = self
                .user_repo
                .update_password(id, new_password_hash, &mut *tx)
                .await;

            new_user
        }
        .await;

        match result {
            Ok(Some(user)) => {
                tx.commit()
                    .await
                    .map_err(|e| ServiceError::InternalError(e.to_string()))?;
                Ok(user.into())
            }
            Ok(None) => {
                tx.rollback()
                    .await
                    .map_err(|e| ServiceError::InternalError(e.to_string()))?;
                Err(ServiceError::NotFound("User not found".into()))
            }
            Err(e) => {
                tx.rollback()
                    .await
                    .map_err(|e| ServiceError::InternalError(e.to_string()))?;
                return Err(e.into());
            }
        }
    }

    pub async fn verify_user(&self, id: Uuid, token: String) -> Result<UserResponse, ServiceError> {
        let token_hash = hex::encode(Sha256::digest(token.as_bytes()));

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::InternalError(e.to_string()))?;

        let result = async {
            self.auth_token_repo
                .consume_token(
                    id,
                    AuthTokenPurpose::EmailVerification,
                    token_hash,
                    &mut *tx,
                )
                .await?
                .ok_or_else(|| {
                    InfrastructureError::InvalidInput(
                        "Invalid or expired verification token".into(),
                    )
                })?;

            let user = self.user_repo.verify_user(id, &mut *tx).await;
            user
        }
        .await;

        match result {
            Ok(Some(user)) => {
                tx.commit()
                    .await
                    .map_err(|e| ServiceError::InternalError(e.to_string()))?;
                Ok(user.into())
            }
            Ok(None) => {
                tx.rollback()
                    .await
                    .map_err(|e| ServiceError::InternalError(e.to_string()))?;
                Err(ServiceError::NotFound("User not found".to_string()))
            }
            Err(e) => {
                tx.rollback()
                    .await
                    .map_err(|e| ServiceError::InternalError(e.to_string()))?;
                Err(e.into())
            }
        }
    }
}
