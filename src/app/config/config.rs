use crate::app::{create_cors_layer, create_trace_layer};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use serde::{Deserialize, Serialize};
use serde_yaml_bw;
use std::fs::File;
use tower_http::cors::CorsLayer;
use tower_http::trace::{HttpMakeClassifier, TraceLayer};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Dev,
    Staging,
    Prod,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub memgraph_uri: String,
    pub memgraph_user: String,
    pub memgraph_password: String,
    pub resend_api_key: String,
    pub api_base_url: String,
    cors_allowed_origins: Vec<String>,
    pub env: Environment,
    pub client_id: String,
    pub client_secret: String,
}

impl Settings {
    pub fn cors_layer(&self) -> CorsLayer {
        let mut parsed_origins: Vec<HeaderValue> = self
            .cors_allowed_origins
            .iter()
            .map(|origin| {
                origin
                    .parse::<HeaderValue>()
                    .expect("Invalid CORS origin format")
            })
            .collect();

        // Automatically inject local development origin if in Dev mode
        if self.env == Environment::Dev
            && !self
                .cors_allowed_origins
                .iter()
                .any(|o| o.contains("localhost"))
        {
            if let Ok(local_origin) = "http://localhost:3000".parse::<HeaderValue>() {
                parsed_origins.push(local_origin);
            }
        }

        create_cors_layer(
            vec![AUTHORIZATION, CONTENT_TYPE, ACCEPT],
            parsed_origins,
            vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ],
            true,
        )
    }

    pub fn trace_layer(&self) -> TraceLayer<HttpMakeClassifier, crate::app::RequestSpan> {
        create_trace_layer()
    }
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

#[derive(Debug, Deserialize)]
pub struct MemgraphConfig {
    pub max_connections: usize,
    pub fetch_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub db: DatabaseConfig,
    pub memgraph: MemgraphConfig,
}

pub fn load_settings() -> Settings {
    envy::from_env::<Settings>().expect("Invalid environment configuration.")
}

pub fn load_config() -> Config {
    let file = File::open("./src/app/config/config.yaml").expect("Failed to read config file.");
    serde_yaml_bw::from_reader::<File, Config>(file).expect("Failed to parse config file.")
}
