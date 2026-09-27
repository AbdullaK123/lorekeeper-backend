use thiserror::Error;
use crate::infrastructure::InfrastructureError;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Invalid Credentials")]
    AuthError
}

impl From<InfrastructureError> for ServiceError {
    fn from(value: InfrastructureError) -> Self {
        Self::InternalError(value.to_string())
    }
}