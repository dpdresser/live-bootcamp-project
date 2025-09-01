use std::sync::Arc;

use color_eyre::eyre::{eyre, Context};
use redis::{Commands, Connection};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    #[tracing::instrument(name = "Add Code", skip_all)]
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);
        let two_fa_tuple = TwoFATuple(
            login_attempt_id.as_ref().expose_secret().to_string(),
            code.as_ref().expose_secret().to_string(),
        );
        let two_fa_tuple_json = serde_json::to_string(&two_fa_tuple)
            .wrap_err("Failed to serialize 2FA tuple")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;

        self.conn
            .write()
            .await
            .set_ex::<_, _, ()>(key, two_fa_tuple_json, TEN_MINUTES_IN_SECONDS)
            .wrap_err("Failed to set 2FA code in Redis")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    #[tracing::instrument(name = "Remove Code", skip_all)]
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(email);
        self.conn
            .write()
            .await
            .del::<_, ()>(key)
            .wrap_err("Failed to remove 2FA code from Redis")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    #[tracing::instrument(name = "Get Code", skip_all)]
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let key = get_key(email);
        let two_fa_tuple_json = self
            .conn
            .write()
            .await
            .get::<_, String>(key)
            .map_err(|_| TwoFACodeStoreError::LoginAttemptIdNotFound)?;

        let two_fa_tuple: TwoFATuple = serde_json::from_str(&two_fa_tuple_json)
            .wrap_err("Failed to deserialize 2FA tuple")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;

        let login_attempt_id = LoginAttemptId::parse(two_fa_tuple.0.clone())
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(eyre!(e)))?;
        let two_fa_code = TwoFACode::parse(two_fa_tuple.1.clone())
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(eyre!(e)))?;
        Ok((login_attempt_id, two_fa_code))
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{TWO_FA_CODE_PREFIX}{}", email.as_ref().expose_secret())
}
