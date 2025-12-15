//! TUI application state

use anyhow::Result;
use crossterm::event::KeyCode;
use std::path::PathBuf;
use zots_core::{TimestampProof, hash_file, hash_from_hex, hash_to_hex};
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
                self.process_stamp().await;
            }
            AppState::Verify => {
                self.process_verify().await;
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

    /// Process stamp input - file path or hex hash
    async fn process_stamp(&mut self) {
        let input = self.input_buffer.trim();
        if input.is_empty() {
            self.result_message = "Please enter a file path or hash".to_string();
            self.result_is_error = true;
            return;
        }

        // Check if wallet is available
        let wallet = match self.wallet.as_mut() {
            Some(w) => w,
            None => {
                self.result_message = "No wallet configured (set ZOTS_SEED)".to_string();
                self.result_is_error = true;
                return;
            }
        };

        // Determine if input is a file or hash
        let path = PathBuf::from(input);
        let (hash_bytes, output_path) = if path.exists() {
            // Hash the file
            self.status_message = "Hashing file...".to_string();
            match hash_file(&path) {
                Ok(h) => {
                    let output = PathBuf::from(format!(
                        "{}.zots",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                    (h, output)
                }
                Err(e) => {
                    self.result_message = format!("Hash error: {}", e);
                    self.result_is_error = true;
                    return;
                }
            }
        } else if input.len() >= 40 {
            // Try parsing as hex hash
            match hash_from_hex(input) {
                Ok(h) => {
                    let output = PathBuf::from(format!("{}.zots", &input[..16]));
                    (h, output)
                }
                Err(e) => {
                    self.result_message = format!("Invalid hash: {}", e);
                    self.result_is_error = true;
                    return;
                }
            }
        } else {
            self.result_message = "File not found and input is not a valid hash".to_string();
            self.result_is_error = true;
            return;
        };

        // Create timestamp transaction
        self.status_message = "Creating transaction...".to_string();
        let tx_result = match wallet.create_timestamp_tx(&hash_bytes).await {
            Ok(r) => r,
            Err(e) => {
                self.result_message = format!("Transaction failed: {}", e);
                self.result_is_error = true;
                self.status_message = "Ready".to_string();
                return;
            }
        };

        // Create pending proof (don't wait for confirmation in TUI)
        let proof = TimestampProof::new(hash_bytes);
        if let Err(e) = proof.save(&output_path) {
            self.result_message = format!("Save error: {}", e);
            self.result_is_error = true;
            self.status_message = "Ready".to_string();
            return;
        }

        self.result_message = format!(
            "TX broadcast: {}\nProof saved: {}",
            &tx_result.txid[..16],
            output_path.display()
        );
        self.result_is_error = false;
        self.status_message = "Ready".to_string();
    }

    /// Process verify input - proof file path
    async fn process_verify(&mut self) {
        let input = self.input_buffer.trim();
        if input.is_empty() {
            self.result_message = "Please enter a proof file path".to_string();
            self.result_is_error = true;
            return;
        }

        let path = PathBuf::from(input);
        if !path.exists() {
            self.result_message = format!("File not found: {}", path.display());
            self.result_is_error = true;
            return;
        }

        // Load proof
        let proof = match TimestampProof::load(&path) {
            Ok(p) => p,
            Err(e) => {
                self.result_message = format!("Load error: {}", e);
                self.result_is_error = true;
                return;
            }
        };

        // Display proof info
        if proof.attestations.is_empty() {
            self.result_message = format!(
                "Hash: {}\nStatus: PENDING (no attestations)",
                hash_to_hex(&proof.hash)
            );
        } else {
            let att = &proof.attestations[0];
            self.result_message = format!(
                "VALID TIMESTAMP\nHash: {}\nBlock: {}\nTime: {}\nTXID: {}",
                &hash_to_hex(&proof.hash)[..16],
                att.block_height,
                att.timestamp().to_rfc3339(),
                &att.txid_hex()[..16]
            );
        }
        self.result_is_error = false;
    }

    /// Get network name for display
    pub fn network_name(&self) -> &str {
        self.config
            .as_ref()
            .map(|c| c.network.name())
            .unwrap_or("unknown")
    }
}
