//! Real auth router checks with local storage and no provider discovery.
use super::*;
use axum::{body::Body, http::Request};
use openidconnect::core::CoreProviderMetadata;
use serde_json::json;
use tower::ServiceExt;
use sha2::{Digest, Sha256};

async fn router() -> Router {
    let url = std::env::var("DATABASE_URL").unwrap();
    assert!(
        url.ends_with("/lorekeeper_adversarial"),
        "Callback tests must use isolated QA storage"
    );
    assert_eq!(
        std::env::var("OPENROUTER_API_KEY").unwrap(),
        "qa-not-sent",
        "Unused model services must not carry real credentials"
    );
    let pool = PgPool::connect(&url).await.unwrap();
    let graph = Graph::new("bolt://127.0.0.1:7687", "", "").await.unwrap();
    let graph_repo = GraphRepository::new(graph.clone());
    let provider = openrouter::from_env().unwrap();
    let model = provider.chat("unused-test-model");
    let embedding: DynModel<Embedding> = provider.embedding("unused-test-embedding", None).into();
    let retrieval = RetrievalService::new(graph_repo.clone(), embedding);
    let agent = ChatAgentFactory::new(
        model.into(),
        "unused".into(),
        retrieval.clone(),
        graph_repo.clone(),
    );
    let dbos = DBOS::new(dbos::Config::new("qa-oauth-callback", &url));
    let research = dbos
        .register_workflow(
            "unused-research",
            |_input: crate::service::ResearchInput| async {
                Ok::<_, dbos::Error<crate::service::ServiceError>>(String::new())
            },
        )
        .unwrap();
    // DBOS is never launched: auth requests must not start model work.
    let dbos_client = dbos::Client::connect(dbos::ClientConfig::new(&url))
        .await
        .unwrap();
    let metadata: CoreProviderMetadata = serde_json::from_value(json!({
        "issuer":"http://127.0.0.1:9", "authorization_endpoint":"http://127.0.0.1:9/authorize",
        "token_endpoint":"http://127.0.0.1:9/token", "jwks_uri":"http://127.0.0.1:9/jwks",
        "response_types_supported":["code"], "subject_types_supported":["public"], "id_token_signing_alg_values_supported":["HS256"]
    })).unwrap();
    let oauth = openidconnect::core::CoreClient::from_provider_metadata(
        metadata,
        openidconnect::ClientId::new("qa-client".into()),
        Some(openidconnect::ClientSecret::new("qa-secret".into())),
    );
    let http = openidconnect::reqwest::Client::builder()
        .redirect(openidconnect::reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let world_repo = WorldRepository::new(pool.clone());
    let state = AppState {
        is_prod: false,
        pool: pool.clone(),
        graph,
        auth_service: AuthService::new(
            UserRepository::new(pool.clone()),
            "re_test_not_sent",
            "http://127.0.0.1:9",
            oauth,
            http,
        ),
        world_service: WorldService::new(world_repo.clone(), graph_repo),
        conversation_service: ConversationService::new(
            ConversationRepository::new(pool),
            world_repo,
            retrieval,
            agent,
            research,
            dbos_client,
        ),
    };
    Router::new()
        .nest("/auth", create_auth_controller())
        .with_state(state)
}

async fn callback(app: Router, query: &str, cookie: &str) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .uri(format!("/auth/google/callback?{query}"))
            .header("cookie", cookie)
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap()
}

#[tokio::test]
#[ignore = "requires QA Postgres/Memgraph; dummy OPENROUTER_API_KEY; no provider or email calls"]
async fn google_cancellation_clears_state_without_creating_a_session() {
    let app = router().await;
    let response = callback(
        app,
        "state=qa-state&error=access_denied",
        "oauth_state=qa-state",
    )
    .await;
    assert_eq!(
        response.status(),
        401,
        "Google cancellation reaches a successful callback"
    );
    let cookies: Vec<_> = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|value| value.to_str().unwrap())
        .collect();
    assert!(
        cookies
            .iter()
            .any(|cookie| cookie.starts_with("oauth_state=")
                && cookie.contains("Max-Age=0")
                && cookie.contains("Path=/")),
        "Cancellation leaves the temporary state cookie behind"
    );
    assert!(
        !cookies
            .iter()
            .any(|cookie| cookie.starts_with("session_id=")),
        "Cancellation creates a login session"
    );
}

