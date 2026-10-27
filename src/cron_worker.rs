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
            if let Err(error) = cleanup_service.run_once().await {
                tracing::error!("world cleanup failed: {error}");
            }
        })
    })?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    Ok(scheduler)
}
