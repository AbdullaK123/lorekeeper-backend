use crate::shared::{PasswordInput, Username};
use chrono::prelude::*;
use passcheck::PasswordChecker;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(FromRow, Debug, Clone, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub verified: bool,
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub google_sub: Option<String>,
}

fn validate_password(password: &PasswordInput) -> Result<(), validator::ValidationError> {
    let checker = PasswordChecker::new()
        .min_length(8, Some("Password must contain at least 8 characters"))
        .require_special_char(Some("Password must contain at least one special character"))
        .require_upper_lower(Some(
            "Password must contain at least one upper and lowercase letter",
        ))
        .require_number(Some("Password must contain at least one number"));
    match checker.validate(password.as_str()) {
        Ok(()) => Ok(()),
        Err(errors) => {
            let mut err = validator::ValidationError::new("weak_password");
            err.message = Some(errors.join("; ").into());
            Err(err)
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: PasswordInput,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SignupRequest {
    pub username: Username,
    #[validate(email)]
    pub email: String,
    #[validate(custom(function = "validate_password"))]
    pub password: PasswordInput,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PasswordResetRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PasswordResetParams {
    pub user_id: Uuid,
    #[validate(length(min = 1, max = 256))]
    pub token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            verified: value.verified,
            email: value.email,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct VerifyEmailRequest {
    pub user_id: Uuid,
    #[validate(length(min = 1, max = 256))]
    pub token: String,
}

#[derive(Deserialize, Validate)]
pub struct PasswordResetPayload {
    #[validate(custom(function = "validate_password"))]
    pub new_password: PasswordInput,
}
