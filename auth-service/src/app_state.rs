use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{BannedTokenStore, TwoFACodeStore, UserStore};

pub type BannedTokenStoreType = Arc<RwLock<dyn BannedTokenStore + Send + Sync>>;
pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;
pub type TwoFACodeStoreType = Arc<RwLock<dyn TwoFACodeStore + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub banned_token_list: BannedTokenStoreType,
    pub user_store: UserStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
}

impl AppState {
    pub fn new(
        banned_token_list: BannedTokenStoreType,
        user_store: UserStoreType,
        two_fa_code_store: TwoFACodeStoreType,
    ) -> Self {
        Self {
            banned_token_list,
            user_store,
            two_fa_code_store,
        }
    }
}
