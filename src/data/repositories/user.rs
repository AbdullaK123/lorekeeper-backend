use sqlx::PgPool;
use uuid::Uuid;
use crate::data::models::User;
use crate::infrastructure::InfrastructureError;

#[derive(Clone, Debug)]
pub struct UserRepository {
    pool: PgPool
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }

    pub async fn sign_up(
        &self,
        username: String,
        email: String,
        password_hash: String
    ) -> Result<User, InfrastructureError> {
        let result = sqlx::query_as!(
            User,
            "
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING *
            ",
            username,
            email,
            password_hash
        ).fetch_one(&self.pool).await?;

        Ok(result)
    }

    pub async fn get_user_by_email(
        &self,
        email: String
    ) -> Result<Option<User>, InfrastructureError> {
        let result = sqlx::query_as!(
            User,
            "
            SELECT *
            FROM users
            WHERE email = $1
            ",
            email
        ).fetch_optional(&self.pool).await?;
        Ok(result)
    }


    pub async fn get_user_by_id(
        &self,
        id: Uuid
    ) -> Result<Option<User>, InfrastructureError> {
        let result = sqlx::query_as!(
            User,
            "
            SELECT  *
            FROM users
            WHERE id = $1
            ",
            id
        ).fetch_optional(&self.pool).await?;
        Ok(result)
    }



}