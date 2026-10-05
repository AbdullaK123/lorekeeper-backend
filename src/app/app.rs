use axum::{Router};
use axum::routing::get;
use neo4rs::Graph;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::Level;
use sqlx::postgres::PgPool;
use crate::app::{create_auth_controller, create_session_layer};
use crate::data::UserRepository;
use crate::infrastructure::{create_graph, create_pool, load_config, load_settings};
use crate::service::AuthService;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    graph: Graph,
    pub auth_service: AuthService
}

pub async fn create_app() -> Router {

    let settings = load_settings();
    let config = load_config();
    let pool = create_pool(&settings, &config.db).await;
    let graph = create_graph(&settings, &config.memgraph).await;
    let user_repo = UserRepository::new(pool.clone());
    let auth_service = AuthService::new(user_repo);

    let app_state = AppState {
        pool: pool.clone(),
        graph,
        auth_service
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/auth", create_auth_controller())
        .layer(settings.trace_layer())
        .layer(settings.session_layer(pool.clone()).await)
        .layer(settings.cors_layer())
        .with_state(app_state);

    app
}