use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;
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
    #[error("Downstream service failed: {0}")]
    DownstreamServiceError(String),
    #[error("Resource not found: {0}")]
    NotFound(String)
}

impl From<InfrastructureError> for AppError {
    fn from(value: InfrastructureError) -> Self {
        AppError::DownstreamServiceError(value.to_string())
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

impl From<ValidationErrors> for AppError {
    fn from(value: ValidationErrors) -> Self {
        let messages: Vec<String> = value
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| {
                    match &e.message {
                        Some(msg) => format!("{}: {}", field, msg),
                        None => format!("{}: invalid", field),
                    }
                })
            })
            .collect();

        AppError::ValidationError(messages.join("; "))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, tag, message) = match &self {
            AppError::InternalError(e) => (StatusCode::INTERNAL_SERVER_ERROR, "internal", e),
            AppError::AuthError(e) => (StatusCode::UNAUTHORIZED, "auth", e),
            AppError::NotFound(e) => (StatusCode::NOT_FOUND, "not_found", e),
            AppError::ForbiddenError(e) => (StatusCode::FORBIDDEN, "forbidden", e),
            AppError::ValidationError(e) => (StatusCode::UNPROCESSABLE_ENTITY, "validation", e),
            AppError::DownstreamServiceError(e) => (StatusCode::SERVICE_UNAVAILABLE, "downstream", e),
        };
        (status, Json(json!({"error": tag, "message": message}))).into_response()
    }
}