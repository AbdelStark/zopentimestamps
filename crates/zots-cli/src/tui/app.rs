//! TUI application state

use anyhow::Result;
use crossterm::event::KeyCode;
use zots_zcash::{ZcashConfig, ZotsWallet};

/// Current screen/state of the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Menu,
    Stamp,
    Verify,
    Wallet,
}

/// TUI application state
pub struct App {
    /// Current screen
    pub state: AppState,
    /// Wallet instance (if config available)
    pub wallet: Option<ZotsWallet>,
    /// Configuration (if available)
    pub config: Option<ZcashConfig>,
    /// Current block height
    pub block_height: u64,
    /// Wallet balance in zatoshis
    pub balance: u64,
    /// Status message for status bar
    pub status_message: String,
    /// Input buffer for text fields
    pub input_buffer: String,
    /// Result message to display
    pub result_message: String,
    /// Whether result is an error
    pub result_is_error: bool,
}

impl App {
    /// Create new app instance
    pub async fn new() -> Result<Self> {
        let config = ZcashConfig::from_env().ok();

        let (wallet, block_height, balance) = if let Some(ref cfg) = config {
            match ZotsWallet::new(cfg.clone()).await {
                Ok(mut w) => {
                    let _ = w.init_account().await;
                    let h = w.get_block_height().await.unwrap_or(0);
                    let b = w.get_balance().unwrap_or(0);
                    (Some(w), h, b)
                }
                Err(_) => (None, 0, 0),
            }
        } else {
            (None, 0, 0)
        };

        let status = if config.is_some() {
            "Ready".to_string()
        } else {
            "No wallet configured (set ZOTS_SEED)".to_string()
        };

        Ok(Self {
            state: AppState::Menu,
            wallet,
            config,
            block_height,
            balance,
            status_message: status,
            input_buffer: String::new(),
            result_message: String::new(),
            result_is_error: false,
        })
    }

    /// Handle periodic updates
    pub async fn tick(&mut self) -> Result<()> {
        // Could update block height periodically here
        Ok(())
    }

    /// Handle keyboard input in current state
    pub async fn handle_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Enter => {
                self.process_input().await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Process the current input based on state
    async fn process_input(&mut self) -> Result<()> {
        match self.state {
            AppState::Stamp => {
                self.result_message = format!("Would timestamp: {}", self.input_buffer);
                self.result_is_error = false;
                // Full implementation in Phase 7
            }
            AppState::Verify => {
                self.result_message = format!("Would verify: {}", self.input_buffer);
                self.result_is_error = false;
                // Full implementation in Phase 7
            }
            AppState::Wallet => {
                if self.input_buffer.to_lowercase() == "s"
                    || self.input_buffer.to_lowercase() == "sync"
                {
                    self.status_message = "Syncing...".to_string();
                    if let Some(ref mut wallet) = self.wallet {
                        match wallet.sync().await {
                            Ok(_) => {
                                self.block_height = wallet.get_block_height().await.unwrap_or(0);
                                self.balance = wallet.get_balance().unwrap_or(0);
                                self.status_message = "Synced".to_string();
                                self.result_message = "Wallet synced successfully".to_string();
                                self.result_is_error = false;
                            }
                            Err(e) => {
                                self.result_message = format!("Sync failed: {}", e);
                                self.result_is_error = true;
                            }
                        }
                    } else {
                        self.result_message = "No wallet configured".to_string();
                        self.result_is_error = true;
                    }
                }
            }
            AppState::Menu => {}
        }
        self.input_buffer.clear();
        Ok(())
    }

    /// Get network name for display
    pub fn network_name(&self) -> &str {
        self.config
            .as_ref()
            .map(|c| c.network.name())
            .unwrap_or("unknown")
    }
}
