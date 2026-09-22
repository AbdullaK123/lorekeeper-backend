use crate::infrastructure::{Settings, DatabaseConfig, MemgraphConfig};
use neo4rs::{ConfigBuilder, Graph};
use std::time::Duration;
use sqlx::postgres::{
    PgPool,
    PgPoolOptions
};

pub async fn create_pool(settings: &Settings, config: &DatabaseConfig) -> PgPool {
    let pool =
        PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .max_lifetime(Duration::from_secs(config.max_lifetime))
            .acquire_timeout(Duration::from_secs(config.acquire_timeout))
            .idle_timeout(Duration::from_secs(config.idle_timeout))
            .connect(settings.database_url.as_str())
            .await
            .expect("Failed to connect to postgres");
    pool
}

pub async fn create_graph(settings: &Settings, config: &MemgraphConfig) -> Graph {
    let neo_config =
        ConfigBuilder::default()
            .uri(&settings.memgraph_uri)
            .user(&settings.memgraph_user)
            .password(&settings.memgraph_password)
            .max_connections(config.max_connections)
            .fetch_size(config.fetch_size)
            .build()
            .expect("Invalid Memgraph Config");

    Graph::connect(neo_config)
        .await
        .expect("Failed to connect to Memgraph")
}