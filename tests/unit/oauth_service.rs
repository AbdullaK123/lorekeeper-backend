//! Local token-server tests. No Google discovery, email, or paid calls.
use super::*;
use axum::{Form, Json, Router, extract::State, routing::post};
use openidconnect::core::{
    CoreHmacKey, CoreIdToken, CoreIdTokenClaims, CoreJwsSigningAlgorithm, CoreProviderMetadata,
};
use openidconnect::{
    Audience, EmptyAdditionalClaims, EndUserEmail, IssuerUrl, StandardClaims, SubjectIdentifier,
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct Reply {
    token: String,
    verifier: String,
    calls: usize,
}

async fn token_endpoint(
    State(reply): State<Arc<Mutex<Reply>>>,
    Form(form): Form<HashMap<String, String>>,
) -> Json<Value> {
    let mut reply = reply.lock().unwrap();
    assert_eq!(
        form.get("code").map(String::as_str),
        Some("qa-code"),
        "The service exchanges the wrong authorization code"
    );
    assert_eq!(
        form.get("code_verifier"),
        Some(&reply.verifier),
        "Token exchange loses the saved PKCE verifier"
    );
    reply.calls += 1;
    Json(
        json!({"access_token":"qa-access", "token_type":"Bearer", "id_token":reply.token, "expires_in":300}),
    )
}

struct Fixture {
    auth: AuthService,
    pool: sqlx::PgPool,
    reply: Arc<Mutex<Reply>>,
    server: tokio::task::JoinHandle<()>,
    issuer: String,
    sub: String,
    email: String,
    state_hash: String,
}

impl Fixture {
    async fn new() -> Self {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
        assert!(
            url.ends_with("/lorekeeper_adversarial"),
            "OAuth tests must only use the isolated QA database"
        );
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let issuer = format!("http://{}", listener.local_addr().unwrap());
        let reply = Arc::new(Mutex::new(Reply::default()));
        let router = Router::new()
            .route("/token", post(token_endpoint))
            .with_state(reply.clone());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let metadata: CoreProviderMetadata = serde_json::from_value(json!({
            "issuer":issuer, "authorization_endpoint":format!("{issuer}/authorize"),
            "token_endpoint":format!("{issuer}/token"), "jwks_uri":format!("{issuer}/jwks"),
            "response_types_supported":["code"], "subject_types_supported":["public"],
            "id_token_signing_alg_values_supported":["HS256"]
        }))
        .unwrap();
        let client = openidconnect::core::CoreClient::from_provider_metadata(
            metadata,
            openidconnect::ClientId::new("qa-client".into()),
            Some(openidconnect::ClientSecret::new("qa-signing-secret".into())),
        )
        .set_redirect_uri(openidconnect::RedirectUrl::new(format!("{issuer}/callback")).unwrap());
        let http = openidconnect::reqwest::Client::builder()
            .redirect(openidconnect::reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let auth = AuthService::new(
            UserRepository::new(pool.clone()),
            "re_test_not_sent",
            &issuer,
            client,
            http,
        );
        let sub = uuid::Uuid::new_v4().to_string();
        Self {
            auth,
            pool,
            reply,
            server,
            issuer,
            email: format!("oauth-{sub}@example.com"),
            sub,
            state_hash: String::new(),
        }
    }

    async fn attempt(&mut self, mutation: &str) -> String {
        let (url, state) = self.auth.begin_google_login().await.unwrap();
        self.state_hash = hex::encode(Sha256::digest(state.as_bytes()));
        let (nonce, verifier): (String, String) = sqlx::query_as(
            "SELECT nonce, pkce_verifier FROM google_login_attempts WHERE state_hash=$1",
        )
        .bind(&self.state_hash)
        .fetch_one(&self.pool)
        .await
        .unwrap();
        let parsed = openidconnect::reqwest::Url::parse(&url).unwrap();
        let query: HashMap<_, _> = parsed.query_pairs().into_owned().collect();
        assert_eq!(
            query.get("state"),
            Some(&state),
            "Authorization URL loses browser state"
        );
        assert_eq!(
            query.get("nonce"),
            Some(&nonce),
            "Authorization URL nonce differs from the saved attempt"
        );
        assert_eq!(
            query.get("code_challenge_method").map(String::as_str),
            Some("S256"),
            "Authorization URL omits SHA-256 PKCE"
        );
        let expected =
            PkceCodeChallenge::from_code_verifier_sha256(&PkceCodeVerifier::new(verifier.clone()));
        assert_eq!(
            query.get("code_challenge").map(String::as_str),
            Some(expected.as_str()),
            "Authorization URL challenge differs from the stored verifier"
        );
        let mut standard = StandardClaims::new(SubjectIdentifier::new(self.sub.clone()))
            .set_email_verified(Some(mutation != "unverified"));
        if mutation != "missing_email" {
            standard = standard.set_email(Some(EndUserEmail::new(self.email.clone())));
        }
        let issuer = if mutation == "issuer" {
            "http://wrong.example"
        } else {
            &self.issuer
        };
        let audience = if mutation == "audience" {
            "other-client"
        } else {
            "qa-client"
        };
        let expiry = if mutation == "expired" {
            Utc::now() - chrono::Duration::minutes(5)
        } else {
            Utc::now() + chrono::Duration::minutes(5)
        };
        let claims = CoreIdTokenClaims::new(
            IssuerUrl::new(issuer.into()).unwrap(),
            vec![Audience::new(audience.into())],
            expiry,
            Utc::now(),
            standard,
            EmptyAdditionalClaims {},
        )
        .set_nonce(Some(Nonce::new(if mutation == "nonce" {
            "wrong-nonce".into()
        } else {
            nonce
        })));
        let secret = if mutation == "signature" {
            "wrong-signing-secret"
        } else {
            "qa-signing-secret"
        };
        let token = CoreIdToken::new(
            claims,
            &CoreHmacKey::new(secret),
            CoreJwsSigningAlgorithm::HmacSha256,
            None,
            None,
        )
        .unwrap();
        let mut reply = self.reply.lock().unwrap();
        reply.token = token.to_string();
        reply.verifier = verifier;
        self.state_hash.clone()
    }

    async fn cleanup(self) {
        sqlx::query("DELETE FROM google_login_attempts WHERE state_hash=$1")
            .bind(&self.state_hash)
            .execute(&self.pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM users WHERE google_sub=$1")
            .bind(&self.sub)
            .execute(&self.pool)
            .await
            .unwrap();
        self.server.abort();
        self.pool.close().await;
    }
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn google_completion_creates_a_verified_user_and_usable_session() {
    let mut f = Fixture::new().await;
    let state = f.attempt("valid").await;
    let (user, token, expiry) = f
        .auth
        .complete_google_login(state.clone(), "qa-code".into())
        .await
        .unwrap();
    assert!(
        user.verified && user.password_hash.is_none(),
        "Verified Google login does not create a verified passwordless user"
    );
    assert_eq!(
        user.google_sub.as_deref(),
        Some(f.sub.as_str()),
        "Google login maps the wrong subject"
    );
    assert_eq!(user.email, f.email, "Google signup stores the wrong email");
    assert_eq!(
        f.auth.get_user_id_from_session(&token).await.unwrap(),
        Some(user.id),
        "Successful Google login returns an unusable session token"
    );
    assert!(
        expiry > Utc::now(),
        "Google login returns an expired session"
    );
    let replay = f.auth.complete_google_login(state, "qa-code".into()).await;
    assert!(
        matches!(replay, Err(ServiceError::AuthError)),
        "A consumed OAuth attempt can be replayed"
    );
    assert_eq!(
        f.reply.lock().unwrap().calls,
        1,
        "Replay makes another provider request"
    );
    f.cleanup().await;
}

async fn reject_claim(mutation: &str) {
    let mut f = Fixture::new().await;
    let state = f.attempt(mutation).await;
    let result = f.auth.complete_google_login(state, "qa-code".into()).await;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE google_sub=$1")
        .bind(&f.sub)
        .fetch_one(&f.pool)
        .await
        .unwrap();
    f.cleanup().await;
    assert!(
        matches!(result, Err(ServiceError::AuthError)),
        "Invalid {mutation} claim reaches a successful login"
    );
    assert_eq!(count, 0, "Rejected {mutation} token creates an account");
}

macro_rules! rejection_test {
    ($name:ident, $mutation:literal) => {
        #[tokio::test]
        #[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
        async fn $name() {
            reject_claim($mutation).await;
        }
    };
}
rejection_test!(google_wrong_nonce_cannot_create_an_account, "nonce");
rejection_test!(google_wrong_signature_cannot_create_an_account, "signature");
rejection_test!(google_wrong_audience_cannot_create_an_account, "audience");
rejection_test!(google_wrong_issuer_cannot_create_an_account, "issuer");
rejection_test!(google_expired_id_token_cannot_create_an_account, "expired");
rejection_test!(
    google_unverified_email_cannot_create_an_account,
    "unverified"
);
rejection_test!(
    google_missing_email_cannot_create_an_account,
    "missing_email"
);

#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn google_expired_attempt_cannot_reach_the_token_server() {
    let mut f = Fixture::new().await;
    let state = f.attempt("valid").await;
    sqlx::query(
        "UPDATE google_login_attempts SET expires_at=NOW()-INTERVAL '1 minute' WHERE state_hash=$1",
    )
    .bind(&state)
    .execute(&f.pool)
    .await
    .unwrap();
    let result = f.auth.complete_google_login(state, "qa-code".into()).await;
    let calls = f.reply.lock().unwrap().calls;
    f.cleanup().await;
    assert!(
        matches!(result, Err(ServiceError::AuthError)),
        "An expired browser login attempt can authenticate"
    );
    assert_eq!(
        calls, 0,
        "Expired attempts send authorization codes to the provider"
    );
}
