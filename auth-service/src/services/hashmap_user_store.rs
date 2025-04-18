use std::collections::HashMap;

use crate::domain::{User, UserStore, UserStoreError};

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        self.users.insert(user.email.clone(), user);

        Ok(())
    }

    async fn get_user<'a>(&'a self, email: &str) -> Result<&'a User, UserStoreError> {
        if let Some(user) = self.users.get(email) {
            return Ok(user);
        }

        Err(UserStoreError::UserNotFound)
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        if let Some(user) = self.users.get(email) {
            if user.password == password {
                return Ok(());
            } else {
                return Err(UserStoreError::InvalidCredentials);
            }
        }

        Err(UserStoreError::UserNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut user_store = HashmapUserStore::default();

        assert!(user_store
            .add_user(User {
                email: "test@example.com".to_string(),
                password: "password123".to_string(),
                requires_2fa: true
            })
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut user_store = HashmapUserStore::default();

        let test_user = User {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            requires_2fa: true,
        };

        let _ = user_store.add_user(test_user.clone()).await;

        assert_eq!(
            *(user_store.get_user("test@example.com").await).unwrap(),
            test_user
        );
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut user_store = HashmapUserStore::default();

        let test_user = User {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            requires_2fa: true,
        };

        let _ = user_store.add_user(test_user.clone()).await;

        assert!(user_store
            .validate_user("test@example.com", "password123")
            .await
            .is_ok());
    }
}
