use std::collections::HashMap;

use crate::domain::{Email, Password, User, UserStore, UserStoreError};

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(user.email.as_ref()) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        self.users.insert(user.email.as_ref().to_string(), user);

        Ok(())
    }

    async fn get_user<'a>(&'a self, email: &'a Email) -> Result<&'a User, UserStoreError> {
        if let Some(user) = self.users.get(email.as_ref()) {
            return Ok(user);
        }

        Err(UserStoreError::UserNotFound)
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        if let Some(user) = self.users.get(email.as_ref()) {
            if user.password == *password {
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
                email: Email::parse("test@example.com").unwrap(),
                password: Password::parse("password123").unwrap(),
                requires_2fa: true
            })
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut user_store = HashmapUserStore::default();

        let test_user = User {
            email: Email::parse("test@example.com").unwrap(),
            password: Password::parse("password123").unwrap(),
            requires_2fa: true,
        };

        let _ = user_store.add_user(test_user.clone()).await;

        assert_eq!(
            *(user_store.get_user(&test_user.email).await).unwrap(),
            test_user
        );
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut user_store = HashmapUserStore::default();

        let test_user = User {
            email: Email::parse("test@example.com").unwrap(),
            password: Password::parse("password123").unwrap(),
            requires_2fa: true,
        };

        let _ = user_store.add_user(test_user.clone()).await;

        assert!(user_store
            .validate_user(&test_user.email, &test_user.password)
            .await
            .is_ok());
    }
}
