use std::collections::HashMap;

use crate::domain::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Default)]
pub struct HashMapUserStore {
    users: HashMap<String, User>,
}

impl HashMapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(user.email()) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        self.users.insert(user.email().to_owned(), user);
        Ok(())
    }

    pub fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        self.users.get(email).ok_or(UserStoreError::UserNotFound)
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email)?;
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

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashMapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password123".to_string(),
            false,
        );
        assert_eq!(store.add_user(user).unwrap(), ());
        assert_eq!(store.users.len(), 1);
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashMapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password123".to_string(),
            false,
        );
        store.add_user(user).unwrap();
        let retrieved_user = store.get_user("test@example.com").unwrap();
        assert_eq!(retrieved_user.email(), "test@example.com");
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashMapUserStore::default();
        let user = User::new(
            "test@example.com".to_string(),
            "password123".to_string(),
            false,
        );
        store.add_user(user).unwrap();
        assert!(store
            .validate_user("test@example.com", "password123")
            .is_ok());
    }
}
