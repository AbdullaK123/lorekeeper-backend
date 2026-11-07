//! Real Postgres token lifecycle checks. No provider or email calls.
use super::*;
use sha2::{Digest, Sha256};

struct Fixture {
    repo: data::AuthTokenRepository,
    user: uuid::Uuid,
}

impl Fixture {
    async fn new() -> Self {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
        assert!(
            url.ends_with("/lorekeeper_adversarial"),
            "Token tests must use the isolated QA database"
        );
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(4)
            .connect(&url)
            .await
            .unwrap();
        let user = uuid::Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, username, email) VALUES ($1, 'Token QA', $2)")
            .bind(user)
            .bind(format!("token-{user}@example.com"))
            .execute(&pool)
            .await
            .unwrap();
        Self {
            repo: data::AuthTokenRepository::new(pool),
            user,
        }
    }

    async fn consume_token(
        &self,
        user: uuid::Uuid,
        purpose: data::AuthTokenPurpose,
        hash: String,
    ) -> Result<Option<data::AuthTokenRow>, infrastructure::InfrastructureError> {
        let mut connection = self.repo.pool.acquire().await?;
        self.repo
            .consume_token(user, purpose, hash, &mut connection)
            .await
    }

    fn auth_service(&self) -> service::AuthService {
        service::AuthService::new(
            self.repo.pool.clone(),
            data::UserRepository::new(self.repo.pool.clone()),
            self.repo.clone(),
            "re_test_not_sent",
            "https://api.example.test",
        )
    }

    async fn saved_password(&self) -> Option<String> {
        sqlx::query_scalar("SELECT password_hash FROM users WHERE id=$1")
            .bind(self.user)
            .fetch_one(&self.repo.pool)
            .await
            .unwrap()
    }

    async fn cleanup(&self) {
        sqlx::query("DELETE FROM users WHERE id=$1")
            .bind(self.user)
            .execute(&self.repo.pool)
            .await
            .unwrap();
        self.repo.pool.close().await;
    }
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_valid_token_cannot_be_reused() {
    let f = Fixture::new().await;
    let (row, raw) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::PasswordReset)
        .await
        .unwrap();
    assert_ne!(
        row.token_hash, raw,
        "The database stores the usable reset token in plaintext"
    );
    assert_eq!(
        row.token_hash,
        hex::encode(Sha256::digest(raw.as_bytes())),
        "The emailed token cannot reproduce its saved verifier"
    );
    let first = f
        .consume_token(
            f.user,
            data::AuthTokenPurpose::PasswordReset,
            row.token_hash.clone(),
        )
        .await
        .unwrap();
    let second = f
        .consume_token(
            f.user,
            data::AuthTokenPurpose::PasswordReset,
            row.token_hash,
        )
        .await
        .unwrap();
    f.cleanup().await;
    assert!(
        first.is_some(),
        "An unexpired reset link cannot be consumed"
    );
    assert!(
        second.is_none(),
        "The same reset link authorizes more than one account change"
    );
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_expired_token_cannot_be_consumed() {
    let f = Fixture::new().await;
    let (row, _) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::PasswordReset)
        .await
        .unwrap();
    sqlx::query("UPDATE auth_tokens SET created_at=NOW()-INTERVAL '2 hours', expired_at=NOW()-INTERVAL '1 hour' WHERE user_id=$1")
        .bind(f.user).execute(&f.repo.pool).await.unwrap();
    let consumed = f
        .consume_token(
            f.user,
            data::AuthTokenPurpose::PasswordReset,
            row.token_hash,
        )
        .await
        .unwrap();
    f.cleanup().await;
    assert!(
        consumed.is_none(),
        "An expired reset link still authorizes an account change"
    );
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_reissue_invalidates_the_previous_link() {
    let f = Fixture::new().await;
    let (old, old_raw) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::PasswordReset)
        .await
        .unwrap();
    sqlx::query("UPDATE auth_tokens SET created_at=NOW()-INTERVAL '1 minute', expired_at=NOW()+INTERVAL '1 minute' WHERE user_id=$1")
        .bind(f.user).execute(&f.repo.pool).await.unwrap();
    let (new, new_raw) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::PasswordReset)
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM auth_tokens WHERE user_id=$1 AND purpose='password_reset'",
    )
    .bind(f.user)
    .fetch_one(&f.repo.pool)
    .await
    .unwrap();
    let old_result = f
        .consume_token(
            f.user,
            data::AuthTokenPurpose::PasswordReset,
            old.token_hash,
        )
        .await
        .unwrap();
    let new_result = f
        .consume_token(
            f.user,
            data::AuthTokenPurpose::PasswordReset,
            new.token_hash,
        )
        .await
        .unwrap();
    f.cleanup().await;
    assert_ne!(
        old_raw, new_raw,
        "Reissuing a reset link reuses its old secret"
    );
    assert_eq!(
        count, 1,
        "Reissue leaves multiple outstanding links for one purpose"
    );
    assert!(
        new.expired_at > old.expired_at - chrono::Duration::seconds(5),
        "Reissue does not grant a fresh expiry window"
    );
    assert!(
        old_result.is_none(),
        "An earlier reset link survives reissue"
    );
    assert!(
        new_result.is_some(),
        "A rejected old link consumes the replacement token"
    );
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_wrong_user_cannot_consume_the_token() {
    let f = Fixture::new().await;
    let (row, _) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::PasswordReset)
        .await
        .unwrap();
    let wrong = f
        .consume_token(
            uuid::Uuid::new_v4(),
            data::AuthTokenPurpose::PasswordReset,
            row.token_hash.clone(),
        )
        .await
        .unwrap();
    let right = f
        .consume_token(
            f.user,
            data::AuthTokenPurpose::PasswordReset,
            row.token_hash,
        )
        .await
        .unwrap();
    f.cleanup().await;
    assert!(
        wrong.is_none(),
        "A token is accepted for a different account"
    );
    assert!(
        right.is_some(),
        "A wrong-user attempt deletes the owner's valid token"
    );
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_wrong_purpose_cannot_consume_the_token() {
    let f = Fixture::new().await;
    let (row, _) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::EmailVerification)
        .await
        .unwrap();
    let wrong = f
        .consume_token(
            f.user,
            data::AuthTokenPurpose::PasswordReset,
            row.token_hash.clone(),
        )
        .await
        .unwrap();
    let right = f
        .consume_token(
            f.user,
            data::AuthTokenPurpose::EmailVerification,
            row.token_hash,
        )
        .await
        .unwrap();
    f.cleanup().await;
    assert!(
        wrong.is_none(),
        "An email-verification token authorizes a password reset"
    );
    assert!(
        right.is_some(),
        "A wrong-purpose attempt consumes the verification link"
    );
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_wrong_hash_does_not_consume_a_valid_link() {
    let f = Fixture::new().await;
    let (row, _) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::PasswordReset)
        .await
        .unwrap();
    let wrong = f
        .consume_token(
            f.user,
            data::AuthTokenPurpose::PasswordReset,
            "0".repeat(64),
        )
        .await
        .unwrap();
    let right = f
        .consume_token(
            f.user,
            data::AuthTokenPurpose::PasswordReset,
            row.token_hash,
        )
        .await
        .unwrap();
    f.cleanup().await;
    assert!(
        wrong.is_none(),
        "An incorrect token hash authorizes an account change"
    );
    assert!(
        right.is_some(),
        "An incorrect hash destroys a valid reset link"
    );
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_concurrent_consumers_cannot_both_succeed() {
    let f = Fixture::new().await;
    let (row, _) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::PasswordReset)
        .await
        .unwrap();
    let (a, b) = tokio::join!(
        f.consume_token(
            f.user,
            data::AuthTokenPurpose::PasswordReset,
            row.token_hash.clone()
        ),
        f.consume_token(
            f.user,
            data::AuthTokenPurpose::PasswordReset,
            row.token_hash
        ),
    );
    let successes = usize::from(a.unwrap().is_some()) + usize::from(b.unwrap().is_some());
    f.cleanup().await;
    assert_eq!(
        successes, 1,
        "Concurrent requests both authorize changes using one token"
    );
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_purpose_lifetimes_are_not_swapped() {
    let f = Fixture::new().await;
    let (reset, _) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::PasswordReset)
        .await
        .unwrap();
    let (verify, _) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::EmailVerification)
        .await
        .unwrap();
    f.cleanup().await;
    let reset_seconds = (reset.expired_at - reset.created_at).num_seconds();
    let verify_seconds = (verify.expired_at - verify.created_at).num_seconds();
    assert!(
        (1790..=1810).contains(&reset_seconds),
        "Reset links do not use the intended 30-minute lifetime"
    );
    assert!(
        (86390..=86410).contains(&verify_seconds),
        "Verification links do not use the intended 24-hour lifetime"
    );
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_reset_commits_password_with_token_consumption() {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let f = Fixture::new().await;
    let (_, raw) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::PasswordReset)
        .await
        .unwrap();
    let result = f
        .auth_service()
        .reset_password(f.user, raw.clone(), "New-Password-938!".into())
        .await;
    let hash = f.saved_password().await.unwrap();
    let valid_password = Argon2::default()
        .verify_password(b"New-Password-938!", &PasswordHash::new(&hash).unwrap())
        .is_ok();
    let consumed = f
        .consume_token(
            f.user,
            data::AuthTokenPurpose::PasswordReset,
            hex::encode(Sha256::digest(raw.as_bytes())),
        )
        .await
        .unwrap();
    f.cleanup().await;
    assert!(result.is_ok(), "A valid reset transaction cannot commit");
    assert!(
        valid_password,
        "Reset reports success without persisting the new password"
    );
    assert!(
        consumed.is_none(),
        "The password commits while its reset token remains usable"
    );
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_reset_with_invalid_secret_cannot_change_password() {
    let f = Fixture::new().await;
    let (row, _) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::PasswordReset)
        .await
        .unwrap();
    let before = f.saved_password().await;
    let result = f
        .auth_service()
        .reset_password(f.user, "wrong-secret".into(), "New-Password-938!".into())
        .await;
    let after = f.saved_password().await;
    let still_valid = f
        .consume_token(
            f.user,
            data::AuthTokenPurpose::PasswordReset,
            row.token_hash,
        )
        .await
        .unwrap();
    f.cleanup().await;
    assert!(
        matches!(result, Err(service::ServiceError::InvalidInput(_))),
        "An invalid token reaches the successful reset path"
    );
    assert_eq!(
        after, before,
        "An invalid secret changes the account password"
    );
    assert!(
        still_valid.is_some(),
        "An invalid-secret request destroys the valid reset token"
    );
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_reset_with_expired_secret_cannot_change_password() {
    let f = Fixture::new().await;
    let (_, raw) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::PasswordReset)
        .await
        .unwrap();
    sqlx::query("UPDATE auth_tokens SET created_at=NOW()-INTERVAL '2 hours', expired_at=NOW()-INTERVAL '1 hour' WHERE user_id=$1")
        .bind(f.user).execute(&f.repo.pool).await.unwrap();
    let before = f.saved_password().await;
    let result = f
        .auth_service()
        .reset_password(f.user, raw, "New-Password-938!".into())
        .await;
    let after = f.saved_password().await;
    f.cleanup().await;
    assert!(
        matches!(result, Err(service::ServiceError::InvalidInput(_))),
        "An expired reset secret authorizes a password change"
    );
    assert_eq!(after, before, "An expired token changes the password");
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_reset_failure_restores_consumed_token() {
    let f = Fixture::new().await;
    let (_, raw) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::PasswordReset)
        .await
        .unwrap();
    let name = format!("qa_reset_failure_{}", f.user.simple());
    sqlx::query(&format!("CREATE FUNCTION {name}() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'Injected password-update failure'; END $$"))
        .execute(&f.repo.pool).await.unwrap();
    sqlx::query(&format!("CREATE TRIGGER {name} BEFORE UPDATE OF password_hash ON users FOR EACH ROW WHEN (OLD.id = '{}'::uuid) EXECUTE FUNCTION {name}()", f.user))
        .execute(&f.repo.pool).await.unwrap();
    let before = f.saved_password().await;
    let failed = f
        .auth_service()
        .reset_password(f.user, raw.clone(), "New-Password-938!".into())
        .await;
    let after = f.saved_password().await;
    sqlx::query(&format!("DROP TRIGGER {name} ON users"))
        .execute(&f.repo.pool)
        .await
        .unwrap();
    sqlx::query(&format!("DROP FUNCTION {name}()"))
        .execute(&f.repo.pool)
        .await
        .unwrap();
    let retried = f
        .auth_service()
        .reset_password(f.user, raw, "New-Password-938!".into())
        .await;
    f.cleanup().await;
    assert!(
        matches!(&failed, Err(service::ServiceError::InternalError(_))),
        "The injected database failure is not propagated as an internal error"
    );
    assert_eq!(after, before, "Failed reset partially changes the account");
    assert!(
        retried.is_ok(),
        "A failed password write permanently consumes the reset token"
    );
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_reset_secret_cannot_be_replayed_through_service() {
    let f = Fixture::new().await;
    let (_, raw) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::PasswordReset)
        .await
        .unwrap();
    let first = f
        .auth_service()
        .reset_password(f.user, raw.clone(), "First-Password-938!".into())
        .await;
    let before = f.saved_password().await;
    let second = f
        .auth_service()
        .reset_password(f.user, raw, "Second-Password-938!".into())
        .await;
    let after = f.saved_password().await;
    f.cleanup().await;
    assert!(first.is_ok(), "The initial valid reset fails");
    assert!(
        matches!(second, Err(service::ServiceError::InvalidInput(_))),
        "A consumed secret authorizes a second password change"
    );
    assert_eq!(after, before, "Replay changes the password again");
}

