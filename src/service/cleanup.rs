use crate::data::GraphRepository;
use crate::infrastructure::InfrastructureError;
use crate::service::ServiceError;
use sqlx::PgPool;

#[derive(Clone)]
pub struct CleanupService {
    pool: PgPool,
    graph_repo: GraphRepository,
}

impl CleanupService {
    pub fn new(pool: PgPool, graph_repo: GraphRepository) -> Self {
        Self { pool, graph_repo }
    }

    pub async fn run_once(&self) -> Result<(), ServiceError> {
        let pending_batch = sqlx::query!(
            "
            SELECT *
            FROM world_cleanup_jobs
            LIMIT $1
            ",
            20_i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(InfrastructureError::from)?;

        for pending in pending_batch {
            // Delete this world's graph data, then remove its cleanup job.
            let result = self
                .graph_repo
                .delete_graph(&pending.world_id.to_string())
                .await;

            if let Err(e) = &result {
                tracing::error!(world_id = %pending.world_id, error = %e, "graph cleanup failed");
            }

            if let Ok(()) = &result {
                let result = sqlx::query!(
                    "
                    DELETE FROM world_cleanup_jobs WHERE world_id=$1
                    ",
                    pending.world_id
                )
                .execute(&self.pool)
                .await
                .map_err(InfrastructureError::from);

                if let Err(e) = result {
                    tracing::error!(world_id = %pending.world_id, error = %e, "removing cleanup job failed");
                }
            }
        }

        Ok(())
    }
}
