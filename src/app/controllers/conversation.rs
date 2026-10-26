use crate::app::errors::AppError;
use crate::app::{AppState, UserId, ValidatedJson};
use crate::data::{ChatRequest, ConversationResponse};
use axum::extract::{Path, State};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, NoContent};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::{Stream, StreamExt};
use uuid::Uuid;

async fn get_conversation(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
) -> Result<Json<ConversationResponse>, AppError> {
    let result = ctx
        .conversation_service
        .get_conversation(id, user_id)
        .await?;
    Ok(Json(result))
}

async fn delete_conversation(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = ctx
        .conversation_service
        .delete_conversation(id, user_id)
        .await?;
    if !result {
        return Err(AppError::NotFound("Conversation not found".to_string()));
    }
    Ok(NoContent)
}

async fn chat_turn(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, AppError> {
    let stream = ctx
        .conversation_service
        .chat_turn(id, user_id, payload.content)
        .await?;
    let events = async_stream::stream! {
        futures::pin_mut!(stream);
        while let Some(item) = stream.next().await {
            match item {
                Ok(item) => yield Event::default().json_data(&item),
                Err(error) => {
                    yield Ok(Event::default().event("error").data(error.to_string()));
                    break;
                }
            }
        }
    };
    Ok(Sse::new(events))
}

pub fn create_conversation_controller() -> Router<AppState> {
    Router::new()
        .route("/{id}", get(get_conversation).delete(delete_conversation))
        .route("/{id}/messages", post(chat_turn))
}
