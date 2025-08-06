use std::collections::HashSet;

use crate::domain::{BannedTokenStore, BannedTokenStoreError};

#[derive(Default)]
pub struct HashSetBannedTokenStore {
    banned_tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashSetBannedTokenStore {
    async fn add_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        if self.banned_tokens.contains(token) {
            Err(BannedTokenStoreError::TokenAlreadyExists)
        } else {
            self.banned_tokens.insert(token.to_string());
            Ok(())
        }
    }

    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.banned_tokens.contains(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_token_success() {
        let mut store = HashSetBannedTokenStore::default();
        let token = "test_token_123";

        let result = store.add_token(token).await;

        assert!(result.is_ok());
        assert!(store.banned_tokens.contains(token));
    }

    #[tokio::test]
    async fn test_add_token_already_exists() {
        let mut store = HashSetBannedTokenStore::default();
        let token = "duplicate_token";

        // Add token first time - should succeed
        let first_result = store.add_token(token).await;
        assert!(first_result.is_ok());

        // Add same token again - should fail
        let second_result = store.add_token(token).await;
        assert!(second_result.is_err());
        assert!(matches!(
            second_result.unwrap_err(),
            BannedTokenStoreError::TokenAlreadyExists
        ));
    }

    #[tokio::test]
    async fn test_is_token_banned_true() {
        let mut store = HashSetBannedTokenStore::default();
        let token = "banned_token";

        // Add token to banned list
        store.add_token(token).await.unwrap();

        // Check if token is banned
        let result = store.is_token_banned(token).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_is_token_banned_false() {
        let store = HashSetBannedTokenStore::default();
        let token = "not_banned_token";

        // Check if token is banned (it shouldn't be)
        let result = store.is_token_banned(token).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_multiple_tokens() {
        let mut store = HashSetBannedTokenStore::default();
        let tokens = vec!["token1", "token2", "token3"];

        // Add multiple tokens
        for token in &tokens {
            let result = store.add_token(token).await;
            assert!(result.is_ok());
        }

        // Verify all tokens are banned
        for token in &tokens {
            let result = store.is_token_banned(token).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), true);
        }

        // Verify a non-added token is not banned
        let result = store.is_token_banned("non_existent_token").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_empty_token() {
        let mut store = HashSetBannedTokenStore::default();
        let empty_token = "";

        // Add empty token
        let result = store.add_token(empty_token).await;
        assert!(result.is_ok());

        // Check if empty token is banned
        let result = store.is_token_banned(empty_token).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_default_store_is_empty() {
        let store = HashSetBannedTokenStore::default();

        // New store should not have any banned tokens
        let result = store.is_token_banned("any_token").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);

        // Verify the internal HashSet is empty
        assert!(store.banned_tokens.is_empty());
    }
}
