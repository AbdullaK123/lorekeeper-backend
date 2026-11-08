use crate::app::AppState;
use crate::app::errors::AppError;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::cookie::CookieJar;
use uuid::Uuid;

pub struct UserId(pub Uuid);

impl FromRequestParts<AppState> for UserId {
    type Rejection = AppError;

    #[tracing::instrument(name = "extractor.user_id", level = "debug", skip_all)]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies = CookieJar::from_headers(&parts.headers);

        let cookie = cookies.get("session_id").ok_or_else(|| {
            tracing::debug!(reason = "missing_cookie", "authentication rejected");
            AppError::AuthError("Invalid credentials".into())
        })?;

        let user_id: Option<Uuid> = state
            .auth_service
            .get_user_id_from_session(cookie.value())
            .await?;

        match user_id {
            Some(user_id) => Ok(UserId(user_id)),
            None => Err(AppError::AuthError("Invalid credentials".to_string())),
        }
    }
}
