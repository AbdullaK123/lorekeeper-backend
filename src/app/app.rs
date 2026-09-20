use axum::{Router};
use axum::routing::get;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::Level;
use sqlx::postgres::PgPool;
use crate::infrastructure::{create_pool, load_config, load_settings};

#[derive(Clone)]
struct AppState {
    pool: PgPool
}

pub async fn create_app() -> Router {

    let settings = load_settings();
    let config = load_config();
    let pool = create_pool(&settings, &config).await.expect("Failed to initialize database pool");

    let app_state = AppState {
        pool
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .layer(
            TraceLayer::new_for_http()
                // Ensure spans and events are emitted at INFO by default
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
        .with_state(app_state);

    app
}