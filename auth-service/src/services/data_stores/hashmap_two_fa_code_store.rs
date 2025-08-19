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
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        self.codes.remove(email);
        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        self.codes
            .get(email)
            .cloned()
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_email() -> Email {
        Email::parse("test@example.com").unwrap()
    }

    fn get_test_login_attempt_id() -> LoginAttemptId {
        LoginAttemptId::parse("550e8400-e29b-41d4-a716-446655440000".to_string()).unwrap()
    }

    fn get_test_two_fa_code() -> TwoFACode {
        TwoFACode::parse("123456".to_string()).unwrap()
    }

    #[tokio::test]
    async fn test_add_code_success() {
        let mut store = HashMapTwoFACodeStore::default();
        let email = get_test_email();
        let login_attempt_id = get_test_login_attempt_id();
        let code = get_test_two_fa_code();

        let result = store
            .add_code(email, login_attempt_id.clone(), code.clone())
            .await;

        assert!(result.is_ok());

        // Verify the code was actually stored
        let email_for_get = get_test_email();
        let stored = store.get_code(&email_for_get).await.unwrap();
        assert_eq!(stored.0, login_attempt_id);
        assert_eq!(stored.1, code);
    }

    #[tokio::test]
    async fn test_add_code_overwrites_existing() {
        let mut store = HashMapTwoFACodeStore::default();
        let email1 = get_test_email();
        let email2 = get_test_email();
        let first_attempt_id =
            LoginAttemptId::parse("550e8400-e29b-41d4-a716-446655440001".to_string()).unwrap();
        let second_attempt_id =
            LoginAttemptId::parse("550e8400-e29b-41d4-a716-446655440002".to_string()).unwrap();
        let first_code = TwoFACode::parse("111111".to_string()).unwrap();
        let second_code = TwoFACode::parse("222222".to_string()).unwrap();

        // Add first code
        store
            .add_code(email1, first_attempt_id, first_code)
            .await
            .unwrap();

        // Add second code for same email (should overwrite)
        store
            .add_code(email2, second_attempt_id.clone(), second_code.clone())
            .await
            .unwrap();

        // Verify only the second code exists
        let email_for_get = get_test_email();
        let stored = store.get_code(&email_for_get).await.unwrap();
        assert_eq!(stored.0, second_attempt_id);
        assert_eq!(stored.1, second_code);
    }

    #[tokio::test]
    async fn test_get_code_success() {
        let mut store = HashMapTwoFACodeStore::default();
        let email = get_test_email();
        let login_attempt_id = get_test_login_attempt_id();
        let code = get_test_two_fa_code();

        store
            .add_code(email, login_attempt_id.clone(), code.clone())
            .await
            .unwrap();

        let email_for_get = get_test_email();
        let result = store.get_code(&email_for_get).await;

        assert!(result.is_ok());
        let (stored_attempt_id, stored_code) = result.unwrap();
        assert_eq!(stored_attempt_id, login_attempt_id);
        assert_eq!(stored_code, code);
    }

    #[tokio::test]
    async fn test_get_code_not_found() {
        let store = HashMapTwoFACodeStore::default();
        let email = get_test_email();

        let result = store.get_code(&email).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TwoFACodeStoreError::LoginAttemptIdNotFound
        ));
    }

    #[tokio::test]
    async fn test_remove_code_success() {
        let mut store = HashMapTwoFACodeStore::default();
        let email = get_test_email();
        let login_attempt_id = get_test_login_attempt_id();
        let code = get_test_two_fa_code();

        // Add code first
        store.add_code(email, login_attempt_id, code).await.unwrap();

        // Verify it exists
        let email_for_get1 = get_test_email();
        assert!(store.get_code(&email_for_get1).await.is_ok());

        // Remove the code
        let email_for_remove = get_test_email();
        let result = store.remove_code(&email_for_remove).await;

        assert!(result.is_ok());

        // Verify it's gone
        let email_for_get2 = get_test_email();
        let get_result = store.get_code(&email_for_get2).await;
        assert!(get_result.is_err());
        assert!(matches!(
            get_result.unwrap_err(),
            TwoFACodeStoreError::LoginAttemptIdNotFound
        ));
    }

    #[tokio::test]
    async fn test_remove_code_not_found() {
        let mut store = HashMapTwoFACodeStore::default();
        let email = get_test_email();

        // Remove code that doesn't exist (should succeed)
        let result = store.remove_code(&email).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_emails() {
        let mut store = HashMapTwoFACodeStore::default();
        let email1 = Email::parse("user1@example.com").unwrap();
        let email2 = Email::parse("user2@example.com").unwrap();
        let attempt1 =
            LoginAttemptId::parse("550e8400-e29b-41d4-a716-446655440001".to_string()).unwrap();
        let attempt2 =
            LoginAttemptId::parse("550e8400-e29b-41d4-a716-446655440002".to_string()).unwrap();
        let code1 = TwoFACode::parse("111111".to_string()).unwrap();
        let code2 = TwoFACode::parse("222222".to_string()).unwrap();

        // Add codes for both emails
        store
            .add_code(email1, attempt1.clone(), code1.clone())
            .await
            .unwrap();
        store
            .add_code(email2, attempt2.clone(), code2.clone())
            .await
            .unwrap();

        // Verify both exist independently
        let email1_for_get = Email::parse("user1@example.com").unwrap();
        let email2_for_get = Email::parse("user2@example.com").unwrap();
        let result1 = store.get_code(&email1_for_get).await.unwrap();
        let result2 = store.get_code(&email2_for_get).await.unwrap();

        assert_eq!(result1.0, attempt1);
        assert_eq!(result1.1, code1);
        assert_eq!(result2.0, attempt2);
        assert_eq!(result2.1, code2);

        // Remove one, verify the other still exists
        let email1_for_remove = Email::parse("user1@example.com").unwrap();
        store.remove_code(&email1_for_remove).await.unwrap();

        let email1_for_final_check = Email::parse("user1@example.com").unwrap();
        let email2_for_final_check = Email::parse("user2@example.com").unwrap();
        assert!(store.get_code(&email1_for_final_check).await.is_err());
        assert!(store.get_code(&email2_for_final_check).await.is_ok());
    }

    #[tokio::test]
    async fn test_default_implementation() {
        let store = HashMapTwoFACodeStore::default();
        let email = get_test_email();

        // Default store should be empty
        let result = store.get_code(&email).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TwoFACodeStoreError::LoginAttemptIdNotFound
        ));
    }
}
