use axum::{Json, Router};
use axum::routing::get;

pub async fn create_app() -> Router {

    let app = Router::new()
        .route("/health", get(|| async { "ok" }));

    app
}