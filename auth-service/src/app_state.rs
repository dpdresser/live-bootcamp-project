use std::sync::Arc;

use tokio::sync::RwLock;

use crate::services::HashMapUserStore;

pub type UserStoreType = Arc<RwLock<HashMapUserStore>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType) -> Self {
        Self { user_store }
    }
}
