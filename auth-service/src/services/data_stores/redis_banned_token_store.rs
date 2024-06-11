use std::sync::Arc;

use redis::{Commands, Connection};
use tokio::sync::RwLock;

use crate::{
    domain::data_stores::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

#[derive(Clone)]
pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn store_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        match self.contains_token(&token).await {
            Ok(contains) => {
                if contains {
                    return Err(BannedTokenStoreError::TokenAlreadyExists);
                } else {
                    println!("storing this {token}");
                    let new_key = get_key(&token);
                    let mut conn = self.conn.write().await;

                    match conn.set_ex(new_key, token, TOKEN_TTL_SECONDS.unsigned_abs()) {
                        Ok(()) => Ok(()),
                        Err(_) => Err(BannedTokenStoreError::UnexpectedError),
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let mut conn = self.conn.write().await;

        let key = get_key(token);
        match conn.exists(key) {
            Ok(result) => return Ok(result),

            Err(_) => Err(BannedTokenStoreError::UnexpectedError),
        }
    }
}

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
