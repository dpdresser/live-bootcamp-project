use std::collections::HashMap;

use crate::domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};

#[derive(Default)]
pub struct HashMapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashMapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        if self.codes.insert(email, (login_attempt_id, code)).is_none() {
            Ok(())
        } else {
            Err(TwoFACodeStoreError::UnexpectedError)
        }
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        if self.codes.remove(email).is_some() {
            Ok(())
        } else {
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        }
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        if let Some(entry) = self.codes.get(email) {
            Ok((entry.0.clone(), entry.1.clone()))
        } else {
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_2fa_code() {
        let mut two_fa_code_store = HashMapTwoFACodeStore::default();

        assert!(two_fa_code_store
            .add_code(
                Email::parse("test@example.com".to_string()).unwrap(),
                LoginAttemptId::default(),
                TwoFACode::default()
            )
            .await
            .is_ok())
    }

    #[tokio::test]
    async fn test_remove_2fa_code() {
        let mut two_fa_code_store = HashMapTwoFACodeStore::default();

        let _ = two_fa_code_store
            .add_code(
                Email::parse("test@example.com".to_string()).unwrap(),
                LoginAttemptId::default(),
                TwoFACode::default(),
            )
            .await;

        assert!(two_fa_code_store
            .remove_code(&Email::parse("test@example.com".to_string()).unwrap())
            .await
            .is_ok())
    }

    #[tokio::test]
    async fn test_get_2fa_code() {
        let mut two_fa_code_store = HashMapTwoFACodeStore::default();

        let email = Email::parse("test@example.com".to_string()).unwrap();
        let id = LoginAttemptId::default();
        let code = TwoFACode::default();

        let _ = two_fa_code_store
            .add_code(email.clone(), id.clone(), code.clone())
            .await;

        assert!(two_fa_code_store.get_code(&email).await.is_ok());
        let result = two_fa_code_store.get_code(&email).await.unwrap();
        assert_eq!(result.0, id);
        assert_eq!(result.1, code);
    }
}
