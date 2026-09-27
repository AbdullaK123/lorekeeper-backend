use sqlx::PgPool;
use tower_sessions_sqlx_store_chrono::PostgresStore;
use tower_sessions::{
    SessionManagerLayer,
    cookie,
    Expiry
};
use tower_sessions::cookie::time::Duration;
use tower_sessions::service::{SignedCookie};

pub async fn create_session_layer(pool: PgPool, is_prod: bool, secret_key: &str) -> SessionManagerLayer<PostgresStore, SignedCookie> {

    if secret_key.as_bytes().len() < 64 {
        panic!("Invalid secret key. Must be at least 64 bytes.")
    }

    let store = PostgresStore::new(pool);
    let secret_key = cookie::Key::from(secret_key.as_bytes());
    store.migrate().await.unwrap();
    let layer = SessionManagerLayer::new(store)
        .with_name("session_id")
        .with_secure(is_prod)
        .with_http_only(true)
        .with_same_site(cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::hours(24)))
        .with_signed(secret_key);
    layer
}