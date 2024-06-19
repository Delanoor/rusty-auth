use color_eyre::eyre::Context;
use redis::{Commands, Connection};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use color_eyre::eyre::Result;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    Email,
};

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
    #[tracing::instrument(name = "Adding 2FA code to Redis", skip_all)]
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let new_key = get_key(&email);
        let tup = TwoFATuple(
            login_attempt_id.as_ref().expose_secret().to_owned(),
            code.as_ref().expose_secret().to_owned(),
        );
        let serialized_data = serde_json::to_string::<TwoFATuple>(&tup)
            .wrap_err("failed to serialize 2FA tuple")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;

        let _ = self
            .conn
            .write()
            .await
            .set_ex(new_key, serialized_data, TEN_MINUTES_IN_SECONDS)
            .wrap_err("failed to set 2FA code in Redis")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    #[tracing::instrument(name = "Removing 2FA code from Redis", skip_all)]
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(email);

        let _ = self
            .conn
            .write()
            .await
            .del(&key)
            .wrap_err("failed to delete 2FA code from Redis")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;
        Ok(())
    }

    #[tracing::instrument(name = "Retrieving 2FA code from Redis", skip_all)]
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let new_key = get_key(email);

        match self.conn.write().await.get::<_, String>(new_key) {
            Ok(value) => {
                let data: TwoFATuple = serde_json::from_str(&value)
                    .wrap_err("failed to deserialize 2FA tuple")
                    .map_err(TwoFACodeStoreError::UnexpectedError)?;

                let login_attempt_id = LoginAttemptId::parse(Secret::new(data.0))
                    .map_err(TwoFACodeStoreError::UnexpectedError)?;

                let two_fa_code = TwoFACode::parse(Secret::new(data.1))
                    .map_err(TwoFACodeStoreError::UnexpectedError)?;

                Ok((login_attempt_id, two_fa_code))
            }

            Err(_) => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref().expose_secret())
}
