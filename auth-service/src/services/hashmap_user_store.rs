use std::collections::HashMap;

use crate::domain::{Email, User, UserStore, UserStoreError};

#[derive(Default)]
pub struct HashMapUserStore {
    users: HashMap<String, User>,
}

#[async_trait::async_trait]
impl UserStore for HashMapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(user.email()) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        self.users.insert(user.email().to_owned(), user);
        Ok(())
    }

    async fn get_user(&self, email: &Email) -> Result<&User, UserStoreError> {
        self.users
            .get(email.as_ref())
            .ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(&self, email: &Email, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        if user.password() == password {
            Ok(())
        } else {
            Err(UserStoreError::InvalidCredentials)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Password;

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashMapUserStore::default();
        let user = User::new(
            Email::parse("test@example.com").unwrap(),
            Password::parse("password123!").unwrap(),
            false,
        );
        assert_eq!(store.add_user(user).await.unwrap(), ());
        assert_eq!(store.users.len(), 1);
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashMapUserStore::default();
        let user = User::new(
            Email::parse("test@example.com").unwrap(),
            Password::parse("password123!").unwrap(),
            false,
        );
        store.add_user(user).await.unwrap();
        let retrieved_user = store
            .get_user(&Email::parse("test@example.com").unwrap())
            .await
            .unwrap();
        assert_eq!(retrieved_user.email(), "test@example.com");
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashMapUserStore::default();
        let user = User::new(
            Email::parse("test@example.com").unwrap(),
            Password::parse("password123!").unwrap(),
            false,
        );
        store.add_user(user).await.unwrap();
        assert!(store
            .validate_user(&Email::parse("test@example.com").unwrap(), "password123!")
            .await
            .is_ok());
    }
}
