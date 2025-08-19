use std::sync::Arc;

use redis::{Commands, Connection};
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
    async fn add_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        let key = get_key(token);

        self.conn
            .write()
            .await
            .set_ex::<_, _, ()>(key, true, TOKEN_TTL_SECONDS as u64)
            .map_err(|_| BannedTokenStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(token);

        let result = self
            .conn
            .write()
            .await
            .exists::<_, bool>(key)
            .map_err(|_| BannedTokenStoreError::UnexpectedError)?;

        Ok(result)
    }
}

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{BANNED_TOKEN_KEY_PREFIX}{token}")
}
