use axum::Router;
use std::path::Path;
use tower_http::services::ServeDir;

use crate::app::config::Environment;

pub(crate) fn with_dev_playground(
    app: Router,
    environment: &Environment,
    directory: impl AsRef<Path>,
) -> Router {
    if *environment != Environment::Dev {
        return app;
    }
    tracing::info!("serving development playground at /");
    Router::new()
        .nest("/api", app.clone())
        .merge(app)
        .fallback_service(ServeDir::new(directory))
}

#[cfg(test)]
#[path = "../../tests/unit/playground.rs"]
mod tests;
