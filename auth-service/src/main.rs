use auth_service::{
    app_state::AppState,
    services::{HashmapUserStore, HashsetBannedTokenStore},
    utils::constants::prod,
    Application,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let banned_token_list = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
    let app_state = AppState {
        banned_token_list,
        user_store,
    };

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app")
}
