use axum::{Router};
use axum::routing::get;
use neo4rs::Graph;
use sqlx::postgres::PgPool;
use crate::app::{create_auth_controller, create_conversation_controller, create_world_controller};
use crate::data::{ConversationRepository, UserRepository, WorldRepository};
use crate::infrastructure::{create_graph, create_pool, load_config, load_settings};
use crate::service::{AuthService, ConversationService, WorldService};

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    graph: Graph,
    pub auth_service: AuthService,
    pub world_service: WorldService,
    pub conversation_service: ConversationService
}

pub async fn create_app() -> Router {

    let settings = load_settings();
    let config = load_config();
    let pool = create_pool(&settings, &config.db).await;
    let graph = create_graph(&settings, &config.memgraph).await;
    let user_repo = UserRepository::new(pool.clone());
    let world_repo = WorldRepository::new(pool.clone());
    let conversation_repo = ConversationRepository::new(pool.clone());
    let auth_service = AuthService::new(user_repo);
    let world_service = WorldService::new(world_repo);
    let conversation_service = ConversationService::new(conversation_repo);

    let app_state = AppState {
        pool: pool.clone(),
        graph,
        auth_service,
        world_service,
        conversation_service
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/auth", create_auth_controller())
        .nest("/world", create_world_controller())
        .nest("/conversation", create_conversation_controller())
        .layer(settings.trace_layer())
        .layer(settings.session_layer(pool.clone()).await)
        .layer(settings.cors_layer())
        .with_state(app_state);

    app
}