//! Opt-in recovery HTTP checks. Email requests stay on a loopback capture server.
use super::*;
use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::State,
    http::Request,
    routing::post,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

type Captured = Arc<Mutex<Vec<Value>>>;

async fn capture_email(State(emails): State<Captured>, Json(email): Json<Value>) -> Json<Value> {
    emails.lock().unwrap().push(email);
    Json(json!({"id": "00000000-0000-4000-8000-000000000001"}))
}

async fn request(app: &Router, path: &str, body: Value) -> (u16, Value, Option<String>) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status().as_u16();
    let cookie = response
        .headers()
        .get("set-cookie")
        .map(|value| value.to_str().unwrap().to_owned());
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap(), cookie)
}

async fn with_cookie(
    app: &Router,
    method: &str,
    path: &str,
    cookie: &str,
) -> (u16, Option<String>) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    (
        response.status().as_u16(),
        response
            .headers()
            .get("set-cookie")
            .map(|value| value.to_str().unwrap().to_owned()),
    )
}

fn cookie_pair(header: &str) -> &str {
    header.split(';').next().unwrap()
}

fn verification_payload(emails: &Captured, email: &str, user_id: &str) -> Value {
    let captured = emails.lock().unwrap();
    let message = captured
        .last()
        .expect("Verification request sends no email");
    assert_eq!(
        message["to"][0], email,
        "Verification email goes to the wrong account"
    );
    let link = message["html"]
        .as_str()
        .unwrap()
        .split("href=\"")
        .find_map(|part| {
            part.split('"')
                .next()
                .filter(|url| url.contains("/auth/email_verification?"))
        })
        .expect("Verification email lacks a confirmation link")
        .replace("&#38;", "&")
        .replace("&amp;", "&");
    let base = std::env::var("API_BASE_URL").unwrap();
    assert!(
        link.starts_with(&format!(
            "{}/auth/email_verification?",
            base.trim_end_matches('/')
        )),
        "Verification link ignores the configured API base URL"
    );
    let uri: axum::http::Uri = link.parse().unwrap();
    let axum::extract::Query(params) =
        axum::extract::Query::<data::VerifyEmailRequest>::try_from_uri(&uri).unwrap();
    assert_eq!(
        params.user_id.to_string(),
        user_id,
        "Verification link names a different user"
    );
    json!({"user_id":params.user_id, "token":params.token})
}

