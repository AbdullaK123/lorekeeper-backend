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
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn playground_and_api_alias_exist_only_in_dev() {
        let directory =
            std::env::temp_dir().join(format!("playground-static-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(directory.join("assets")).unwrap();
        std::fs::write(directory.join("index.html"), "<h1>Test playground</h1>").unwrap();
        std::fs::write(directory.join("assets/test.js"), "console.log('static');").unwrap();
        for environment in [Environment::Dev, Environment::Staging, Environment::Prod] {
            let app = with_dev_playground(
                Router::new().route("/health", get(|| async { "healthy" })),
                &environment,
                &directory,
            );
            for path in ["/", "/assets/test.js", "/api/health"] {
                let response = app
                    .clone()
                    .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                    .await
                    .unwrap();
                assert_eq!(
                    response.status(),
                    if environment == Environment::Dev {
                        StatusCode::OK
                    } else {
                        StatusCode::NOT_FOUND
                    },
                    "Static files or the development API alias escape the dev environment: {environment:?} {path}"
                );
                if environment == Environment::Dev && path == "/" {
                    assert_eq!(
                        to_bytes(response.into_body(), 1024).await.unwrap().as_ref(),
                        b"<h1>Test playground</h1>",
                        "The root route does not serve the built playground entry"
                    );
                }
            }
            for path in ["/api/missing", "/assets/missing.js", "/%2e%2e/Cargo.toml"] {
                let response = app
                    .clone()
                    .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                    .await
                    .unwrap();
                assert_ne!(
                    response.status(),
                    StatusCode::OK,
                    "Unknown API routes or traversal return playground content: {path}"
                );
            }
            let response = app
                .oneshot(
                    Request::builder()
                        .uri("/health")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "Adding static files breaks the original API route"
            );
        }
        std::fs::remove_dir_all(directory).unwrap();
    }
}
