use axum::extract::MatchedPath;
use axum::http::Request;
use tower_http::trace::{
    DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, HttpMakeClassifier, MakeSpan, TraceLayer,
};
use tracing::{Level, Span};

/// Keep request bodies, query strings, and credentials out of request spans.
#[derive(Clone, Copy)]
pub struct RequestSpan;

impl<B> MakeSpan<B> for RequestSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let route = request
            .extensions()
            .get::<MatchedPath>()
            .map(MatchedPath::as_str)
            .unwrap_or("unmatched");
        tracing::info_span!("http_request", method = %request.method(), route)
    }
}

pub fn create_trace_layer() -> TraceLayer<HttpMakeClassifier, RequestSpan> {
    TraceLayer::new_for_http()
        .make_span_with(RequestSpan)
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
        .on_failure(DefaultOnFailure::new().level(Level::ERROR))
}