async fn verify_email_flow(
    app: &Router,
    pool: &sqlx::PgPool,
    emails: &Captured,
    email: &str,
    user_id: &str,
    cookie: &str,
) {
    let initial = emails.lock().unwrap().len();
    let denied = request(app, "/auth/email_verification/request", json!({})).await;
    assert_eq!(
        denied.0, 401,
        "Logged-out callers can request verification email"
    );
    assert_eq!(
        emails.lock().unwrap().len(),
        initial,
        "An unauthorized request sends email"
    );
    let sent = with_cookie(app, "POST", "/auth/email_verification/request", cookie).await;
    assert_eq!(sent.0, 200, "A logged-in user cannot request verification");
    assert_eq!(
        emails.lock().unwrap().len(),
        initial + 1,
        "Verification sends an unexpected number of emails"
    );
    let expired = verification_payload(emails, email, user_id);
    let user_uuid = uuid::Uuid::parse_str(user_id).unwrap();
    sqlx::query("UPDATE auth_tokens SET created_at=NOW()-INTERVAL '2 hours', expired_at=NOW()-INTERVAL '1 hour' WHERE user_id=$1 AND purpose='email_verification'")
        .bind(user_uuid).execute(pool).await.unwrap();
    let rejected = request(app, "/auth/email_verification", expired).await;
    assert_eq!(rejected.0, 422, "An expired token verifies the account");
    let flag: bool = sqlx::query_scalar("SELECT verified FROM users WHERE id=$1")
        .bind(user_uuid)
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(
        !flag,
        "Expired-token rejection still changes the verification flag"
    );
    assert_eq!(
        with_cookie(app, "POST", "/auth/email_verification/request", cookie)
            .await
            .0,
        200,
        "An expired link prevents issuing a new one"
    );
    let valid = verification_payload(emails, email, user_id);
    let invalid = request(
        app,
        "/auth/email_verification",
        json!({"user_id":user_id,"token":"wrong-secret"}),
    )
    .await;
    assert_eq!(invalid.0, 422, "An invalid secret verifies an account");
    let wrong_user = request(
        app,
        "/auth/email_verification",
        json!({"user_id":uuid::Uuid::new_v4(),"token":valid["token"]}),
    )
    .await;
    assert_eq!(
        wrong_user.0, 422,
        "Verification token authorizes a different account"
    );
    let verified = request(app, "/auth/email_verification", valid.clone()).await;
    assert_eq!(
        verified.0, 200,
        "Valid emailed credentials cannot confirm without a session"
    );
    assert_eq!(
        verified.1["verified"], true,
        "Confirmation response does not report verification"
    );
    assert_eq!(
        verified.1["id"], user_id,
        "Verification response names a different account"
    );
    assert!(
        verified.2.is_none(),
        "Email confirmation creates a login session"
    );
    let saved: bool = sqlx::query_scalar("SELECT verified FROM users WHERE id=$1")
        .bind(user_uuid)
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(
        saved,
        "Verification success does not persist the account flag"
    );
    let replay = request(app, "/auth/email_verification", valid).await;
    assert_eq!(replay.0, 422, "A verification token can be consumed twice");
    let count = emails.lock().unwrap().len();
    let repeat = with_cookie(app, "POST", "/auth/email_verification/request", cookie).await;
    assert_eq!(
        repeat.0, 409,
        "Already verified accounts can issue more verification links"
    );
    assert_eq!(
        emails.lock().unwrap().len(),
        count,
        "Already verified accounts still trigger delivery"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires isolated QA Postgres, Memgraph and RESEND_BASE_URL=http://127.0.0.1:18082; no real email/model calls"]
async fn recovery_http_requires_valid_input_and_consumes_links_once() {
    let database = std::env::var("DATABASE_URL").unwrap();
    assert!(
        database.ends_with("/lorekeeper_adversarial"),
        "Recovery tests must not modify development users"
    );
    assert_eq!(
        std::env::var("RESEND_BASE_URL").unwrap(),
        "http://127.0.0.1:18082",
        "Email traffic must stay on the local capture server"
    );
    assert_eq!(
        std::env::var("RESEND_API_KEY").unwrap(),
        "re_test_not_sent",
        "Recovery tests must not use real credentials"
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:18082")
        .await
        .unwrap();
    let emails: Captured = Arc::default();
    let capture = Router::new()
        .route("/emails", post(capture_email))
        .with_state(emails.clone());
    let server = tokio::spawn(async move {
        axum::serve(listener, capture).await.unwrap();
    });
    let (app, mut scheduler, dbos) = app::create_app().await;
    let pool = sqlx::PgPool::connect(&database).await.unwrap();
    let suffix = uuid::Uuid::new_v4();
    let email = format!("recovery-{suffix}@example.com");
    let old_password = "Original-Password-982!";
    let new_password = "Changed-Password-671!";
    let created = request(
        &app,
        "/auth/signup",
        json!({"username":"Recovery QA", "email":email, "password":old_password}),
    )
    .await;
    assert_eq!(
        created.0, 200,
        "Recovery fixture cannot sign up through the real route"
    );
    let user_id = created.1["id"].as_str().unwrap().to_owned();

    let invalid = request(
        &app,
        "/auth/password_reset/request",
        json!({"email":"not-an-email"}),
    )
    .await;
    assert_eq!(invalid.0, 422, "Invalid emails reach the recovery sender");
    let unknown = request(
        &app,
        "/auth/password_reset/request",
        json!({"email":format!("missing-{suffix}@example.com")}),
    )
    .await;
    assert_eq!(
        unknown.0, 200,
        "Unknown accounts reveal their absence through an error status"
    );
    assert!(
        emails.lock().unwrap().is_empty(),
        "Invalid or unknown addresses trigger email delivery"
    );
    let known = request(&app, "/auth/password_reset/request", json!({"email":email})).await;
    assert_eq!(known.0, 200, "A logged-out user cannot request recovery");
    assert_eq!(
        known.1, unknown.1,
        "Recovery responses reveal whether an account exists"
    );
    let captured = emails.lock().unwrap().clone();
    assert_eq!(
        captured.len(),
        1,
        "A valid recovery request does not send exactly one email"
    );
    assert_eq!(
        captured[0]["to"][0], email,
        "The recovery link goes to the wrong account"
    );
    let html = captured[0]["html"].as_str().unwrap();
    let link = html
        .split("href=\"")
        .find_map(|part| {
            part.split('"')
                .next()
                .filter(|url| url.contains("/auth/password_reset?"))
        })
        .expect("Reset email lacks a usable link")
        .replace("&#38;", "&")
        .replace("&amp;", "&");
    let base = std::env::var("API_BASE_URL").unwrap();
    let path = link
        .strip_prefix(base.trim_end_matches('/'))
        .expect("Reset link ignores API_BASE_URL");
    assert!(
        path.contains(&format!("user_id={user_id}")),
        "Reset link names the wrong user"
    );

    let missing = request(
        &app,
        "/auth/password_reset",
        json!({"new_password":new_password}),
    )
    .await;
    assert_eq!(
        missing.0, 422,
        "Confirmation accepts missing query credentials"
    );
    let weak = request(&app, path, json!({"new_password":"weak"})).await;
    assert_eq!(
        weak.0, 422,
        "Confirmation bypasses the signup password policy"
    );
    let wrong = request(
        &app,
        &format!("/auth/password_reset?user_id={user_id}&token=wrong"),
        json!({"new_password":new_password}),
    )
    .await;
    assert_eq!(
        wrong.0, 422,
        "An invalid secret authorizes a password reset"
    );
    let first = request(
        &app,
        "/auth/login",
        json!({"email":email,"password":old_password}),
    )
    .await;
    let second = request(
        &app,
        "/auth/login",
        json!({"email":email,"password":old_password}),
    )
    .await;
    assert_eq!(first.0, 200, "First device cannot log in before reset");
    assert_eq!(second.0, 200, "Second device cannot log in before reset");
    let first_header = first.2.as_deref().expect("Login does not return a cookie");
    let second_header = second
        .2
        .as_deref()
        .expect("Second login does not return a cookie");
    let first_cookie = cookie_pair(first_header);
    let second_cookie = cookie_pair(second_header);
    let parsed = axum_extra::extract::cookie::Cookie::parse(first_header).unwrap();
    assert_eq!(
        parsed.path(),
        Some("/"),
        "Login cookie cannot reach all protected routes"
    );
    assert_eq!(
        parsed.http_only(),
        Some(true),
        "Browser scripts can read session credentials"
    );
    assert_eq!(
        parsed.same_site(),
        Some(axum_extra::extract::cookie::SameSite::Lax),
        "Login cookie lacks the selected SameSite policy"
    );
    assert_eq!(
        parsed.secure() == Some(true),
        std::env::var("ENV").unwrap() == "prod",
        "Cookie Secure flag differs from the configured environment"
    );
    let stored: (String, chrono::DateTime<chrono::Utc>) =
        sqlx::query_as("SELECT token_hash, expires_at FROM sessions WHERE token_hash=$1")
            .bind(hex::encode(<sha2::Sha256 as sha2::Digest>::digest(
                parsed.value().as_bytes(),
            )))
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_ne!(
        stored.0,
        parsed.value(),
        "Database stores the usable session secret"
    );
    assert_eq!(
        parsed.expires_datetime().unwrap().unix_timestamp(),
        stored.1.timestamp(),
        "Browser expiry differs from database expiry"
    );
    assert_eq!(
        with_cookie(&app, "GET", "/auth/me", first_cookie).await.0,
        200,
        "First session cannot resolve the profile"
    );
    assert_eq!(
        with_cookie(&app, "GET", "/auth/me", second_cookie).await.0,
        200,
        "Second session cannot resolve the profile"
    );
    assert_eq!(
        with_cookie(&app, "GET", "/auth/me", "session_id=forged-token")
            .await
            .0,
        401,
        "Unknown cookies grant account access"
    );

    let expired_login = request(
        &app,
        "/auth/login",
        json!({"email":email,"password":old_password}),
    )
    .await;
    let expired_header = expired_login.2.as_deref().unwrap();
    let expired_cookie = axum_extra::extract::cookie::Cookie::parse(expired_header).unwrap();
    sqlx::query("UPDATE sessions SET expires_at=NOW()-INTERVAL '1 minute' WHERE token_hash=$1")
        .bind(hex::encode(<sha2::Sha256 as sha2::Digest>::digest(
            expired_cookie.value().as_bytes(),
        )))
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        with_cookie(&app, "GET", "/auth/me", cookie_pair(expired_header))
            .await
            .0,
        401,
        "An expired session still grants account access"
    );

    let logout_login = request(
        &app,
        "/auth/login",
        json!({"email":email,"password":old_password}),
    )
    .await;
    let logout_cookie = cookie_pair(logout_login.2.as_deref().unwrap());
    let logged_out = with_cookie(&app, "POST", "/auth/logout", logout_cookie).await;
    assert_eq!(logged_out.0, 200, "Logout fails to delete a valid session");
    let removed = axum_extra::extract::cookie::Cookie::parse(logged_out.1.unwrap()).unwrap();
    assert_eq!(
        removed.path(),
        Some("/"),
        "Logout clears the wrong cookie path"
    );
    assert!(
        removed.value().is_empty(),
        "Logout leaves the browser token populated"
    );
    assert_eq!(
        removed.max_age().unwrap().whole_seconds(),
        0,
        "Logout does not expire the cookie"
    );
    assert_eq!(
        with_cookie(&app, "GET", "/auth/me", logout_cookie).await.0,
        401,
        "A copied cookie survives logout"
    );
    assert_eq!(
        with_cookie(&app, "GET", "/auth/me", first_cookie).await.0,
        200,
        "Logout revokes another device's session"
    );

    let reset = request(&app, path, json!({"new_password":new_password})).await;
    assert_eq!(
        reset.0, 200,
        "The emailed query credentials and JSON password cannot confirm recovery: {}",
        reset.1
    );
    assert_eq!(
        reset.1["id"], user_id,
        "Confirmation returns a different account"
    );
    assert!(
        reset.2.is_none(),
        "Password reset unexpectedly creates a login session"
    );
    assert_eq!(
        with_cookie(&app, "GET", "/auth/me", first_cookie).await.0,
        401,
        "First device stays logged in after password reset"
    );
    assert_eq!(
        with_cookie(&app, "GET", "/auth/me", second_cookie).await.0,
        401,
        "Second device stays logged in after password reset"
    );
    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions WHERE user_id=$1")
        .bind(uuid::Uuid::parse_str(&user_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        remaining, 0,
        "Password reset leaves database sessions for the user"
    );
    let old_login = request(
        &app,
        "/auth/login",
        json!({"email":email,"password":old_password}),
    )
    .await;
    assert_eq!(
        old_login.0, 401,
        "The old password still authenticates after reset"
    );
    let new_login = request(
        &app,
        "/auth/login",
        json!({"email":email,"password":new_password}),
    )
    .await;
    assert_eq!(
        new_login.0, 200,
        "The confirmed password cannot authenticate"
    );
    assert!(
        new_login.2.is_some(),
        "Successful login does not establish a session"
    );
    let replay = request(&app, path, json!({"new_password":"Replay-Password-713!"})).await;
    assert_eq!(
        replay.0, 422,
        "An emailed reset link can change the password twice"
    );
    let after = request(
        &app,
        "/auth/login",
        json!({"email":email,"password":new_password}),
    )
    .await;
    assert_eq!(
        after.0, 200,
        "Replay rejection still changes the saved password"
    );

    let active_cookie = cookie_pair(after.2.as_deref().unwrap());
    verify_email_flow(&app, &pool, &emails, &email, &user_id, active_cookie).await;

    sqlx::query("DELETE FROM users WHERE id=$1")
        .bind(uuid::Uuid::parse_str(&user_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    scheduler.shutdown().await.unwrap();
    dbos.shutdown().await;
    server.abort();
    pool.close().await;
}
