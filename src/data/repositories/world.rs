use sqlx::PgPool;
use uuid::Uuid;
use crate::data::World;
use crate::infrastructure::InfrastructureError;

#[derive(Clone, Debug)]
pub struct WorldRepository {
    pool: PgPool
}

impl WorldRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }

    pub async fn get(
        &self,
        user_id: Uuid
    ) -> Result<Vec<World>, InfrastructureError> {
        let result = sqlx::query_as!(
            World,
            "SELECT * FROM worlds WHERE user_id=$1",
            user_id
        ).fetch_all(&self.pool).await?;
        Ok(result)
    }

    pub async fn get_by_id(
        &self,
        id: Uuid,
        user_id: Uuid
    ) -> Result<Option<World>, InfrastructureError> {
        let result = sqlx::query_as!(
            World,
            "
            SELECT * FROM worlds WHERE id=$1 AND user_id=$2
            ",
            id,
            user_id
        ).fetch_optional(&self.pool).await?;
        Ok(result)
    }

    pub async fn create(
        &self,
        name: String,
        description: String,
        user_id: Uuid
    ) -> Result<World, InfrastructureError> {
        let result = sqlx::query_as!(
            World,
            "
            INSERT INTO worlds (user_id, name, description)
            VALUES ($1, $2, $3)
            RETURNING *
            ",
            user_id,
            name,
            description
        ).fetch_one(&self.pool).await?;
        Ok(result)
    }

    pub async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        updated_name: Option<String>,
        updated_description: Option<String>
    ) -> Result<Option<World>, InfrastructureError> {
        let result = sqlx::query_as!(
            World,
            "
            UPDATE worlds
            SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                updated_at = NOW()
            WHERE id = $3 AND user_id = $4
            RETURNING *
            ",
            updated_name,
            updated_description,
            id,
            user_id
        ).fetch_optional(&self.pool).await?;
        Ok(result)
    }

    pub async fn delete(
        &self,
        id: Uuid,
        user_id: Uuid
    ) -> Result<bool, InfrastructureError> {
        let result = sqlx::query!(
            "
            DELETE FROM worlds
            WHERE id = $1 AND user_id = $2
            ",
            id,
            user_id
        ).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }
}
