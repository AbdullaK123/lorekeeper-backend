use sqlx::FromRow;
use serde::{Deserialize};
use chrono::prelude::*;
use uuid::Uuid;
use validator::Validate;
use passcheck::PasswordChecker;
use crate::data::errors::DataError;

#[derive(FromRow, Debug, Clone, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

fn validate_password(password: &str) -> Result<(), validator::ValidationError> {
    let checker = PasswordChecker::new()
        .min_length(8, Some("Password must contain at least 8 characters"))
        .require_special_char(Some("Password must contain at least one special character"))
        .require_upper_lower(Some("Password must contain at least one upper and lowercase letter"))
        .require_number(Some("Password must contain at least one number"));
    match checker.validate(password) {
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
    #[validate(length(min=8, max=100))]
    pub password: String
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(length(min=1, max=100))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(max=100), custom(function="validate_password"))]
    pub password: String
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            created_at: value.created_at,
            updated_at: value.updated_at
        }
    }
}