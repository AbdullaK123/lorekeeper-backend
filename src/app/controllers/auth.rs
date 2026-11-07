use crate::app::errors::AppError;
use crate::app::{AppState, UserId, ValidatedJson};
use crate::data::{
    LoginRequest, ResetPasswordRequest, SignupRequest, UserResponse, VerifyEmailRequest,
};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use tower_sessions::Session;

#[tracing::instrument(name = "controller.auth.login", level = "debug", skip_all)]
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

#[tracing::instrument(name = "controller.auth.logout", level = "debug", skip_all)]
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

#[tracing::instrument(name = "controller.auth.signup", level = "debug", skip_all)]
async fn signup(
    State(ctx): State<AppState>,
    ValidatedJson(payload): ValidatedJson<SignupRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = ctx.auth_service.signup(payload).await?;
    Ok(Json(user))
}

#[tracing::instrument(name = "controller.auth.get_profile", level = "debug", skip_all)]
async fn get_profile(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
) -> Result<Json<UserResponse>, AppError> {
    let user = ctx.auth_service.get_profile(user_id).await?;
    Ok(Json(user))
}

#[tracing::instrument(
    name = "controller.auth.request_password_reset",
    level = "debug",
    skip_all
)]
async fn request_password_reset(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
) -> Result<Json<serde_json::Value>, AppError> {
    ctx.auth_service.send_password_reset_email(user_id).await?;
    Ok(Json(json!({ "message": "Password reset email sent." })))
}

#[tracing::instrument(name = "controller.auth.reset_password", level = "debug", skip_all)]
async fn reset_password(
    State(ctx): State<AppState>,
    ValidatedJson(payload): ValidatedJson<ResetPasswordRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = ctx
        .auth_service
        .reset_password(
            payload.user_id,
            payload.token,
            payload.new_password.into_inner(),
        )
        .await?;
    Ok(Json(user))
}

#[tracing::instrument(
    name = "controller.auth.request_email_verification",
    level = "debug",
    skip_all
)]
async fn request_email_verification(
    State(ctx): State<AppState>,
    UserId(user_id): UserId,
) -> Result<Json<serde_json::Value>, AppError> {
    ctx.auth_service.send_verification_email(user_id).await?;
    Ok(Json(json!({ "message": "Verification email sent." })))
}

#[tracing::instrument(name = "controller.auth.verify_email", level = "debug", skip_all)]
async fn verify_email(
    State(ctx): State<AppState>,
    ValidatedJson(payload): ValidatedJson<VerifyEmailRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = ctx
        .auth_service
        .verify_user(payload.user_id, payload.token)
        .await?;
    Ok(Json(user))
}

pub fn create_auth_controller() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_profile))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/signup", post(signup))
        .route("/password_reset/request", post(request_password_reset))
        .route("/password_reset", post(reset_password))
        .route(
            "/email_verification/request",
            post(request_email_verification),
        )
        .route("/email_verification", post(verify_email))
}
