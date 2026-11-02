mod app;
mod cron_worker;
mod data;
mod infrastructure;
mod service;
mod shared;

use app::create_app;
use infrastructure::init_tracing;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize global tracing/logging as early as possible
    init_tracing();
    info!("application starting");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000")
        .await
        .inspect_err(|_| tracing::error!(port = 8000, "HTTP listener bind failed"))?;

    let (app, mut scheduler, dbos) = create_app().await;

    info!("Started server on port 8000...");
    let server_result = axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for Ctrl+C");
            info!("shutdown requested; draining HTTP requests");
        })
        .await;

    info!("stopping cleanup scheduler");
    scheduler
        .shutdown()
        .await
        .inspect_err(|_| tracing::error!("cleanup scheduler shutdown failed"))?;
    info!("stopping DBOS runtime");
    dbos.shutdown().await;
    if server_result.is_err() {
        tracing::error!("HTTP server stopped with an error");
    }
    server_result?;

    info!("application shutdown complete");
    Ok(())
}
