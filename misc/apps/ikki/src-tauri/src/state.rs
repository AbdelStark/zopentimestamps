//! Application state management

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::wallet::IkkiWallet;

/// Global application state
pub struct AppState {
    pub wallet: Arc<Mutex<Option<IkkiWallet>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            wallet: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
