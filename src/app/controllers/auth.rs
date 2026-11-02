use crate::app::errors::AppError;
use crate::app::{AppState, UserId, ValidatedJson};
use crate::data::{LoginRequest, SignupRequest, UserResponse};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use tower_sessions::Session;

async fn login(
    State(ctx): State<AppState>,
    session: Session,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = ctx
        .auth_service
        .authenticate_user(payload.email, payload.password.into_inner())
        .await?;

    session
        .cycle_id()
        .await
        .map_err(|e| AppError::DownstreamServiceError(e.to_string()))?;

    session
        .insert("user_id", user.id)
        .await
        .map_err(|e| AppError::DownstreamServiceError(e.to_string()))?;

    tracing::info!(user_id = %user.id, "login session established");
    Ok(Json(user.into()))
}

async fn logout(session: Session) -> Result<impl IntoResponse, AppError> {
    session
        .flush()
        .await
        .map_err(|e| AppError::DownstreamServiceError(e.to_string()))?;
    tracing::info!("login session ended");
    Ok(Json(json!({
        "message": "Successfully logged out."
    })))
}

async fn signup(
    State(ctx): State<AppState>,
    ValidatedJson(payload): ValidatedJson<SignupRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = ctx.auth_service.signup(payload).await?;
    Ok(Json(user))
}

async fn get_profile(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
) -> Result<Json<UserResponse>, AppError> {
    let user = ctx.auth_service.get_profile(user_id).await?;
    Ok(Json(user))
}

pub fn create_auth_controller() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_profile))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/signup", post(signup))
}