#[tokio::test]
#[ignore = "requires isolated Postgres; no email or model calls"]
async fn auth_tokens_verification_rollback_restores_account_and_token() {
    let f = Fixture::new().await;
    let (row, _) = f
        .repo
        .upsert_token(f.user, data::AuthTokenPurpose::EmailVerification)
        .await
        .unwrap();
    let mut tx = f.repo.pool.begin().await.unwrap();
    let consumed = f
        .repo
        .consume_token(
            f.user,
            data::AuthTokenPurpose::EmailVerification,
            row.token_hash.clone(),
            &mut tx,
        )
        .await
        .unwrap();
    let updated = data::UserRepository::new(f.repo.pool.clone())
        .verify_user(f.user, &mut tx)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    let verified: bool = sqlx::query_scalar("SELECT verified FROM users WHERE id=$1")
        .bind(f.user)
        .fetch_one(&f.repo.pool)
        .await
        .unwrap();
    let restored = f
        .consume_token(
            f.user,
            data::AuthTokenPurpose::EmailVerification,
            row.token_hash,
        )
        .await
        .unwrap();
    f.cleanup().await;
    assert!(
        consumed.is_some() && updated.is_some_and(|user| user.verified),
        "Verification transaction never consumes the token and sets verified=true"
    );
    assert!(!verified, "Rolling back leaves the account marked verified");
    assert!(
        restored.is_some(),
        "Rolling back permanently consumes the verification token"
    );
}
