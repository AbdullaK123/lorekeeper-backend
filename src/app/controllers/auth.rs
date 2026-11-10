use crate::app::errors::AppError;
use crate::app::{AppState, UserId, ValidatedJson, ValidatedQuery};
use crate::data::{
    LoginRequest, PasswordResetParams, PasswordResetPayload, PasswordResetRequest, SignupRequest,
    UserResponse, VerifyEmailRequest,
};
use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde_json::json;

#[tracing::instrument(name = "controller.auth.login", level = "debug", skip_all)]
async fn login(
    State(ctx): State<AppState>,
    cookies: CookieJar,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> Result<(CookieJar, Json<UserResponse>), AppError> {
    let user = ctx
        .auth_service
        .authenticate_user(payload.email, payload.password.into_inner())
        .await?;

    let (token, expires_at) = ctx.auth_service.create_session(&user.id).await?;

    let expiry = time::OffsetDateTime::from_unix_timestamp(expires_at.timestamp())
        .map_err(|error| AppError::InternalError(error.to_string()))?;
    let cookie = Cookie::build(("session_id", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(ctx.is_prod)
        .expires(expiry)
        .build();

    tracing::info!(user_id = %user.id, expires_at=%expires_at, "login session established");
    Ok((cookies.add(cookie), Json(user.into())))
}

#[tracing::instrument(name = "controller.auth.logout", level = "debug", skip_all)]
async fn logout(
    cookies: CookieJar,
    UserId(user_id): UserId,
    State(ctx): State<AppState>,
) -> Result<(CookieJar, impl IntoResponse), AppError> {
    let cookie = cookies
        .get("session_id")
        .ok_or_else(|| AppError::AuthError("Invalid credentials".into()))?;

    ctx.auth_service
        .delete_session(cookie.value(), &user_id)
        .await?;

    tracing::info!(user_id = %user_id, "login session ended");
    Ok((
        cookies.remove(Cookie::build(("session_id", "")).path("/").build()),
        Json(json!({
            "message": "Successfully logged out."
        })),
    ))
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
    ValidatedJson(payload): ValidatedJson<PasswordResetRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    ctx.auth_service
        .send_password_reset_email(payload.email)
        .await?;
    Ok(Json(
        json!({ "message": "If an account exists, we will send a reset link." }),
    ))
}

#[tracing::instrument(name = "controller.auth.reset_password", level = "debug", skip_all)]
async fn reset_password(
    State(ctx): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<PasswordResetParams>,
    ValidatedJson(payload): ValidatedJson<PasswordResetPayload>,
) -> Result<Json<UserResponse>, AppError> {
    let user = ctx
        .auth_service
        .reset_password(
            params.user_id,
            params.token,
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

#[tracing::instrument(name = "controller.auth.begin_google_login", level = "debug", skip_all)]
async fn begin_google_login(
    cookies: CookieJar,
    State(ctx): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let (url, state) = ctx.auth_service.begin_google_login().await?;

    let cookie = Cookie::build(("oauth_state", state))
        .http_only(true)
        .path("/")
        .same_site(SameSite::Lax)
        .secure(ctx.is_prod)
        .max_age(time::Duration::minutes(10))
        .build();

    tracing::info!(provider = "google", "OAuth redirect prepared");
    Ok((cookies.add(cookie), Redirect::to(&url)))
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
        .route("/google", get(begin_google_login))
}
