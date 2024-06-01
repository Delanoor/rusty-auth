use std::collections::HashSet;

use crate::domain::data_stores::{BannedTokenStore, BannedTokenStoreError};

pub struct HashsetBannedTokenStore {
    tokens: HashSet<String>,
}

impl Default for HashsetBannedTokenStore {
    fn default() -> Self {
        Self {
            tokens: HashSet::new(),
        }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn store_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        match self.tokens.get(&token) {
            Some(_) => return Err(BannedTokenStoreError::TokenAlreadyExists),
            None => {
                self.tokens.insert(token);
                return Ok(());
            }
        }
    }

    async fn get_token(&self, token: &str) -> Result<(), BannedTokenStoreError> {
        match self.tokens.get(token) {
            Some(_) => Ok(()),
            None => Err(BannedTokenStoreError::TokenNotFound),
        }
    }
}
