use axum_extra::extract::cookie::{Cookie, SameSite};
use color_eyre::eyre::{eyre, Context, ContextCompat, Result};
use jsonwebtoken::{decode, DecodingKey, Validation};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

use crate::{
    domain::Email,
    utils::{JWT_COOKIE_NAME, JWT_SECRET},
};

#[tracing::instrument(name = "Generate Auth Cookie", skip_all)]
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>> {
    let token = generate_auth_token(email)?;
    Ok(create_auth_cookie(token))
}

#[tracing::instrument(name = "Create Auth Cookie", skip_all)]
fn create_auth_cookie(token: String) -> Cookie<'static> {
    Cookie::build((JWT_COOKIE_NAME, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build()
}

pub const TOKEN_TTL_SECONDS: i64 = 600; // 10 minutes

#[tracing::instrument(name = "Generate Auth Token", skip_all)]
fn generate_auth_token(email: &Email) -> Result<String> {
    let delta = chrono::Duration::try_seconds(TOKEN_TTL_SECONDS)
        .wrap_err("Failed to create 10 minute time delta")?;

    let expiration = chrono::Utc::now()
        .checked_add_signed(delta)
        .ok_or(eyre!("Failed to add time delta to current time"))?
        .timestamp();

    let expiration: usize = expiration
        .try_into()
        .wrap_err("Failed to convert expiration to usize")?;

    let sub = email.as_ref().expose_secret().to_string();

    let claims = Claims {
        sub,
        exp: expiration,
    };

    create_token(&claims)
}

#[tracing::instrument(name = "Validate Token", skip_all)]
pub async fn validate_token(token: &str) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.expose_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| eyre!("Token validation error: {}", e))
}

#[tracing::instrument(name = "Create Token", skip_all)]
fn create_token(claims: &Claims) -> Result<String> {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        claims,
        &jsonwebtoken::EncodingKey::from_secret(JWT_SECRET.expose_secret().as_bytes()),
    )
    .map_err(|e| eyre!("Failed to create token: {}", e))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        let email = Email::parse("test@example.com").unwrap();
        let cookie = generate_auth_cookie(&email).unwrap();
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(
            cookie.value().split('.').count(),
            3,
            "JWT should have 3 parts"
        );
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        let token = "test.token.value";
        let cookie = create_auth_cookie(token.to_string());
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        let email = Email::parse("test@example.com").unwrap();
        let result = generate_auth_token(&email).unwrap();
        assert_eq!(result.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let email = Email::parse("test@example.com").unwrap();
        let token = generate_auth_token(&email).unwrap();

        let result = validate_token(&token).await.unwrap();
        assert_eq!(result.sub, "test@example.com");

        let exp = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();

        assert!(result.exp > exp as usize);
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let token = "invalid_token".to_owned();
        let result = validate_token(&token).await;
        assert!(result.is_err());
    }
}
