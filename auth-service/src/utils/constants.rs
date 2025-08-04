use dotenvy::dotenv;

pub const JWT_COOKIE_NAME: &str = "jwt";

lazy_static::lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
}

fn set_token() -> String {
    dotenv().ok();
    let secret = std::env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET not set in environment");
    if secret.is_empty() {
        panic!("JWT_SECRET is empty");
    }
    secret
}

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
}
