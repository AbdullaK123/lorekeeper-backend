use crate::infrastructure::InfrastructureError;
use rig::{ProviderError, completion::StructuredOutputError, run::PromptError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Invalid Credentials")]
    AuthError,
}

impl From<InfrastructureError> for ServiceError {
    fn from(value: InfrastructureError) -> Self {
        Self::InternalError(value.to_string())
    }
}

impl From<StructuredOutputError> for ServiceError {
    fn from(value: StructuredOutputError) -> Self {
        Self::InternalError(value.to_string())
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
