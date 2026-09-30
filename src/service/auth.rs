use tokio;
use crate::data::{UserRepository, User, LoginRequest, UserResponse, SignupRequest};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use uuid::Uuid;
use crate::service::ServiceError;

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository
}

impl AuthService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self {
            user_repo
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

    pub async fn authenticate_user(&self, email: String, password: String) -> Result<User, ServiceError> {
        let user = self.user_repo.get_user_by_email(email)
            .await?;
        if user.is_none() {
            return Err(ServiceError::NotFound("A user with that email does not exist".to_string()))
        }
        let unwrapped_user = user.unwrap();
        if unwrapped_user.password_hash.is_none() {
            return Err(ServiceError::NotFound("Password hash not found. OAuth accounts should not use this method.".to_string()))
        }
        let password_hash = PasswordHash::new(&unwrapped_user.password_hash.clone().unwrap())
            .map_err(|e| ServiceError::InternalError(e.to_string()))?;
        let password_valid = Argon2::default()
            .verify_password(password.as_bytes(), &password_hash)
            .is_ok();
        if !password_valid {
            return Err(ServiceError::AuthError)
        }
        Ok(unwrapped_user.clone())
    }
    
    pub async fn signup(&self, payload: SignupRequest) -> Result<UserResponse, ServiceError> {
        let password_hash = Self::hash_password(payload.password).await?;
        let user = self.user_repo.sign_up(
            payload.username,
            payload.email,
            password_hash
        ).await?;
        Ok(user.into())
    }
    
    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserResponse, ServiceError> {
        let user = self.user_repo.get_user_by_id(user_id).await?;
        match user {
            Some(user) => Ok(user.into()),
            None => Err(ServiceError::AuthError)
        }
    }
}