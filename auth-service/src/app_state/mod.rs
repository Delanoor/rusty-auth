use crate::domain::{
    data_stores::{BannedTokenStore, TwoFACodeStore, UserStore},
    EmailClient,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;
pub type TokenStoreType = Arc<RwLock<dyn BannedTokenStore + Send + Sync>>;
pub type TwoFACodeStoreType = Arc<RwLock<dyn TwoFACodeStore + Send + Sync>>;
pub type EmailClientType = Arc<RwLock<dyn EmailClient + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub token_store: TokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub email_client: EmailClientType,
    pub clean_up_called: bool,
}

impl AppState {
    pub fn new(
        user_store: UserStoreType,
        token_store: TokenStoreType,
        two_fa_code_store: TwoFACodeStoreType,
        email_client: EmailClientType,
        clean_up_called: bool,
    ) -> Self {
        Self {
            user_store,
            token_store,
            two_fa_code_store,
            email_client,
            clean_up_called,
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        if !self.clean_up_called {
            // panic!("Db not cleaned up");
            // println!("+++++++++++++++++++++++++{}", self.clean_up_called)
        }
    }
}
