//! Real auth router checks with local storage and no provider discovery.
use super::*;
use axum::{body::Body, http::Request};
use openidconnect::core::CoreProviderMetadata;
use serde_json::json;
use tower::ServiceExt;

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