#[tokio::test]
#[ignore = "requires QA Postgres/Memgraph; dummy OPENROUTER_API_KEY; no provider or email calls"]
async fn google_callback_rejects_a_different_browser_state() {
    let response = callback(
        router().await,
        "state=other-browser&error=access_denied",
        "oauth_state=qa-state",
    )
    .await;
    assert_eq!(
        response.status(),
        401,
        "A callback from another browser is accepted"
    );
    assert!(
        !response
            .headers()
            .get_all("set-cookie")
            .iter()
            .any(|cookie| cookie.to_str().unwrap().starts_with("session_id=")),
        "Mismatched state creates a login session"
    );
}

#[tokio::test]
#[ignore = "requires QA Postgres/Memgraph; dummy OPENROUTER_API_KEY; no provider or email calls"]
async fn google_missing_attempt_failure_clears_the_state_cookie() {
    let state = uuid::Uuid::new_v4().to_string();
    let response = callback(
        router().await,
        &format!("state={state}&code=qa-code"),
        &format!("oauth_state={state}"),
    )
    .await;
    assert_eq!(
        response.status(),
        401,
        "A callback without a saved attempt authenticates"
    );
    assert!(
        response
            .headers()
            .get_all("set-cookie")
            .iter()
            .any(|cookie| {
                let cookie = cookie.to_str().unwrap();
                cookie.starts_with("oauth_state=") && cookie.contains("Max-Age=0")
            }),
        "Failed callback leaves a stale OAuth state cookie in the browser"
    );
}

// ---- Account linking through the real routes (US-043) ----

async fn qa_pool() -> PgPool {
    let url = std::env::var("DATABASE_URL").unwrap();
    assert!(url.ends_with("/lorekeeper_adversarial"), "Link route tests must use isolated QA storage");
    PgPool::connect(&url).await.unwrap()
}

/// A password user with a live session. Returns (user id, raw session token).
async fn logged_in_user(pool: &PgPool) -> (uuid::Uuid, String) {
    let tag = uuid::Uuid::new_v4();
    let id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO users (username, email, password_hash, verified) VALUES ('Local', $1, 'qa-hash', TRUE) RETURNING id",
    )
    .bind(format!("route-{tag}@example.com"))
    .fetch_one(pool)
    .await
    .unwrap();
    let token = format!("qa-session-{tag}");
    sqlx::query("INSERT INTO sessions (token_hash, user_id) VALUES ($1, $2)")
        .bind(hex::encode(Sha256::digest(token.as_bytes())))
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    (id, token)
}

