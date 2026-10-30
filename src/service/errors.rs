use crate::infrastructure::InfrastructureError;
use rig::{ProviderError, run::PromptError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ServiceError {
    #[error("{0}")]
    Conflict(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Invalid Credentials")]
    AuthError,
}

impl From<InfrastructureError> for ServiceError {
    fn from(value: InfrastructureError) -> Self {
        match value {
            InfrastructureError::Conflict(message) => Self::Conflict(message),
            other => Self::InternalError(other.to_string()),
        }
    }
}

impl From<ProviderError> for ServiceError {
    fn from(value: ProviderError) -> Self {
        Self::InternalError(value.to_string())
    }
}

impl From<PromptError> for ServiceError {
    fn from(value: PromptError) -> Self {
        Self::InternalError(value.to_string())
    }
}
