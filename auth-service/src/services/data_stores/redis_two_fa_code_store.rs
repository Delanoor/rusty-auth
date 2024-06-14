use std::sync::Arc;

use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

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
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let new_key = get_key(&email);
        let tup = TwoFATuple(
            login_attempt_id.as_ref().to_owned(),
            code.as_ref().to_owned(),
        );
        let tup_serialized = serde_json::to_string::<TwoFATuple>(&tup)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        let mut conn: tokio::sync::RwLockWriteGuard<Connection> = self.conn.write().await;
        match conn.set_ex(new_key, tup_serialized, TEN_MINUTES_IN_SECONDS) {
            Ok(()) => Ok(()),
            Err(_) => Err(TwoFACodeStoreError::UnexpectedError),
        }
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        match self.get_code(email).await {
            Ok(_) => {
                let mut conn: tokio::sync::RwLockWriteGuard<Connection> = self.conn.write().await;
                let key = get_key(email);
                match conn.del(key) {
                    Ok(()) => Ok(()),
                    Err(_) => Err(TwoFACodeStoreError::UnexpectedError),
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let mut conn = self.conn.write().await;

        let new_key = get_key(email);

        match conn.get::<_, String>(new_key) {
            Ok(result) => {
                let tuple = serde_json::from_str(&result);
                match tuple {
                    Ok((login_attempt_id, two_fa_code)) => {
                        let attempt_id = LoginAttemptId::parse(login_attempt_id)
                            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
                        let code = TwoFACode::parse(two_fa_code)
                            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

                        Ok((attempt_id, code))
                    }
                    Err(_) => Err(TwoFACodeStoreError::UnexpectedError),
                }
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
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