async fn delete_users(pool: &PgPool, ids: &[uuid::Uuid]) {
    sqlx::query("DELETE FROM users WHERE id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await
        .unwrap();
}

/// Start Google sign-in. Returns the response and the saved attempt's user_id (outer None = no attempt saved).
async fn begin(pool: &PgPool, cookie: Option<&str>) -> (axum::response::Response, Option<Option<uuid::Uuid>>) {
    let mut request = Request::builder().uri("/auth/google");
    if let Some(cookie) = cookie {
        request = request.header("cookie", cookie);
    }
    let response = router()
        .await
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let state = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find_map(|cookie| cookie.strip_prefix("oauth_state="))
        .map(|rest| rest.split(';').next().unwrap().to_owned());
    let saved = match state {
        Some(state) => {
            let hash = hex::encode(Sha256::digest(state.as_bytes()));
            let row: Option<Option<uuid::Uuid>> = sqlx::query_scalar(
                "DELETE FROM google_login_attempts WHERE state_hash=$1 RETURNING user_id",
            )
            .bind(hash)
            .fetch_optional(pool)
            .await
            .unwrap();
            row
        }
        None => None,
    };
    (response, saved)
}

#[tokio::test]
#[ignore = "requires QA Postgres/Memgraph; dummy OPENROUTER_API_KEY; no provider or email calls"]
async fn google_start_while_logged_in_forgets_who_is_linking() {
    let pool = qa_pool().await;
    let (user, token) = logged_in_user(&pool).await;
    let (response, saved) = begin(&pool, Some(&format!("session_id={token}"))).await;
    delete_users(&pool, &[user]).await;
    assert!(response.status().is_redirection(), "Logged-in Google start does not redirect to Google");
    assert_eq!(
        saved,
        Some(Some(user)),
        "The link attempt does not record the logged-in user; the callback cannot link"
    );
}

#[tokio::test]
#[ignore = "requires QA Postgres/Memgraph; dummy OPENROUTER_API_KEY; no provider or email calls"]
async fn google_start_with_a_dead_session_blocks_plain_login() {
    let pool = qa_pool().await;
    let (response, saved) = begin(&pool, Some("session_id=qa-session-that-does-not-exist")).await;
    assert!(
        response.status().is_redirection(),
        "A stale session cookie stops the writer from signing in with Google"
    );
    assert_eq!(saved, Some(None), "A stale session cookie turns a plain login into a link");
}

/// Fault injection: the sessions table is briefly renamed, so only the session lookup fails.
/// (The QA role is a superuser, so revoking privileges would not fail anything.)
/// Swallowing that error turns a logged-in writer's link into a plain login,
/// which can sign them into a brand-new empty account.
#[tokio::test]
#[ignore = "requires QA Postgres/Memgraph; dummy OPENROUTER_API_KEY; no provider or email calls"]
async fn google_start_hides_a_failed_session_lookup() {
    let pool = qa_pool().await;
    let (user, token) = logged_in_user(&pool).await;
    sqlx::query("ALTER TABLE sessions RENAME TO sessions_qa_offline")
        .execute(&pool)
        .await
        .unwrap();
    let (response, saved) = begin(&pool, Some(&format!("session_id={token}"))).await;
    sqlx::query("ALTER TABLE sessions_qa_offline RENAME TO sessions")
        .execute(&pool)
        .await
        .unwrap();
    delete_users(&pool, &[user]).await;
    assert!(
        response.status().is_server_error(),
        "A failed session lookup is treated as 'logged out' (got {}); the link silently becomes a login",
        response.status()
    );
    assert_eq!(saved, None, "A failed session lookup still saves a plain login attempt");
}

#[tokio::test]
#[ignore = "requires QA Postgres/Memgraph; dummy OPENROUTER_API_KEY; no provider or email calls"]
async fn google_callback_ignores_the_session_for_link_attempts() {
    let pool = qa_pool().await;
    let (owner, _) = logged_in_user(&pool).await;
    let (other, other_token) = logged_in_user(&pool).await;
    let state = format!("qa-link-{}", uuid::Uuid::new_v4());
    let state_hash = hex::encode(Sha256::digest(state.as_bytes()));
    sqlx::query(
        "INSERT INTO google_login_attempts (state_hash, nonce, pkce_verifier, user_id) VALUES ($1, 'n', 'v', $2)",
    )
    .bind(&state_hash)
    .bind(owner)
    .execute(&pool)
    .await
    .unwrap();
    // The token server (127.0.0.1:9) is unreachable: passing the attempt gate would give 500.
    let response = callback(
        router().await,
        &format!("state={state}&code=qa-code"),
        &format!("oauth_state={state}; session_id={other_token}"),
    )
    .await;
    let still_pending: i64 =
        sqlx::query_scalar("SELECT count(*) FROM google_login_attempts WHERE state_hash=$1")
            .bind(&state_hash)
            .fetch_one(&pool)
            .await
            .unwrap();
    delete_users(&pool, &[owner, other]).await;
    assert_eq!(
        response.status(),
        401,
        "The callback does not pass the session through; another user's browser reaches the token exchange"
    );
    assert_eq!(still_pending, 1, "A rejected callback burns the real owner's link attempt");
}
