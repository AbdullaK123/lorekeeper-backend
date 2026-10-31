use crate::app::errors::AppError;
use crate::app::{AppState, UserId, ValidatedJson, ValidatedQuery};
use crate::data::{
    ConflictListQuery, ConflictResponse, ConversationItemsResponse, ConversationResponse,
    CreateConversationRequest, CreateWorldRequest, EntityListQuery, EntityResponse,
    EntitySearchQuery, GraphQuery, GraphResponse, NeighborQuery, RelationshipResponse,
    UpdateWorldRequest, WorldResponse,
};
use crate::shared::PageResponse;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, NoContent};
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

async fn create_world(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    ValidatedJson(payload): ValidatedJson<CreateWorldRequest>,
) -> Result<Json<WorldResponse>, AppError> {
    let result = ctx.world_service.create_world(user_id, payload).await?;
    Ok(Json(result))
}

async fn get_worlds(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
) -> Result<Json<Vec<WorldResponse>>, AppError> {
    let result = ctx.world_service.get(user_id).await?;
    Ok(Json(result))
}

async fn get_world(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>,
) -> Result<Json<WorldResponse>, AppError> {
    let result = ctx.world_service.get_by_id(world_id, user_id).await?;
    Ok(Json(result))
}

async fn update_world(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateWorldRequest>,
) -> Result<Json<WorldResponse>, AppError> {
    let result = ctx
        .world_service
        .update_world(world_id, user_id, payload)
        .await?;
    Ok(Json(result))
}

async fn delete_world(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = ctx.world_service.delete_world(world_id, user_id).await?;
    if !result {
        return Err(AppError::NotFound("World not found".to_string()));
    }
    Ok(NoContent)
}

async fn get_conversations(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>,
) -> Result<Json<ConversationItemsResponse>, AppError> {
    let result = ctx
        .conversation_service
        .get_conversations(world_id, user_id)
        .await?;
    Ok(Json(result))
}

async fn create_conversation(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<CreateConversationRequest>,
) -> Result<Json<ConversationResponse>, AppError> {
    let result = ctx
        .conversation_service
        .create_conversation(world_id, user_id, payload.title.into_inner())
        .await?;
    Ok(Json(result))
}

async fn get_graph(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>,
    ValidatedQuery(query): ValidatedQuery<GraphQuery>,
) -> Result<Json<PageResponse<GraphResponse>>, AppError> {
    Ok(Json(
        ctx.world_service
            .get_graph(world_id, user_id, &query)
            .await?,
    ))
}

async fn list_entities(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>,
    ValidatedQuery(query): ValidatedQuery<EntityListQuery>,
) -> Result<Json<PageResponse<Vec<EntityResponse>>>, AppError> {
    Ok(Json(
        ctx.world_service
            .list_entities(world_id, user_id, &query)
            .await?,
    ))
}

async fn search_entities(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>,
    ValidatedQuery(query): ValidatedQuery<EntitySearchQuery>,
) -> Result<Json<PageResponse<Vec<EntityResponse>>>, AppError> {
    Ok(Json(
        ctx.world_service
            .search_entities(world_id, user_id, &query)
            .await?,
    ))
}

async fn list_conflicts(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path(world_id): Path<Uuid>,
    ValidatedQuery(query): ValidatedQuery<ConflictListQuery>,
) -> Result<Json<PageResponse<Vec<ConflictResponse>>>, AppError> {
    Ok(Json(
        ctx.world_service
            .list_conflicts(world_id, user_id, &query)
            .await?,
    ))
}

async fn get_entity(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path((world_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<EntityResponse>, AppError> {
    Ok(Json(
        ctx.world_service.get_entity(world_id, user_id, id).await?,
    ))
}

async fn get_relationship(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path((world_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<RelationshipResponse>, AppError> {
    Ok(Json(
        ctx.world_service
            .get_relationship(world_id, user_id, id)
            .await?,
    ))
}

async fn get_conflict(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path((world_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ConflictResponse>, AppError> {
    Ok(Json(
        ctx.world_service
            .get_conflict(world_id, user_id, id)
            .await?,
    ))
}

async fn get_entity_neighbors(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
    Path((world_id, entity_id)): Path<(Uuid, Uuid)>,
    ValidatedQuery(query): ValidatedQuery<NeighborQuery>,
) -> Result<Json<PageResponse<GraphResponse>>, AppError> {
    Ok(Json(
        ctx.world_service
            .get_entity_neighbors(world_id, user_id, entity_id, &query)
            .await?,
    ))
}

pub fn create_world_controller() -> Router<AppState> {
    Router::new()
        .route("/{world_id}/graph", get(get_graph))
        .route("/{world_id}/entities", get(list_entities))
        .route("/{world_id}/entities/search", get(search_entities))
        .route("/{world_id}/entities/{entity_id}", get(get_entity))
        .route(
            "/{world_id}/entities/{entity_id}/neighbors",
            get(get_entity_neighbors),
        )
        .route(
            "/{world_id}/relationships/{relationship_id}",
            get(get_relationship),
        )
        .route("/{world_id}/conflicts", get(list_conflicts))
        .route("/{world_id}/conflicts/{conflict_id}", get(get_conflict))
        .route("/", post(create_world).get(get_worlds))
        .route(
            "/{world_id}",
            get(get_world).patch(update_world).delete(delete_world),
        )
        .route(
            "/{world_id}/conversations",
            get(get_conversations).post(create_conversation),
        )
}
