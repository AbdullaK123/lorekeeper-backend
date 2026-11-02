use crate::data::GraphRepository;
use crate::service::CleanupService;
use sqlx::PgPool;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

pub async fn start_scheduler(
    pool: PgPool,
    graph_repo: &GraphRepository,
) -> Result<JobScheduler, JobSchedulerError> {
    let scheduler = JobScheduler::new().await?;

    let cleanup_service = CleanupService::new(pool, graph_repo.clone());

    let job = Job::new_async("*/5 * * * * *", move |_, _| {
        let cleanup_service = cleanup_service.clone();
        Box::pin(async move {
            if cleanup_service.run_once().await.is_err() {
                tracing::error!("world cleanup batch failed");
            }
        })
    })?;

    scheduler.add(job).await?;
    scheduler.start().await?;
    tracing::info!(interval_seconds = 5, "world cleanup scheduler started");

    Ok(scheduler)
}
