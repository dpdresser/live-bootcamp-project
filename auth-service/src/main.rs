use std::sync::Arc;

use auth_service::{app_state::AppState, services::HashMapUserStore, utils::prod, Application};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    println!("🚀 Auth Service starting up! Version: 2024-07-31");
    let user_store = Arc::new(RwLock::new(HashMapUserStore::default()));
    let app_state = AppState::new(user_store);

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build application");

    app.run().await.expect("Failed to run application");
}
