use crate::data::models::User;
use crate::infrastructure::InfrastructureError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(name = "repository.user.sign_up", level = "debug", skip_all)]
    pub async fn sign_up(
        &self,
        username: String,
        email: String,
        password_hash: String,
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
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            if error.as_database_error().is_some_and(|error| {
                error.is_unique_violation() && error.constraint() == Some("users_email_key")
            }) {
                InfrastructureError::Conflict("An account with this email already exists".into())
            } else {
                error.into()
            }
        })?;

        Ok(result)
    }

    #[tracing::instrument(name = "repository.user.get_user_by_email", level = "debug", skip_all)]
    pub async fn get_user_by_email(
        &self,
        email: String,
    ) -> Result<Option<User>, InfrastructureError> {
        let result = sqlx::query_as!(
            User,
            "
            SELECT *
            FROM users
            WHERE email = $1
            ",
            email
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    #[tracing::instrument(name = "repository.user.get_user_by_id", level = "debug", skip_all)]
    pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, InfrastructureError> {
        let result = sqlx::query_as!(
            User,
            "
            SELECT  *
            FROM users
            WHERE id = $1
            ",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(result)
    }

    #[tracing::instrument(name = "repository.user.update_password", level = "debug", skip_all, fields(user_id = %id))]
    pub async fn update_password(
        &self,
        id: Uuid,
        new_password_hash: String,
        connection: &mut sqlx::PgConnection,
    ) -> Result<Option<User>, InfrastructureError> {
        Ok(sqlx::query_as!(
            User,
            "
            UPDATE users
            SET password_hash = $2
            WHERE id = $1
            RETURNING *
            ",
            id,
            new_password_hash
        )
        .fetch_optional(connection)
        .await?)
    }

    #[tracing::instrument(name = "repository.user.verify_user", level = "debug", skip_all, fields(user_id = %id))]
    pub async fn verify_user(
        &self,
        id: Uuid,
        connection: &mut sqlx::PgConnection,
    ) -> Result<Option<User>, InfrastructureError> {
        Ok(sqlx::query_as!(
            User,
            "
            UPDATE users
            SET verified=TRUE
            WHERE id=$1
            RETURNING *
            ",
            id
        )
        .fetch_optional(connection)
        .await?)
    }
}
