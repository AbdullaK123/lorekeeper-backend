//! Capture real tracing output without a global subscriber or external services.
use super::*;
use axum::{Router, body::Body, http::Request, response::IntoResponse, routing::get};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use tracing::instrument::WithSubscriber;

#[derive(Clone, Default)]
struct LogBuffer(Arc<Mutex<Vec<u8>>>);

impl Write for LogBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogBuffer {
    type Writer = Self;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

impl LogBuffer {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
    fn subscriber(&self) -> impl tracing::Subscriber + Send + Sync + 'static {
        tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(self.clone())
            .finish()
    }
}

#[tokio::test]
async fn request_logs_hide_query_credentials_and_body() {
    let logs = LogBuffer::default();
    let app = Router::new()
        .route("/world/{world_id}/entities/search", get(|| async { "ok" }))
        .layer(app::create_trace_layer());
    let response = app.oneshot(Request::builder()
        .uri("/world/00000000-0000-4000-8000-000000000001/entities/search?query=PRIVATE_LORE_QUERY")
        .header("authorization", "Bearer PRIVATE_BEARER_TOKEN")
        .header("cookie", "session=PRIVATE_SESSION_COOKIE")
        .body(Body::from("PRIVATE_REQUEST_BODY")).unwrap())
        .with_subscriber(logs.subscriber()).await.unwrap();
    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Logging changes request handling"
    );
    let output = logs.text();
    assert!(
        output.contains("/world/{world_id}/entities/search"),
        "Request logs lose the route template"
    );
    assert!(
        output.contains("status=200"),
        "Request logs lose response status"
    );
    for private in [
        "PRIVATE_LORE_QUERY",
        "PRIVATE_BEARER_TOKEN",
        "PRIVATE_SESSION_COOKIE",
        "PRIVATE_REQUEST_BODY",
    ] {
        assert!(
            !output.contains(private),
            "Request tracing leaks private input: {private}"
        );
    }
}

#[test]
fn request_failure_logs_hide_internal_payloads() {
    let logs = LogBuffer::default();
    let response = tracing::subscriber::with_default(logs.subscriber(), || {
        app::errors::AppError::InternalError("PRIVATE_PROVIDER_PAYLOAD".into()).into_response()
    });
    assert_eq!(
        response.status(),
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "Logging changes the failure response"
    );
    let output = logs.text();
    assert!(
        output.contains("request failed") && output.contains("internal"),
        "Request failure logs lose the error category"
    );
    assert!(
        !output.contains("PRIVATE_PROVIDER_PAYLOAD"),
        "Error logging leaks private provider data"
    );
}
