use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::response::{IntoResponse, NoContent};
use axum::routing::{get, Route};
use uuid::Uuid;
use crate::app::{AppState, UserId};
use crate::app::errors::AppError;
use crate::data::ConversationResponse;

async fn get_conversation(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>
) -> Result<Json<ConversationResponse>, AppError> {
    let result = ctx.conversation_service.get_conversation(
        id,
        user_id
    ).await?;
    Ok(Json(result))
}

async fn delete_conversation(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>
) -> Result<impl IntoResponse, AppError> {
    let result = ctx.conversation_service.delete_conversation(
        id,
        user_id
    ).await?;
    if !result {
        return Err(AppError::NotFound("Conversation not found".to_string()))
    }
    Ok(NoContent)
}

pub fn create_conversation_controller() -> Router<AppState> {
    Router::new()
        .route("/{id}", get(get_conversation).delete(delete_conversation))
}