use crate::domain::data_stores::{BannedTokenStore, UserStore};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;
pub type TokenStoreType = Arc<RwLock<dyn BannedTokenStore + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub token_store: TokenStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType, token_store: TokenStoreType) -> Self {
        Self {
            user_store,
            token_store,
        }
    }
}
