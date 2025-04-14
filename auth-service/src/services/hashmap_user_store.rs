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
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        self.users.insert(user.email.clone(), user);

        Ok(())
    }

    pub fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        if let Some(user) = self.users.get(email) {
            return Ok(user);
        }

        Err(UserStoreError::UserNotFound)
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
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

        let _ = user_store.add_user(test_user.clone());

        assert_eq!(
            *(user_store.get_user("test@example.com").unwrap()),
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

        let _ = user_store.add_user(test_user.clone());

        assert!(user_store
            .validate_user("test@example.com", "password123")
            .is_ok());
    }
}
