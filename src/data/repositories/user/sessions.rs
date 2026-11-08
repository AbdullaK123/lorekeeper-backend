use crate::{data::SessionRow, infrastructure::InfrastructureError};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SessionRepository {
    pool: PgPool,
}

impl SessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(name = "repository.session.create", level = "debug", skip_all, fields(user_id = %user_id))]
    pub async fn create(
        &self,
        token_hash: &str,
        user_id: &Uuid,
    ) -> Result<SessionRow, InfrastructureError> {
        Ok(sqlx::query_as!(
            SessionRow,
            "
            INSERT INTO sessions (token_hash, user_id)
            VALUES ($1, $2)
            RETURNING *
            ",
            token_hash,
            user_id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    #[tracing::instrument(name = "repository.session.get", level = "debug", skip_all)]
    pub async fn get(&self, token_hash: &str) -> Result<Option<SessionRow>, InfrastructureError> {
        Ok(sqlx::query_as!(
            SessionRow,
            "
            SELECT *
            FROM sessions
            WHERE token_hash=$1 AND expires_at > NOW()
            ",
            token_hash
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    #[tracing::instrument(name = "repository.session.delete", level = "debug", skip_all, fields(user_id = %user_id))]
    pub async fn delete(
        &self,
        token_hash: &str,
        user_id: &Uuid,
    ) -> Result<bool, InfrastructureError> {
        let result = sqlx::query!(
            "
            DELETE FROM sessions WHERE token_hash=$1 AND user_id=$2
            ",
            token_hash,
            user_id
        )
        .execute(&self.pool)
        .await?;
        tracing::debug!(
            rows_affected = result.rows_affected(),
            "session deletion query completed"
        );
        Ok(result.rows_affected() > 0)
    }

    #[tracing::instrument(name = "repository.session.delete_all", level = "debug", skip_all, fields(user_id = %user_id))]
    pub async fn delete_all(
        &self,
        user_id: &Uuid,
        connection: &mut PgConnection,
    ) -> Result<bool, InfrastructureError> {
        let result = sqlx::query!(
            "
            DELETE FROM sessions WHERE user_id=$1
            ",
            user_id
        )
        .execute(connection)
        .await?;
        tracing::debug!(
            rows_affected = result.rows_affected(),
            "session deletion query completed"
        );
        Ok(result.rows_affected() > 0)
    }
}
