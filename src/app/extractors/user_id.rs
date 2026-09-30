use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use tower_sessions::Session;
use uuid::Uuid;
use crate::app::errors::AppError;

pub struct UserId(pub Uuid);

impl <S> FromRequestParts<S> for UserId
where
    S: Send + Sync
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {

        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|e| AppError::AuthError(e.1.to_string()))?;

        let user_id: Option<Uuid> = session.get("user_id")
            .await
            .map_err(|e| AppError::DownstreamServiceError(e.to_string()))?;

        match user_id {
            Some(user_id) => Ok(UserId(user_id)),
            None => Err(AppError::AuthError("Invalid credentials".to_string()))
        }
    }
}