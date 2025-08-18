use std::sync::Arc;

use auth_service::{
    app_state::AppState,
    get_postgres_pool,
    services::{HashMapTwoFACodeStore, HashMapUserStore, HashSetBannedTokenStore, MockEmailClient},
    utils::{prod, DB_URL},
    Application,
};
use sqlx::PgPool;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let _pg_pool = configure_postgresql().await;

    let user_store = Arc::new(RwLock::new(HashMapUserStore::default()));
    let banned_token_store = Arc::new(RwLock::new(HashSetBannedTokenStore::default()));
    let two_fa_code_store = Arc::new(RwLock::new(HashMapTwoFACodeStore::default()));
    let email_client = Arc::new(RwLock::new(MockEmailClient));
    let app_state = AppState::new(
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    );

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build application");

    app.run().await.expect("Failed to run application");
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DB_URL)
        .await
        .expect("Failed to create PostgreSQL connection pool");

    // Run database migrations against test database
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}
