use serde::{Deserialize, Serialize};
use std::fs::File;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use serde_yaml_bw;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tower_http::trace::{HttpMakeClassifier, TraceLayer};
use tower_sessions::service::SignedCookie;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store_chrono::PostgresStore;
use crate::app::{create_cors_layer, create_session_layer, create_trace_layer};

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
    session_secret_key: String,
    cors_allowed_origins: Vec<String>,
    pub env: Environment
}

impl Settings {
    pub fn cors_layer(&self) -> CorsLayer {
        let mut parsed_origins: Vec<HeaderValue> = self
            .cors_allowed_origins
            .iter()
            .map(|origin| origin.parse::<HeaderValue>().expect("Invalid CORS origin format"))
            .collect();

        // Automatically inject local development origin if in Dev mode
        if self.env == Environment::Dev && !self.cors_allowed_origins.iter().any(|o| o.contains("localhost")) {
            if let Ok(local_origin) = "http://localhost:3000".parse::<HeaderValue>() {
                parsed_origins.push(local_origin);
            }
        }

        create_cors_layer(
            vec![AUTHORIZATION, CONTENT_TYPE, ACCEPT],
            parsed_origins,
            vec![Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS],
            true
        )
    }

    pub async fn session_layer(&self, pool: PgPool) -> SessionManagerLayer<PostgresStore, SignedCookie> {
        // Hands off execution to your factory, which handles migration and key validation safely
        create_session_layer(
            pool,
            self.env == Environment::Prod,
            &self.session_secret_key
        ).await
    }

    pub fn trace_layer(&self) -> TraceLayer<HttpMakeClassifier> {
        create_trace_layer()
    }
}


#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64
}

#[derive(Debug, Deserialize)]
pub struct MemgraphConfig {
    pub max_connections: usize,
    pub fetch_size: usize
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub db: DatabaseConfig,
    pub memgraph: MemgraphConfig
}

pub fn load_settings() -> Settings {
    envy::from_env::<Settings>()
        .expect("Invalid environment configuration.")
}

pub fn load_config() -> Config {
    let file = File::open("./config.yaml")
        .expect("Failed to read config file.");
    serde_yaml_bw::from_reader::<File, Config>(file)
        .expect("Failed to parse config file.")
}