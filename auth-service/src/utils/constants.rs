use dotenvy::dotenv;
use secrecy::Secret;

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";

lazy_static::lazy_static! {
    pub static ref JWT_SECRET: Secret<String> = Secret::new(set_token());
    pub static ref DB_URL: Secret<String> = Secret::new(set_db_url());
    pub static ref REDIS_HOST_NAME: String = set_redis_host();
    pub static ref POSTMARK_AUTH_TOKEN: Secret<String> = Secret::new(set_postmark_token());
}

fn set_token() -> String {
    dotenv().ok();
    let secret = std::env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET not set in environment");
    if secret.is_empty() {
        panic!("JWT_SECRET is empty");
    }
    secret
}

fn set_db_url() -> String {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set in environment");
    if db_url.is_empty() {
        panic!("DATABASE_URL is empty");
    }
    db_url
}

fn set_redis_host() -> String {
    dotenv().ok();

    std::env::var(env::REDIS_HOST_NAME_ENV_VAR).unwrap_or(DEFAULT_REDIS_HOSTNAME.to_owned())
}

fn set_postmark_token() -> String {
    dotenv().ok();
    let token = std::env::var(env::POSTMARK_AUTH_TOKEN_ENV_VAR)
        .expect("POSTMARK_AUTH_TOKEN not set in environment");
    if token.is_empty() {
        panic!("POSTMARK_AUTH_TOKEN is empty");
    }
    token
}

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
    pub const POSTMARK_AUTH_TOKEN_ENV_VAR: &str = "POSTMARK_AUTH_TOKEN";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
    pub mod email_client {
        use std::time::Duration;

        pub const BASE_URL: &str = "https://api.postmarkapp.com/email";
        pub const SENDER: &str = "bogdan@codeiron.io";
        pub const TIMEOUT: Duration = std::time::Duration::from_secs(10);
    }
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
    pub mod email_client {
        use std::time::Duration;

        pub const SENDER: &str = "test@email.com";
        pub const TIMEOUT: Duration = std::time::Duration::from_millis(200);
    }
}
