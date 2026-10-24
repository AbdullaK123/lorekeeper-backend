use crate::app::errors::AppError;
use crate::app::{AppState, UserId, ValidatedJson};
use crate::data::{ChatRequest, ConversationResponse, Message};
use crate::service::format_hits;
use axum::extract::{Path, State};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, NoContent};
use axum::routing::{Route, get, post};
use axum::{Json, Router};
use chrono::Utc;
use futures::{Stream, StreamExt};
use rig::agent::MultiTurnStreamItem;
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
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, AppError> {
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
    // The conversation as it was before this turn: extraction's context for resolving "his", "the city", ...
    let earlier_messages: Vec<Message> = conversation.messages.0.clone();
    let agent = ctx
        .agent_factory
        .create_agent(conversation.world_id.to_string(), lore);

    let events = async_stream::stream! {
        let mut rig_stream = agent
            .prompt(payload.content.clone())
            .history(history)
            .max_turns(5)
            .stream();
        while let Some(item) = rig_stream.next().await {
            match item {
                Ok(item) => {
                    if let MultiTurnStreamItem::FinalResponse(resp) = &item {
                        let reply = Message::Assistant {
                            content: Some(resp.output().to_string()),
                            tool_calls: None,
                            created_at: Utc::now()
                        };
                        let svc = &ctx.conversation_service;
                        if let Err(e) = svc.append_message(id, user_id, message.clone()).await {
                            tracing::error!("saving user message failed: {e}");
                        }
                        if let Err(e) = svc.append_message(id, user_id, reply).await {
                            tracing::error!("saving reply failed: {e}");
                        }
                        let extraction = ctx.extraction_service.clone();
                        let world_id = conversation.world_id;
                        let text = payload.content.clone();
                        let earlier = earlier_messages.clone();
                        tokio::spawn(async move {
                            if let Err(e) = extraction.process_message(world_id, id, &text, &earlier).await {
                                tracing::error!("extraction failed for conversation {id}: {e}");
                            }
                        });
                    }
                    yield Event::default().json_data(&item)
                },
                Err(e) => {
                    yield Ok(Event::default().event("error").data(e.to_string()));
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
