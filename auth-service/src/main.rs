use std::sync::Arc;

use auth_service::app_state::AppState;
use auth_service::services::hashmap_user_store::HashmapUserStore;
use auth_service::services::hashset_banned_token_store::HashsetBannedTokenStore;
use auth_service::utils::constants::prod;
use auth_service::Application;

#[tokio::main]
async fn main() {
    let user_store = Arc::new(tokio::sync::RwLock::new(HashmapUserStore::default()));
    let token_store = Arc::new(tokio::sync::RwLock::new(HashsetBannedTokenStore::default()));
    let app_state = AppState::new(user_store, token_store);
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build application");

    app.run().await.expect("Failed to run application");
}
