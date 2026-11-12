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
    /// Logged-in user who starts the attempt. `None` is a plain login.
    link_user: Option<uuid::Uuid>,
    /// Password users created by a test; removed in cleanup.
    local_users: Vec<uuid::Uuid>,
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
            link_user: None,
            local_users: Vec::new(),
        }
    }

    async fn attempt(&mut self, mutation: &str) -> String {
        let (url, state) = self.auth.begin_google_login(self.link_user).await.unwrap();
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

    /// Insert a password account, as if the writer signed up with email first.
    async fn local_user(&mut self, email: &str) -> uuid::Uuid {
        let id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO users (username, email, password_hash, verified) VALUES ('Local', $1, 'qa-hash', TRUE) RETURNING id",
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .unwrap();
        self.local_users.push(id);
        id
    }

    async fn google_sub_of(&self, id: uuid::Uuid) -> Option<String> {
        sqlx::query_scalar("SELECT google_sub FROM users WHERE id=$1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn users_with_sub(&self) -> i64 {
        sqlx::query_scalar("SELECT count(*) FROM users WHERE google_sub=$1")
            .bind(&self.sub)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn cleanup(self) {
        sqlx::query("DELETE FROM users WHERE id = ANY($1)")
            .bind(&self.local_users)
            .execute(&self.pool)
            .await
            .unwrap();
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
        .complete_google_login(state.clone(), "qa-code".into(), None)
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
    let replay = f.auth.complete_google_login(state, "qa-code".into(), None).await;
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
    let result = f.auth.complete_google_login(state, "qa-code".into(), None).await;
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
    let result = f.auth.complete_google_login(state, "qa-code".into(), None).await;
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

// ---- Account linking (US-043) ----

#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn google_link_creates_a_second_account_instead_of_linking() {
    let mut f = Fixture::new().await;
    let local = f.local_user(&format!("local-{}@example.com", f.sub)).await;
    f.link_user = Some(local);
    let state = f.attempt("valid").await;
    let result = f
        .auth
        .complete_google_login(state, "qa-code".into(), Some(local))
        .await;
    let linked_sub = f.google_sub_of(local).await;
    let owners = f.users_with_sub().await;
    f.cleanup().await;
    let (user, _, _) = result.expect("A valid link from the owning session fails");
    assert_eq!(
        user.id, local,
        "Linking logs the writer into a different account; their worlds seem gone"
    );
    assert_eq!(
        linked_sub.as_deref(),
        Some(user.google_sub.as_deref().unwrap()),
        "Link reports success but the password account has no Google subject"
    );
    assert_eq!(owners, 1, "Linking creates a duplicate account for the same Google subject");
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn google_link_fails_when_google_email_matches_the_local_account() {
    let mut f = Fixture::new().await;
    let email = f.email.clone();
    let local = f.local_user(&email).await;
    f.link_user = Some(local);
    let state = f.attempt("valid").await;
    let result = f
        .auth
        .complete_google_login(state, "qa-code".into(), Some(local))
        .await;
    let linked_sub = f.google_sub_of(local).await;
    f.cleanup().await;
    assert!(
        matches!(&result, Ok((user, _, _)) if user.id == local),
        "Same-email link hits the signup email collision; the common case cannot link"
    );
    assert!(linked_sub.is_some(), "Same-email link leaves the account unlinked");
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn google_link_can_be_finished_by_another_users_session() {
    let mut f = Fixture::new().await;
    let victim = f.local_user(&format!("victim-{}@example.com", f.sub)).await;
    let other = f.local_user(&format!("other-{}@example.com", f.sub)).await;
    f.link_user = Some(victim);
    let state = f.attempt("valid").await;
    let result = f
        .auth
        .complete_google_login(state, "qa-code".into(), Some(other))
        .await;
    let victim_sub = f.google_sub_of(victim).await;
    let other_sub = f.google_sub_of(other).await;
    let calls = f.reply.lock().unwrap().calls;
    f.cleanup().await;
    assert!(
        matches!(result, Err(ServiceError::AuthError)),
        "Another user's session completes a link it did not start"
    );
    assert!(
        victim_sub.is_none() && other_sub.is_none(),
        "A cross-session callback still links a Google account"
    );
    assert_eq!(calls, 0, "A rejected link callback still spends the authorization code");
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn google_link_can_be_finished_after_logging_out() {
    let mut f = Fixture::new().await;
    let local = f.local_user(&format!("local-{}@example.com", f.sub)).await;
    f.link_user = Some(local);
    let state = f.attempt("valid").await;
    let result = f
        .auth
        .complete_google_login(state, "qa-code".into(), None)
        .await;
    let linked_sub = f.google_sub_of(local).await;
    let owners = f.users_with_sub().await;
    f.cleanup().await;
    assert!(
        matches!(result, Err(ServiceError::AuthError)),
        "A link attempt finishes with no session, so anyone holding the browser can complete it"
    );
    assert!(linked_sub.is_none(), "A sessionless callback links the account");
    assert_eq!(owners, 0, "A sessionless link callback signs up a new Google user instead");
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn google_link_steals_a_google_account_owned_by_another_user() {
    let mut f = Fixture::new().await;
    let owner = f.local_user(&format!("owner-{}@example.com", f.sub)).await;
    sqlx::query("UPDATE users SET google_sub=$1 WHERE id=$2")
        .bind(&f.sub)
        .bind(owner)
        .execute(&f.pool)
        .await
        .unwrap();
    let thief = f.local_user(&format!("thief-{}@example.com", f.sub)).await;
    f.link_user = Some(thief);
    let state = f.attempt("valid").await;
    let result = f
        .auth
        .complete_google_login(state, "qa-code".into(), Some(thief))
        .await;
    let owner_sub = f.google_sub_of(owner).await;
    let thief_sub = f.google_sub_of(thief).await;
    let sub = f.sub.clone();
    f.cleanup().await;
    assert!(
        matches!(result, Err(ServiceError::Conflict(_))),
        "Linking an already-linked Google account is not reported as a conflict (500 or silent success)"
    );
    assert_eq!(
        owner_sub.as_deref(),
        Some(sub.as_str()),
        "The original owner loses their Google sign-in"
    );
    assert!(thief_sub.is_none(), "A second account claims someone else's Google identity");
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn google_link_survives_deletion_of_the_linking_user() {
    let mut f = Fixture::new().await;
    let local = f.local_user(&format!("local-{}@example.com", f.sub)).await;
    f.link_user = Some(local);
    let state = f.attempt("valid").await;
    sqlx::query("DELETE FROM users WHERE id=$1")
        .bind(local)
        .execute(&f.pool)
        .await
        .unwrap();
    let pending: i64 =
        sqlx::query_scalar("SELECT count(*) FROM google_login_attempts WHERE state_hash=$1")
            .bind(&state)
            .fetch_one(&f.pool)
            .await
            .unwrap();
    let result = f
        .auth
        .complete_google_login(state, "qa-code".into(), Some(local))
        .await;
    let owners = f.users_with_sub().await;
    f.cleanup().await;
    assert_eq!(pending, 0, "Deleting a user leaves their pending link attempt behind");
    assert!(
        matches!(result, Err(ServiceError::AuthError)),
        "A link for a deleted user still completes"
    );
    assert_eq!(owners, 0, "A link for a deleted user signs up a fresh account instead");
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn google_plain_login_breaks_when_a_session_is_present() {
    let mut f = Fixture::new().await;
    let local = f.local_user(&format!("local-{}@example.com", f.sub)).await;
    // Attempt started with no session (link_user stays None); the callback sees one.
    let state = f.attempt("valid").await;
    let result = f
        .auth
        .complete_google_login(state, "qa-code".into(), Some(local))
        .await;
    let linked_sub = f.google_sub_of(local).await;
    f.cleanup().await;
    assert!(
        result.is_ok(),
        "A plain Google login fails just because a session cookie is present"
    );
    assert!(
        linked_sub.is_none(),
        "A plain login attempt links the Google account to whoever is logged in"
    );
}

// ---- Account linking: forged tokens, stale attempts, races ----

async fn reject_link_claim(mutation: &str) {
    let mut f = Fixture::new().await;
    let local = f.local_user(&format!("local-{}@example.com", f.sub)).await;
    f.link_user = Some(local);
    let state = f.attempt(mutation).await;
    let result = f
        .auth
        .complete_google_login(state, "qa-code".into(), Some(local))
        .await;
    let linked_sub = f.google_sub_of(local).await;
    let owners = f.users_with_sub().await;
    f.cleanup().await;
    assert!(
        matches!(result, Err(ServiceError::AuthError)),
        "A forged {mutation} token completes a link"
    );
    assert!(
        linked_sub.is_none(),
        "A forged {mutation} token attaches an attacker's Google subject to the account"
    );
    assert_eq!(owners, 0, "A forged {mutation} token creates an account during a link");
}

macro_rules! link_rejection_test {
    ($name:ident, $mutation:literal) => {
        #[tokio::test]
        #[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
        async fn $name() {
            reject_link_claim($mutation).await;
        }
    };
}
link_rejection_test!(google_link_accepts_a_wrong_nonce, "nonce");
link_rejection_test!(google_link_accepts_a_wrong_signature, "signature");
link_rejection_test!(google_link_accepts_a_wrong_audience, "audience");
link_rejection_test!(google_link_accepts_a_wrong_issuer, "issuer");
link_rejection_test!(google_link_accepts_an_expired_id_token, "expired");

#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn google_link_completes_from_an_expired_attempt() {
    let mut f = Fixture::new().await;
    let local = f.local_user(&format!("local-{}@example.com", f.sub)).await;
    f.link_user = Some(local);
    let state = f.attempt("valid").await;
    sqlx::query(
        "UPDATE google_login_attempts SET expires_at=NOW()-INTERVAL '1 minute' WHERE state_hash=$1",
    )
    .bind(&state)
    .execute(&f.pool)
    .await
    .unwrap();
    let result = f
        .auth
        .complete_google_login(state, "qa-code".into(), Some(local))
        .await;
    let linked_sub = f.google_sub_of(local).await;
    let calls = f.reply.lock().unwrap().calls;
    f.cleanup().await;
    assert!(
        matches!(result, Err(ServiceError::AuthError)),
        "A stale link attempt still links; an old tab can attach Google later"
    );
    assert!(linked_sub.is_none(), "An expired link attempt changes the account");
    assert_eq!(calls, 0, "Expired link attempts still spend the authorization code");
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn google_link_attempt_can_be_replayed() {
    let mut f = Fixture::new().await;
    let local = f.local_user(&format!("local-{}@example.com", f.sub)).await;
    f.link_user = Some(local);
    let state = f.attempt("valid").await;
    let first = f
        .auth
        .complete_google_login(state.clone(), "qa-code".into(), Some(local))
        .await;
    let replay = f
        .auth
        .complete_google_login(state, "qa-code".into(), Some(local))
        .await;
    let calls = f.reply.lock().unwrap().calls;
    f.cleanup().await;
    assert!(first.is_ok(), "The first valid link fails");
    assert!(
        matches!(replay, Err(ServiceError::AuthError)),
        "A consumed link attempt can be replayed to mint another session"
    );
    assert_eq!(calls, 1, "Link replay makes another provider request");
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn google_relinking_the_same_account_reports_a_conflict() {
    let mut f = Fixture::new().await;
    let local = f.local_user(&format!("local-{}@example.com", f.sub)).await;
    f.link_user = Some(local);
    let first_state = f.attempt("valid").await;
    f.auth
        .complete_google_login(first_state, "qa-code".into(), Some(local))
        .await
        .expect("The first valid link fails");
    // The writer clicks "Link Google" again with the same Google account.
    let second_state = f.attempt("valid").await;
    let second = f
        .auth
        .complete_google_login(second_state, "qa-code".into(), Some(local))
        .await;
    let linked_sub = f.google_sub_of(local).await;
    let sub = f.sub.clone();
    f.cleanup().await;
    assert!(
        matches!(&second, Ok((user, _, _)) if user.id == local),
        "Re-linking an account to its own Google identity fails; a double click shows an error"
    );
    assert_eq!(
        linked_sub.as_deref(),
        Some(sub.as_str()),
        "Re-linking the same Google account loses the link"
    );
}

/// Two writers link the same Google account at the same moment.
/// Writer A's update holds the unique-index slot in an open transaction.
/// Writer B's update must wait, then lose with a Conflict, not a raw database error.
#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn concurrent_links_of_one_google_account_return_a_server_error() {
    let mut f = Fixture::new().await;
    let first = f.local_user(&format!("first-{}@example.com", f.sub)).await;
    let second = f.local_user(&format!("second-{}@example.com", f.sub)).await;
    let sub = f.sub.clone();

    let mut tx = f.pool.begin().await.unwrap();
    sqlx::query("UPDATE users SET google_sub=$1 WHERE id=$2")
        .bind(&sub)
        .bind(first)
        .execute(&mut *tx)
        .await
        .unwrap();

    let repo = UserRepository::new(f.pool.clone());
    let racing_sub = sub.clone();
    let racer = tokio::spawn(async move { repo.set_google_sub(second, racing_sub).await });

    // Wait until the second update is really blocked on the first, so the race is deterministic.
    let mut blocked = false;
    for _ in 0..100 {
        let waiting: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_stat_activity
             WHERE wait_event_type = 'Lock' AND query LIKE '%UPDATE users SET google_sub%'
               AND pid <> pg_backend_pid()",
        )
        .fetch_one(&f.pool)
        .await
        .unwrap();
        if waiting > 0 {
            blocked = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    tx.commit().await.unwrap();
    let result = racer.await.unwrap();
    let second_sub = f.google_sub_of(second).await;
    let first_sub = f.google_sub_of(first).await;
    f.cleanup().await;

    assert!(blocked, "Test setup: the second link never waited on the first");
    assert!(
        matches!(result, Err(crate::infrastructure::InfrastructureError::Conflict(_))),
        "A racing link surfaces as a 500 instead of 'already linked'"
    );
    assert!(second_sub.is_none(), "Both writers end up claiming one Google account");
    assert_eq!(first_sub.as_deref(), Some(sub.as_str()), "The winning link is lost");
}

/// Proposed rule (undecided in US-043): linking must not silently replace an existing Google link.
/// Delete this test if you decide replacing is correct.
#[tokio::test]
#[ignore = "requires isolated QA Postgres and loopback sockets; no Google/email calls"]
async fn google_link_silently_replaces_an_existing_google_link() {
    let mut f = Fixture::new().await;
    let local = f.local_user(&format!("local-{}@example.com", f.sub)).await;
    let old_sub = format!("old-{}", f.sub);
    sqlx::query("UPDATE users SET google_sub=$1 WHERE id=$2")
        .bind(&old_sub)
        .bind(local)
        .execute(&f.pool)
        .await
        .unwrap();
    f.link_user = Some(local);
    let state = f.attempt("valid").await;
    let result = f
        .auth
        .complete_google_login(state, "qa-code".into(), Some(local))
        .await;
    let linked_sub = f.google_sub_of(local).await;
    f.cleanup().await;
    assert!(
        matches!(result, Err(ServiceError::Conflict(_))),
        "Linking a second Google account is not reported as a conflict (got {:?}); the writer sees a generic failure or loses the first link", result.as_ref().map(|(user, _, _)| user.id)
    );
    assert_eq!(
        linked_sub.as_deref(),
        Some(old_sub.as_str()),
        "The writer's original Google sign-in stops working after linking another account"
    );
}
