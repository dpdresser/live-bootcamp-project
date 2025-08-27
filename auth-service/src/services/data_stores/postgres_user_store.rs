use std::error::Error;

use argon2::{
    password_hash::SaltString, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
};
use sqlx::PgPool;

use crate::domain::{Email, Password, User, UserStore, UserStoreError};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(user.password().to_string())
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        let email_str: &str = user.email();
        let password_hash_str: &str = password_hash.as_ref();

        sqlx::query!(
            "INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)",
            email_str,
            password_hash_str,
            user.requires_2fa()
        )
        .execute(&self.pool)
        .await
        .map_err(|_| UserStoreError::UserAlreadyExists)?;

        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let row = sqlx::query!("SELECT * FROM users WHERE email = $1", email.as_ref())
            .fetch_one(&self.pool)
            .await
            .map_err(|_| UserStoreError::UserNotFound)?;

        let user = User::new(
            Email::parse(&row.email).map_err(|_| UserStoreError::UnexpectedError)?,
            Password::parse(&row.password_hash).map_err(|_| UserStoreError::UnexpectedError)?,
            row.requires_2fa,
        );

        Ok(user)
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(&self, email: &Email, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;

        verify_password_hash(user.password().to_string(), password.to_string())
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)?;

        Ok(())
    }
}

// Helper function to verify if a given password matches an expected hash
#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    tokio::task::spawn_blocking(move || {
        // Parse the password hash
        let expected_password_hash = PasswordHash::new(&expected_password_hash)
            .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;

        // Verify the password
        Argon2::default()
            .verify_password(password_candidate.as_bytes(), &expected_password_hash)
            .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;

        Ok(())
    })
    .await
    .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?
}

// Helper function to hash passwords before persisting them in the database
#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    tokio::task::spawn_blocking(move || {
        let salt: SaltString = SaltString::generate(&mut rand::thread_rng());

        let password_hash = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(15000, 2, 1, None)?,
        )
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

        Ok(password_hash)
    })
    .await
    .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?
}
