use crate::app::errors::AppError;
use crate::app::{AppState, UserId, ValidatedJson};
use crate::data::{ChatRequest, ConversationResponse, Message};
use crate::service::format_hits;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, NoContent};
use axum::routing::{Route, get, post};
use axum::{Json, Router};
use chrono::Utc;
use rig::message::Message as RigMessage;
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
) -> Result<impl IntoResponse, AppError> {
    let conversation = ctx
        .conversation_service
        .get_conversation(id, user_id)
        .await?;
    let hits = ctx
        .retrieval_service
        .search(&payload.content, &conversation.world_id.to_string(), 60.0)
        .await?;
    let lore = format_hits(&hits);
    let message = Message::User {
        content: payload.content.clone(),
        created_at: Utc::now(),
    };
    let history: Vec<RigMessage> = conversation
        .messages
        .iter()
        .filter_map(Message::to_rig)
        .collect();
    let agent = ctx
        .agent_factory
        .create_agent(conversation.world_id.to_string(), lore);
    let response = agent
        .prompt(payload.content.clone())
        .history(history)
        .max_turns(5)
        .await?;
    let reply = response.output().to_string();
    ctx.conversation_service
        .append_message(id, user_id, message)
        .await?;

    let reply_message = Message::Assistant {
        content: Some(reply.clone()),
        tool_calls: None,
        created_at: Utc::now(),
    };
    let updated = ctx
        .conversation_service
        .append_message(id, user_id, reply_message)
        .await?;
    let extraction = ctx.extraction_service.clone();
    let world_id = conversation.world_id;
    let text = payload.content.clone();
    tokio::spawn(async move {
        if let Err(e) = extraction.process_message(world_id, id, &text).await {
            tracing::error!("extraction failed for conversation {id}: {e}");
        }
    });
    Ok(Json(updated))
}

pub fn create_conversation_controller() -> Router<AppState> {
    Router::new()
        .route("/{id}", get(get_conversation).delete(delete_conversation))
        .route("/{id}/messages", post(chat_turn))
}
