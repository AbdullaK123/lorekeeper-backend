use sqlx::PgPool;
use tower_sessions_sqlx_store_chrono::PostgresStore;
use tower_sessions::{
    SessionManagerLayer,
    cookie,
    Expiry
};
use tower_sessions::cookie::time::Duration;
use tower_sessions::service::{SignedCookie};

pub async fn create_session_layer(pool: PgPool, is_prod: bool) -> SessionManagerLayer<PostgresStore, SignedCookie> {
    let store = PostgresStore::new(pool);
    let secret_key = cookie::Key::generate();
    store.migrate().await.unwrap();
    let layer = SessionManagerLayer::new(store)
        .with_name("session_id")
        .with_secure(if is_prod {true} else {false})
        .with_http_only(true)
        .with_same_site(cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::hours(24)))
        .with_signed(secret_key);
    layer
}