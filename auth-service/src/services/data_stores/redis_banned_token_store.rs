use std::sync::Arc;

use color_eyre::eyre::Context;
use redis::{Commands, Connection};
use secrecy::{ExposeSecret, Secret};
use tokio::sync::RwLock;

use crate::{
    domain::{BannedTokenStore, BannedTokenStoreError},
    utils::TOKEN_TTL_SECONDS,
};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(name = "Add Token", skip_all)]
    async fn add_token(&mut self, token: &Secret<String>) -> Result<(), BannedTokenStoreError> {
        let key = get_key(token.expose_secret());

        self.conn
            .write()
            .await
            .set_ex::<_, _, ()>(key, true, TOKEN_TTL_SECONDS as u64)
            .map_err(|e| BannedTokenStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "Check Token", skip_all)]
    async fn is_token_banned(&self, token: &Secret<String>) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(token.expose_secret());

        let result = self
            .conn
            .write()
            .await
            .exists::<_, bool>(key)
            .wrap_err("Failed to check if token is banned in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        Ok(result)
    }
}

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{BANNED_TOKEN_KEY_PREFIX}{token}")
}
