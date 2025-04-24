use std::collections::HashSet;

use crate::domain::{BannedTokenStore, TokenStoreError};

#[derive(Default)]
pub struct HashsetBannedTokenStore {
    banned_tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn ban_token(&mut self, token: String) -> Result<(), TokenStoreError> {
        if self.banned_tokens.insert(token) {
            Ok(())
        } else {
            Err(TokenStoreError::TokenBanned)
        }
    }

    async fn check_token(&self, token: &str) -> Result<(), TokenStoreError> {
        if self.banned_tokens.contains(token) {
            Err(TokenStoreError::TokenBanned)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ban_token() {
        let mut banned_token_store = HashsetBannedTokenStore::default();

        let token = "test_token".to_string();

        assert!(banned_token_store.ban_token(token).await.is_ok());
    }

    #[tokio::test]
    async fn test_check_token() {
        let mut banned_token_store = HashsetBannedTokenStore::default();

        let token = "test_token".to_string();

        assert!(banned_token_store.check_token(&token).await.is_ok());
        assert!(banned_token_store.ban_token(token.clone()).await.is_ok());
        assert_eq!(
            banned_token_store.check_token(&token).await,
            Err(TokenStoreError::TokenBanned)
        );
    }
}
