use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;
use crate::infrastructure::InfrastructureError;
use crate::service::ServiceError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("Forbidden error: {0}")]
    ForbiddenError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Resource not found: {0}")]
    NotFound(String)
}

impl From<InfrastructureError> for AppError {
    fn from(value: InfrastructureError) -> Self {
        AppError::InternalError(value.to_string())
    }
}

impl From<ServiceError> for AppError {
    fn from(value: ServiceError) -> Self {
        match value {
            ServiceError::AuthError => Self::AuthError(value.to_string()),
            ServiceError::NotFound(e) => Self::NotFound(e.to_string()),
            ServiceError::InternalError(e) => Self::InternalError(e.to_string())
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::InternalError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            AppError::AuthError(e) => (StatusCode::UNAUTHORIZED, e),
            AppError::NotFound(e) => (StatusCode::NOT_FOUND, e),
            AppError::ForbiddenError(e) => (StatusCode::FORBIDDEN, e),
            AppError::ValidationError(e) => (StatusCode::UNPROCESSABLE_ENTITY, e)
        };
        (status, Json(json!({"message": message}))).into_response()
    }
}