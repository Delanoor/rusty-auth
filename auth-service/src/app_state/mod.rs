use crate::domain::data_stores::UserStore;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType) -> Self {
        Self { user_store }
    }
}
