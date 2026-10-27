mod app;
mod cron_worker;
mod data;
mod infrastructure;
mod service;

use app::create_app;
use infrastructure::init_tracing;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize global tracing/logging as early as possible
    init_tracing();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;

    let (app, mut scheduler) = create_app().await;

    info!("Started server on port 8000...");
    let server_result = axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for Ctrl+C")
        })
        .await;

    scheduler.shutdown().await?;
    server_result?;

    Ok(())
}
