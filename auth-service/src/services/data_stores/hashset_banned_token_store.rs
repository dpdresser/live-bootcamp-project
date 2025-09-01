use std::collections::HashSet;

use secrecy::{ExposeSecret, Secret};

use crate::domain::{BannedTokenStore, BannedTokenStoreError};

#[derive(Default)]
pub struct HashSetBannedTokenStore {
    banned_tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashSetBannedTokenStore {
    async fn add_token(&mut self, token: &Secret<String>) -> Result<(), BannedTokenStoreError> {
        if self.banned_tokens.contains(token.expose_secret()) {
            Err(BannedTokenStoreError::TokenAlreadyExists)
        } else {
            self.banned_tokens.insert(token.expose_secret().to_string());
            Ok(())
        }
    }

    async fn is_token_banned(&self, token: &Secret<String>) -> Result<bool, BannedTokenStoreError> {
        Ok(self.banned_tokens.contains(token.expose_secret()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_token_success() {
        let mut store = HashSetBannedTokenStore::default();
        let token = Secret::new("test_token_123".to_string());

        let result = store.add_token(&token).await;

        assert!(result.is_ok());
        assert!(store.banned_tokens.contains(token.expose_secret()));
    }

    #[tokio::test]
    async fn test_add_token_already_exists() {
        let mut store = HashSetBannedTokenStore::default();
        let token = Secret::new("duplicate_token".to_string());

        // Add token first time - should succeed
        let first_result = store.add_token(&token).await;
        assert!(first_result.is_ok());

        // Add same token again - should fail
        let second_result = store.add_token(&token).await;
        assert!(second_result.is_err());
        assert!(matches!(
            second_result.unwrap_err(),
            BannedTokenStoreError::TokenAlreadyExists
        ));
    }

    #[tokio::test]
    async fn test_is_token_banned_true() {
        let mut store = HashSetBannedTokenStore::default();
        let token = Secret::new("banned_token".to_string());

        // Add token to banned list
        store.add_token(&token).await.unwrap();

        // Check if token is banned
        let result = store.is_token_banned(&token).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_is_token_banned_false() {
        let store = HashSetBannedTokenStore::default();
        let token = Secret::new("not_banned_token".to_string());

        // Check if token is banned (it shouldn't be)
        let result = store.is_token_banned(&token).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_multiple_tokens() {
        let mut store = HashSetBannedTokenStore::default();
        let tokens = vec!["token1", "token2", "token3"];

        // Add multiple tokens
        for token in &tokens {
            let result = store.add_token(&Secret::new(token.to_string())).await;
            assert!(result.is_ok());
        }

        // Verify all tokens are banned
        for token in &tokens {
            let result = store.is_token_banned(&Secret::new(token.to_string())).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), true);
        }

        // Verify a non-added token is not banned
        let result = store
            .is_token_banned(&Secret::new("non_existent_token".to_string()))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_empty_token() {
        let mut store = HashSetBannedTokenStore::default();
        let empty_token = Secret::new("".to_string());

        // Add empty token
        let result = store.add_token(&empty_token).await;
        assert!(result.is_ok());

        // Check if empty token is banned
        let result = store.is_token_banned(&empty_token).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_default_store_is_empty() {
        let store = HashSetBannedTokenStore::default();

        // New store should not have any banned tokens
        let result = store
            .is_token_banned(&Secret::new("any_token".to_string()))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);

        // Verify the internal HashSet is empty
        assert!(store.banned_tokens.is_empty());
    }
}
