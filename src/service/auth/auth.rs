use crate::data::{AuthTokenPurpose, SignupRequest, User, UserRepository, UserResponse};
use crate::service::{EmailService, GoogleClient, ServiceError};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, Utc};
use openidconnect::core::CoreAuthenticationFlow;
use openidconnect::reqwest::Client;
use openidconnect::{AuthorizationCode, PkceCodeVerifier, TokenResponse};
use openidconnect::{CsrfToken, Nonce, PkceCodeChallenge, Scope};
use sha2::{Digest, Sha256};
use tokio;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    email_service: EmailService,
    api_base_url: String,
    oauth_client: GoogleClient,
    http_client: Client,
}

impl AuthService {
    pub fn new(
        user_repo: UserRepository,
        resend_api_key: &str,
        api_base_url: &str,
        client: GoogleClient,
        http_client: Client,
    ) -> Self {
        Self {
            user_repo,
            email_service: EmailService::new(resend_api_key),
            api_base_url: api_base_url.trim_end_matches('/').to_owned(),
            oauth_client: client,
            http_client,
        }
    }

    #[tracing::instrument(name = "auth.hash_password", level = "debug", skip_all)]
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
            .sign_up(
                payload.username.into_inner(),
                payload.email,
                Some(password_hash),
                None,
                false,
            )
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

    #[tracing::instrument(name = "auth.send_password_reset_email", level = "info", skip_all)]
    pub async fn send_password_reset_email(&self, user_email: String) -> Result<(), ServiceError> {
        let user = self.user_repo.get_user_by_email(user_email).await?;
        if user.is_none() {
            tracing::info!(reason = "user_not_found", "auth email request rejected");
            return Ok(());
        }
        let user = user.unwrap();
        let (_, token) = self
            .user_repo
            .issue_auth_token(user.id, AuthTokenPurpose::PasswordReset)
            .await?;

        let reset_url = format!(
            "{}/auth/password_reset?user_id={}&token={}",
            self.api_base_url, user.id, token
        );

        self.email_service
            .send_password_reset_email(&user.email, &reset_url)
            .await?;

        Ok(())
    }

