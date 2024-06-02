use crate::domain::data_stores::{BannedTokenStore, TwoFACodeStore, UserStore};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;
pub type TokenStoreType = Arc<RwLock<dyn BannedTokenStore + Send + Sync>>;
pub type TwoFACodeStoreType = Arc<RwLock<dyn TwoFACodeStore + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub token_store: TokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
}

impl AppState {
    pub fn new(
        user_store: UserStoreType,
        token_store: TokenStoreType,
        two_fa_code_store: TwoFACodeStoreType,
    ) -> Self {
        Self {
            user_store,
            token_store,
            two_fa_code_store,
        }
    }
}
