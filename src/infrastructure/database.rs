use crate::infrastructure::{Settings, Config};
use anyhow::Result;
use std::time::Duration;
use sqlx::postgres::{
    PgPool,
    PgPoolOptions
};

pub async fn create_pool(settings: &Settings, config: &Config) -> Result<PgPool> {
    let pool =
        PgPoolOptions::new()
            .max_connections(config.db.max_connections)
            .min_connections(config.db.min_connections)
            .max_lifetime(Duration::from_secs(config.db.max_lifetime))
            .acquire_timeout(Duration::from_secs(config.db.acquire_timeout))
            .idle_timeout(Duration::from_secs(config.db.idle_timeout))
            .connect(settings.database_url.as_str())
            .await?;
    Ok(pool)
}