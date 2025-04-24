use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{BannedTokenStore, UserStore};

pub type BannedTokenStoreType = Arc<RwLock<dyn BannedTokenStore + Send + Sync>>;
pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub banned_token_list: BannedTokenStoreType,
    pub user_store: UserStoreType,
}

impl AppState {
    pub fn new(banned_token_list: BannedTokenStoreType, user_store: UserStoreType) -> Self {
        Self {
            banned_token_list,
            user_store,
        }
    }
}
