use std::collections::HashSet;

use crate::domain::data_stores::{BannedTokenStore, BannedTokenStoreError};

#[derive(Default)]
pub struct HashsetBannedTokenStore {
    tokens: HashSet<String>,
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

    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        match self.tokens.get(token) {
            Some(_) => Ok(true),
            None => Err(BannedTokenStoreError::TokenNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_token() {
        let mut store = HashsetBannedTokenStore::default();
        let token = "test".to_string();

        let result = store.store_token(token.clone()).await;

        assert!(result.is_ok());
        assert!(store.contains_token(&token).await.is_ok());
    }

    #[tokio::test]
    async fn test_contains_token() {
        let mut store = HashsetBannedTokenStore::default();
        let token = "test".to_string();
        store.tokens.insert(token.clone());

        let result = store.contains_token(&token).await;
        assert!(result.is_ok());
    }
}
