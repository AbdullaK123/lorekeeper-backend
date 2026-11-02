use crate::infrastructure::{DatabaseConfig, MemgraphConfig, Settings};
use neo4rs::{ConfigBuilder, Graph};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(settings: &Settings, config: &DatabaseConfig) -> PgPool {
    tracing::info!(
        max_connections = config.max_connections,
        "connecting to Postgres"
    );
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .max_lifetime(Duration::from_secs(config.max_lifetime))
        .acquire_timeout(Duration::from_secs(config.acquire_timeout))
        .idle_timeout(Duration::from_secs(config.idle_timeout))
        .connect(settings.database_url.as_str())
        .await
        .expect("Failed to connect to postgres");
    tracing::info!("Postgres pool ready");
    pool
}

pub async fn create_graph(settings: &Settings, config: &MemgraphConfig) -> Graph {
    tracing::info!(
        max_connections = config.max_connections,
        "connecting to Memgraph"
    );
    let neo_config = ConfigBuilder::default()
        .uri(&settings.memgraph_uri)
        // neo4rs defaults to Neo4j's "neo4j" database; Memgraph's is "memgraph".
        .db("memgraph")
        .user(&settings.memgraph_user)
        .password(&settings.memgraph_password)
        .max_connections(config.max_connections)
        .fetch_size(config.fetch_size)
        .build()
        .expect("Invalid Memgraph Config");

    let graph = Graph::connect(neo_config)
        .await
        .expect("Failed to connect to Memgraph");
    tracing::info!("Memgraph connection ready");
    graph
}
