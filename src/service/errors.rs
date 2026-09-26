use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Invalid Credentials")]
    AuthError
}