use axum::http::{HeaderName, HeaderValue, Method};
use tower_http::cors::{
    CorsLayer
};

pub fn create_cors_layer(
    allowed_headers: Vec<HeaderName>,
    allowed_origins: Vec<HeaderValue>,
    allowed_methods: Vec<Method>,
    allow_credentials: bool
) -> CorsLayer {
    CorsLayer::new()
        .allow_headers(allowed_headers)
        .allow_credentials(allow_credentials)
        .allow_methods(allowed_methods)
        .allow_origin(allowed_origins)
}