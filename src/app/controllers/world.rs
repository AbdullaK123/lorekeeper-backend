use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::response::{IntoResponse, NoContent};
use axum::routing::{get, post};
use uuid::Uuid;
use crate::app::{AppState, UserId, ValidatedJson};
use crate::app::errors::AppError;
use crate::data::{ConversationItemsResponse, ConversationResponse, CreateConversationRequest, CreateWorldRequest, UpdateWorldRequest, WorldResponse};

async fn create_world(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    ValidatedJson(payload): ValidatedJson<CreateWorldRequest>
) -> Result<Json<WorldResponse>, AppError> {
    let result = ctx.world_service.create_world(
        user_id,
        payload
    ).await?;
    Ok(Json(result))
}

async fn get_worlds(
    State(ctx): State<AppState>,
    UserId(user_id): UserId
) -> Result<Json<Vec<WorldResponse>>, AppError> {
    let result = ctx.world_service.get(user_id).await?;
    Ok(Json(result))
}

async fn get_world(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>
) -> Result<Json<WorldResponse>, AppError> {
    let result = ctx.world_service.get_by_id(world_id, user_id).await?;
    Ok(Json(result))
}

async fn update_world(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateWorldRequest>
) -> Result<Json<WorldResponse>, AppError> {
    let result = ctx.world_service.update_world(world_id, user_id, payload).await?;
    Ok(Json(result))
}

async fn delete_world(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = ctx.world_service.delete_world(world_id, user_id).await?;
    if !result {
        return Err(AppError::NotFound("World not found".to_string()))
    }
    Ok(NoContent)
}

async fn get_conversations(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>
) -> Result<Json<ConversationItemsResponse>, AppError> {
    let result = ctx.conversation_service.get_conversations(world_id, user_id).await?;
    Ok(Json(result))
}

async fn create_conversation(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<CreateConversationRequest>
) -> Result<Json<ConversationResponse>, AppError> {
    let result = ctx.conversation_service.create_conversation(
        world_id,
        user_id,
        payload.title
    ).await?;
    Ok(Json(result))
}

pub fn create_world_controller() -> Router<AppState> {
    Router::new()
        .route("", post(create_world).get(get_worlds))
        .route("/{world_id}", get(get_world).patch(update_world).delete(delete_world))
        .route("/world_id/conversations", get(get_conversations).post(create_conversation))
}