use std::{str::FromStr, sync::Arc};

use auth_service::{
    app_state::AppState,
    get_postgres_pool,
    services::{
        data_stores::{HashMapTwoFACodeStore, HashSetBannedTokenStore, PostgresUserStore},
        MockEmailClient,
    },
    utils::{test, DB_URL},
    Application,
};
use reqwest::cookie::Jar;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub app_state: AppState,
    pub db_name: String,
    pub cleanup_called: bool,
}

impl TestApp {
    pub async fn new() -> Self {
        let (pg_pool, db_name) = configure_postgresql().await;
        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let banned_token_store = Arc::new(RwLock::new(HashSetBannedTokenStore::default()));
        let two_fa_code_store = Arc::new(RwLock::new(HashMapTwoFACodeStore::default()));
        let email_client = Arc::new(RwLock::new(MockEmailClient));
        let app_state = AppState::new(
            user_store,
            banned_token_store,
            two_fa_code_store,
            email_client,
        );

        let app = Application::build(app_state.clone(), test::APP_ADDRESS)
            .await
            .expect("Failed to build application");

        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .expect("Failed to build HTTP client");

        Self {
            address,
            cookie_jar,
            http_client,
            app_state,
            db_name,
            cleanup_called: false,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn logout(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn cleanup(&mut self) {
        delete_database(&self.db_name).await;
        self.cleanup_called = true;
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.cleanup_called {
            panic!("TestApp was not cleaned up properly {}", self.db_name);
        }
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", uuid::Uuid::new_v4())
}

async fn configure_postgresql() -> (PgPool, String) {
    let postgresql_conn_url = DB_URL.to_owned();

    // Creating new db for each test case, need to ensure each db has unique name
    let db_name = Uuid::new_v4().to_string();
    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    (
        get_postgres_pool(&postgresql_conn_url_with_db)
            .await
            .expect("Failed to create PostgreSQL connection pool"),
        db_name,
    )
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database");

    // Connect to new database
    let db_conn_string_with_db = format!("{}/{}", db_conn_string, db_name);
    let connection = PgPoolOptions::new()
        .connect(&db_conn_string_with_db)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to run migrations");
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = DB_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                AND pid <> pg_backend_pid();"#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to terminate active connections");

    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop database");
}
