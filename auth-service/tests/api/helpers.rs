use std::{str::FromStr, sync::Arc};

use auth_service::{
    app_state::AppState,
    domain::Email,
    get_postgres_pool, get_redis_client,
    services::{
        data_stores::{PostgresUserStore, RedisBannedTokenStore, RedisTwoFACodeStore},
        PostmarkEmailClient,
    },
    utils::{test, DB_URL, REDIS_HOST_NAME},
    Application,
};
use reqwest::{cookie::Jar, Client};
use secrecy::{ExposeSecret, Secret};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use tokio::sync::RwLock;
use uuid::Uuid;
use wiremock::{
    matchers::{header_exists, method, path},
    Mock, MockServer, ResponseTemplate,
};

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub email_server: MockServer,
    pub app_state: AppState,
    pub db_name: Secret<String>,
    pub cleanup_called: bool,
}

impl TestApp {
    pub async fn new() -> Self {
        let (pg_pool, db_name) = configure_postgresql().await;
        let redis_conn = Arc::new(RwLock::new(configure_redis()));

        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let banned_token_store =
            Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_conn.clone())));
        let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_conn.clone())));

        let email_server = MockServer::start().await;
        let base_url = email_server.uri();
        let email_client = Arc::new(RwLock::new(configure_postmark_email_client(base_url)));

        Mock::given(method("POST"))
            .and(path("/email"))
            .and(header_exists("X-Postmark-Server-Token"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&email_server)
            .await;

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
            email_server,
            app_state,
            db_name: Secret::new(db_name),
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
            panic!("TestApp was not cleaned up properly {:?}", self.db_name);
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

    let postgresql_conn_url_with_db =
        format!("{}/{}", postgresql_conn_url.expose_secret(), db_name);

    // Create a new connection pool and return it
    (
        get_postgres_pool(&Secret::new(postgresql_conn_url_with_db))
            .await
            .expect("Failed to create PostgreSQL connection pool"),
        db_name,
    )
}

async fn configure_database(db_conn_string: &Secret<String>, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string.expose_secret())
        .await
        .expect("Failed to connect to PostgreSQL");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database");

    // Connect to new database
    let db_conn_string_with_db = format!("{}/{}", db_conn_string.expose_secret(), db_name);
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

async fn delete_database(db_name: &Secret<String>) {
    let postgresql_conn_url: Secret<String> = DB_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url.expose_secret())
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
                db_name.expose_secret()
            )
            .as_str(),
        )
        .await
        .expect("Failed to terminate active connections");

    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name.expose_secret()).as_str())
        .await
        .expect("Failed to drop database");
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

fn configure_postmark_email_client(base_url: String) -> PostmarkEmailClient {
    let postmark_auth_token = Secret::new("auth_token".to_owned());

    let sender = Email::parse(test::email_client::SENDER).unwrap();

    let http_client = Client::builder()
        .timeout(test::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(base_url, sender, postmark_auth_token, http_client)
}