    #[tracing::instrument(name = "auth.send_verification_email", level = "info", skip_all, fields(user_id = %user_id))]
    pub async fn send_verification_email(&self, user_id: Uuid) -> Result<(), ServiceError> {
        let user = self.user_repo.get_user_by_id(user_id).await?;
        if user.is_none() {
            tracing::info!(reason = "user_not_found", "auth email request rejected");
            return Err(ServiceError::NotFound("User not found".to_string()));
        }
        let user = user.unwrap();
        if user.verified {
            tracing::info!(
                reason = "already_verified",
                "verification email request rejected"
            );
            return Err(ServiceError::Conflict(
                "User is already verified".to_string(),
            ));
        }
        let (_, token) = self
            .user_repo
            .issue_auth_token(user_id, AuthTokenPurpose::EmailVerification)
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

    #[tracing::instrument(name = "auth.reset_password", level = "info", skip_all, fields(user_id = %id))]
    pub async fn reset_password(
        &self,
        id: Uuid,
        token: String,
        new_password: String,
    ) -> Result<UserResponse, ServiceError> {
        let new_password_hash = Self::hash_password(new_password).await?;
        let token_hash = hex::encode(Sha256::digest(token.as_bytes()));

        self.user_repo
            .reset_password_with_token(id, token_hash, new_password_hash)
            .await?
            .map(Into::into)
            .ok_or_else(|| ServiceError::NotFound("User not found".into()))
    }

    #[tracing::instrument(name = "auth.verify_user", level = "info", skip_all, fields(user_id = %id))]
    pub async fn verify_user(&self, id: Uuid, token: String) -> Result<UserResponse, ServiceError> {
        let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
        self.user_repo
            .verify_email_with_token(id, token_hash)
            .await?
            .map(Into::into)
            .ok_or_else(|| ServiceError::NotFound("User not found".into()))
    }

    #[tracing::instrument(name = "auth.create_session", level = "debug", skip_all, fields(user_id = %user_id))]
    pub async fn create_session(
        &self,
        user_id: &Uuid,
    ) -> Result<(String, DateTime<Utc>), ServiceError> {
        let token = nanoid::nanoid!();
        let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
        let row = self.user_repo.create_session(&token_hash, user_id).await?;
        tracing::debug!(expires_at = %row.expires_at, "session created");
        Ok((token, row.expires_at))
    }

    #[tracing::instrument(name = "auth.get_user_id_from_session", level = "debug", skip_all)]
    pub async fn get_user_id_from_session(
        &self,
        token: &str,
    ) -> Result<Option<Uuid>, ServiceError> {
        let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
        let row = self.user_repo.get_session(&token_hash).await?;
        match row {
            Some(session) => {
                tracing::debug!(user_id = %session.user_id, "session resolved");
                Ok(Some(session.user_id))
            }
            None => {
                tracing::debug!(reason = "missing_or_expired", "session rejected");
                Ok(None)
            }
        }
    }

    #[tracing::instrument(name = "auth.delete_session", level = "debug", skip_all, fields(user_id = %user_id))]
    pub async fn delete_session(&self, token: &str, user_id: &Uuid) -> Result<bool, ServiceError> {
        let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
        let deleted = self.user_repo.delete_session(&token_hash, user_id).await?;
        tracing::debug!(deleted, "session deletion completed");
        Ok(deleted)
    }

    #[tracing::instrument(name = "auth.begin_google_login", level = "info", skip_all)]
    pub async fn begin_google_login(&self) -> Result<(String, String), ServiceError> {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();

        let (url, state, nonce) = self
            .oauth_client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".into()))
            .add_scope(Scope::new("profile".into()))
            .set_pkce_challenge(challenge)
            .url();

        let state_hash = hex::encode(Sha256::digest(state.secret().as_bytes()));

        self.user_repo
            .create_login_attempt(
                state_hash,
                nonce.secret().to_owned(),
                verifier.secret().to_owned(),
            )
            .await?;

        tracing::debug!(provider = "google", "OAuth login attempt stored");
        Ok((url.to_string(), state.secret().to_owned()))
    }

    #[tracing::instrument(name = "auth.complete_google_login", level = "info", skip_all)]
    pub async fn complete_google_login(
        &self,
        state_hash: String,
        code: String,
    ) -> Result<(User, String, DateTime<Utc>), ServiceError> {
        let attempt = self
            .user_repo
            .consume_login_attempt(state_hash)
            .await?
            .ok_or(ServiceError::AuthError)?;
        let tokens = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|_| ServiceError::InternalError("Google token endpoint unavailable".into()))?
            .set_pkce_verifier(PkceCodeVerifier::new(attempt.pkce_verifier))
            .request_async(&self.http_client)
            .await
            .map_err(|_| ServiceError::InternalError("Google token exchange failed".into()))?;

        let id_token = tokens.id_token().ok_or(ServiceError::AuthError)?;

        let verifier = self.oauth_client.id_token_verifier();
        let nonce = Nonce::new(attempt.nonce);

        let claims = id_token
            .claims(&verifier, &nonce)
            .map_err(|_| ServiceError::AuthError)?;

        let google_sub = claims.subject().as_str();
        let (username, email) = match self.user_repo.get_user_by_google_sub(google_sub).await? {
            Some(user) => (user.username, user.email),
            None => {
                if claims.email_verified() != Some(true) {
                    return Err(ServiceError::AuthError);
                }
                let email = claims
                    .email()
                    .ok_or(ServiceError::AuthError)?
                    .as_str()
                    .to_owned();
                let username = claims
                    .name()
                    .and_then(|name| name.get(None))
                    .map(|name| name.as_str())
                    .unwrap_or("Writer")
                    .to_owned();
                (username, email)
            }
        };

        let token = nanoid::nanoid!();
        let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
        let (user, session) = self
            .user_repo
            .login_with_google(google_sub, &username, &email, &token_hash)
            .await?;

        tracing::info!(user_id = %user.id, "Google login completed");
        Ok((user, token, session.expires_at))
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/oauth_service.rs"]
mod oauth_tests;
