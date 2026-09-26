mod app;
mod infrastructure;
mod data;
mod service;

use app::create_app;
use tracing::{info};
use infrastructure::init_tracing;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    // Initialize global tracing/logging as early as possible
    init_tracing();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;

    let app = create_app().await;

    info!("Started server on port 8000...");
    axum::serve(listener, app).await?;

    Ok(())
}
