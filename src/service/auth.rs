use crate::data::{SignupRequest, User, UserRepository, UserResponse};
use crate::service::ServiceError;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use tokio;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
}

impl AuthService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
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
}
