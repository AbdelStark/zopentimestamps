//! TUI application state
//!
//! Manages the state machine for the TUI application, including:
//! - Navigation between screens (Menu, Stamp, Verify, Wallet)
//! - Multi-step input flows with progress tracking
//! - Async operation phases with spinner animation

use anyhow::Result;
use crossterm::event::KeyCode;
use std::path::PathBuf;
use zots_core::{TimestampProof, ZcashAttestation, hash_file, hash_from_hex, hash_to_hex};
use zots_zcash::{ZcashConfig, ZotsWallet};

/// Spinner frames for animated progress indicator
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Current screen/state of the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Menu,
    Stamp,
    Verify,
    Wallet,
}

/// Phase of an async operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationPhase {
    /// Waiting for user input
    Input,
    /// Hashing a file
    Hashing,
    /// Syncing wallet with blockchain
    Syncing,
    /// Creating and broadcasting transaction
    Broadcasting,
    /// Waiting for block confirmation
    WaitingConfirmation { txid: String, attempts: u32 },
    /// Operation completed successfully
    Complete,
    /// Operation failed
    Failed,
}

/// Input step for verify flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyStep {
    /// Enter file path or hash to verify
    FileOrHash,
    /// Enter proof file path
    ProofPath,
    /// Verifying on blockchain
    Verifying,
    /// Showing results
    Results,
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
    /// Current phase of async operation
    pub operation_phase: OperationPhase,
    /// Spinner frame index for animation
    pub spinner_frame: usize,
    /// Current step in verify flow
    pub verify_step: VerifyStep,
    /// Stored file/hash input for verify flow
    pub verify_file_input: String,
    /// Stored hash bytes for verify (computed from file or parsed from hex)
    pub verify_hash: Option<[u8; 32]>,
    /// Stamp result details for display
    pub stamp_result: Option<StampResult>,
    /// Verify result details for display
    pub verify_result: Option<VerifyResult>,
}

/// Result of a successful stamp operation
#[derive(Debug, Clone)]
pub struct StampResult {
    pub hash: String,
    pub txid: String,
    pub block_height: u32,
    pub block_time: u64,
    pub output_path: String,
    pub compact: String,
}

/// Result of a verification operation
#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub hash: String,
    pub valid: bool,
    pub network: String,
    pub block_height: u32,
    pub timestamp: String,
    pub txid: String,
    pub explorer_link: String,
    pub error: Option<String>,
    pub file_hash_matches: Option<bool>,
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
            operation_phase: OperationPhase::Input,
            spinner_frame: 0,
            verify_step: VerifyStep::FileOrHash,
            verify_file_input: String::new(),
            verify_hash: None,
            stamp_result: None,
            verify_result: None,
        })
    }

    /// Handle periodic updates
    pub async fn tick(&mut self) -> Result<()> {
        // Advance spinner for animation during async operations
        if !matches!(self.operation_phase, OperationPhase::Input | OperationPhase::Complete | OperationPhase::Failed) {
            self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
        }
        Ok(())
    }

    /// Get current spinner character
    pub fn spinner(&self) -> &'static str {
        SPINNER_FRAMES[self.spinner_frame]
    }

    /// Reset state when returning to menu
    pub fn reset_state(&mut self) {
        self.input_buffer.clear();
        self.result_message.clear();
        self.result_is_error = false;
        self.operation_phase = OperationPhase::Input;
        self.verify_step = VerifyStep::FileOrHash;
        self.verify_file_input.clear();
        self.verify_hash = None;
        self.stamp_result = None;
        self.verify_result = None;
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
                // Only process if in input phase
                if matches!(self.operation_phase, OperationPhase::Input) {
                    self.process_stamp().await;
                }
            }
            AppState::Verify => {
                self.process_verify().await;
            }
            AppState::Wallet => {
                if self.input_buffer.to_lowercase() == "s"
                    || self.input_buffer.to_lowercase() == "sync"
                {
                    self.operation_phase = OperationPhase::Syncing;
                    self.status_message = "Syncing...".to_string();
                    if let Some(ref mut wallet) = self.wallet {
                        match wallet.sync().await {
                            Ok(_) => {
                                self.block_height = wallet.get_block_height().await.unwrap_or(0);
                                self.balance = wallet.get_balance().unwrap_or(0);
                                self.status_message = "Synced".to_string();
                                self.result_message = "Wallet synced successfully".to_string();
                                self.result_is_error = false;
                                self.operation_phase = OperationPhase::Complete;
                            }
                            Err(e) => {
                                self.result_message = format!("Sync failed: {}", e);
                                self.result_is_error = true;
                                self.operation_phase = OperationPhase::Failed;
                            }
                        }
                    } else {
                        self.result_message = "No wallet configured".to_string();
                        self.result_is_error = true;
                        self.operation_phase = OperationPhase::Failed;
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
        let input = self.input_buffer.trim().to_string();
        if input.is_empty() {
            self.result_message = "Please enter a file path or hash".to_string();
            self.result_is_error = true;
            return;
        }

        // Check if wallet is available
        if self.wallet.is_none() {
            self.result_message = "No wallet configured (set ZOTS_SEED)".to_string();
            self.result_is_error = true;
            self.operation_phase = OperationPhase::Failed;
            return;
        }

        // Determine if input is a file or hash
        let path = PathBuf::from(&input);
        let (hash_bytes, output_path) = if path.exists() {
            // Hash the file
            self.operation_phase = OperationPhase::Hashing;
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
                    self.operation_phase = OperationPhase::Failed;
                    return;
                }
            }
        } else if input.len() >= 40 {
            // Try parsing as hex hash
            match hash_from_hex(&input) {
                Ok(h) => {
                    let output = PathBuf::from(format!("{}.zots", &input[..16]));
                    (h, output)
                }
                Err(e) => {
                    self.result_message = format!("Invalid hash: {}", e);
                    self.result_is_error = true;
                    self.operation_phase = OperationPhase::Failed;
                    return;
                }
            }
        } else {
            self.result_message = "File not found and input is not a valid hash".to_string();
            self.result_is_error = true;
            self.operation_phase = OperationPhase::Failed;
            return;
        };

        let hash_hex = hash_to_hex(&hash_bytes);

        // Sync wallet
        self.operation_phase = OperationPhase::Syncing;
        self.status_message = "Syncing wallet...".to_string();

        let wallet = self.wallet.as_mut().unwrap();
        if let Err(e) = wallet.sync().await {
            self.result_message = format!("Sync failed: {}", e);
            self.result_is_error = true;
            self.operation_phase = OperationPhase::Failed;
            return;
        }

        // Update balance after sync
        self.block_height = wallet.get_block_height().await.unwrap_or(0);
        self.balance = wallet.get_balance().unwrap_or(0);

        // Create timestamp transaction
        self.operation_phase = OperationPhase::Broadcasting;
        self.status_message = "Creating and broadcasting transaction...".to_string();

        let tx_result = match wallet.create_timestamp_tx(&hash_bytes).await {
            Ok(r) => r,
            Err(e) => {
                self.result_message = format!("Transaction failed: {}", e);
                self.result_is_error = true;
                self.operation_phase = OperationPhase::Failed;
                self.status_message = "Transaction failed".to_string();
                return;
            }
        };

        let txid = tx_result.txid.clone();

        // Wait for confirmation
        self.operation_phase = OperationPhase::WaitingConfirmation {
            txid: txid.clone(),
            attempts: 0,
        };
        self.status_message = format!("Waiting for confirmation (TXID: {}...)", &txid[..12]);

        let confirmation = match wallet.wait_confirmation(&txid, 10).await {
            Ok(c) => c,
            Err(e) => {
                // Save pending proof even if confirmation fails
                let proof = TimestampProof::new(hash_bytes);
                let _ = proof.save(&output_path);

                self.result_message = format!(
                    "TX broadcast but confirmation timed out: {}\nPending proof saved: {}",
                    e,
                    output_path.display()
                );
                self.result_is_error = true;
                self.operation_phase = OperationPhase::Failed;
                self.status_message = "Confirmation timeout".to_string();
                return;
            }
        };

        // Get network from config
        let network = self.config.as_ref().map(|c| c.network).unwrap_or(zots_core::Network::Testnet);

        // Create confirmed proof with attestation
        let mut proof = TimestampProof::new(hash_bytes);
        proof.add_attestation(ZcashAttestation::new(
            network,
            tx_result.txid_bytes,
            confirmation.block_height,
            confirmation.block_time,
            0,
        ));

        // Save proof
        if let Err(e) = proof.save(&output_path) {
            self.result_message = format!("Save error: {}", e);
            self.result_is_error = true;
            self.operation_phase = OperationPhase::Failed;
            return;
        }

        // Generate compact format
        let compact = proof.to_compact().unwrap_or_else(|_| "Error generating compact format".to_string());

        // Store result for display
        self.stamp_result = Some(StampResult {
            hash: hash_hex,
            txid: txid.clone(),
            block_height: confirmation.block_height,
            block_time: confirmation.block_time as u64,
            output_path: output_path.display().to_string(),
            compact,
        });

        self.result_is_error = false;
        self.operation_phase = OperationPhase::Complete;
        self.status_message = format!("Confirmed in block {}", confirmation.block_height);
    }

    /// Process verify input - multi-step: file/hash, then proof path
    async fn process_verify(&mut self) {
        let input = self.input_buffer.trim().to_string();

        match self.verify_step {
            VerifyStep::FileOrHash => {
                if input.is_empty() {
                    self.result_message = "Please enter a file path or hash to verify".to_string();
                    self.result_is_error = true;
                    return;
                }

                // Determine if input is a file or hash
                let path = PathBuf::from(&input);
                if path.exists() {
                    // Hash the file
                    match hash_file(&path) {
                        Ok(h) => {
                            self.verify_hash = Some(h);
                            self.verify_file_input = input;
                            self.verify_step = VerifyStep::ProofPath;
                            self.result_message.clear();
                            self.result_is_error = false;
                        }
                        Err(e) => {
                            self.result_message = format!("Hash error: {}", e);
                            self.result_is_error = true;
                        }
                    }
                } else if input.len() >= 40 {
                    // Try parsing as hex hash
                    match hash_from_hex(&input) {
                        Ok(h) => {
                            self.verify_hash = Some(h);
                            self.verify_file_input = input;
                            self.verify_step = VerifyStep::ProofPath;
                            self.result_message.clear();
                            self.result_is_error = false;
                        }
                        Err(e) => {
                            self.result_message = format!("Invalid hash: {}", e);
                            self.result_is_error = true;
                        }
                    }
                } else {
                    self.result_message =
                        "File not found and input is not a valid hash".to_string();
                    self.result_is_error = true;
                }
            }
            VerifyStep::ProofPath => {
                if input.is_empty() {
                    self.result_message = "Please enter a proof file path (.zots)".to_string();
                    self.result_is_error = true;
                    return;
                }

                let path = PathBuf::from(&input);
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

                // Check hash matches
                let proof_hash_bytes = match proof.hash_bytes() {
                    Ok(h) => h,
                    Err(e) => {
                        self.result_message = format!("Invalid proof hash: {}", e);
                        self.result_is_error = true;
                        return;
                    }
                };

                let file_hash_matches = self.verify_hash.map(|h| h == proof_hash_bytes);
                if let Some(matches) = file_hash_matches
                    && !matches
                {
                    self.verify_result = Some(VerifyResult {
                        hash: proof.hash.clone(),
                        valid: false,
                        network: String::new(),
                        block_height: 0,
                        timestamp: String::new(),
                        txid: String::new(),
                        explorer_link: String::new(),
                        error: Some("Hash does NOT match the provided file/hash!".to_string()),
                        file_hash_matches: Some(false),
                    });
                    self.verify_step = VerifyStep::Results;
                    self.operation_phase = OperationPhase::Failed;
                    return;
                }

                // Check if proof has attestations
                if proof.attestations.is_empty() {
                    self.verify_result = Some(VerifyResult {
                        hash: proof.hash.clone(),
                        valid: false,
                        network: String::new(),
                        block_height: 0,
                        timestamp: String::new(),
                        txid: String::new(),
                        explorer_link: String::new(),
                        error: Some("Proof is pending (no attestations yet)".to_string()),
                        file_hash_matches,
                    });
                    self.verify_step = VerifyStep::Results;
                    self.operation_phase = OperationPhase::Complete;
                    return;
                }

                // Verify on blockchain
                self.verify_step = VerifyStep::Verifying;
                self.operation_phase = OperationPhase::Syncing;
                self.status_message = "Verifying against blockchain...".to_string();

                // Check if wallet is available
                if self.wallet.is_none() {
                    // Can't verify on-chain without wallet, show proof info only
                    let att = &proof.attestations[0];
                    self.verify_result = Some(VerifyResult {
                        hash: proof.hash.clone(),
                        valid: true, // Assume valid since we can't verify
                        network: att.network.to_string(),
                        block_height: att.block_height,
                        timestamp: att.timestamp().to_rfc3339(),
                        txid: att.txid_hex().to_string(),
                        explorer_link: att.explorer_link(),
                        error: Some("Cannot verify on-chain (no wallet configured)".to_string()),
                        file_hash_matches,
                    });
                    self.verify_step = VerifyStep::Results;
                    self.operation_phase = OperationPhase::Complete;
                    return;
                }

                let att = &proof.attestations[0];
                let txid_bytes = match att.txid_bytes() {
                    Ok(b) => b,
                    Err(e) => {
                        self.result_message = format!("Invalid TXID: {}", e);
                        self.result_is_error = true;
                        self.operation_phase = OperationPhase::Failed;
                        return;
                    }
                };

                let wallet = self.wallet.as_mut().unwrap();
                let _ = wallet.init_account().await;

                let result = wallet
                    .verify_timestamp_tx(&txid_bytes, &proof_hash_bytes, Some(att.block_height))
                    .await;

                match result {
                    Ok(vr) => {
                        self.verify_result = Some(VerifyResult {
                            hash: proof.hash.clone(),
                            valid: vr.valid,
                            network: att.network.to_string(),
                            block_height: att.block_height,
                            timestamp: att.timestamp().to_rfc3339(),
                            txid: att.txid_hex().to_string(),
                            explorer_link: att.explorer_link(),
                            error: vr.error,
                            file_hash_matches,
                        });
                        self.operation_phase = if vr.valid {
                            OperationPhase::Complete
                        } else {
                            OperationPhase::Failed
                        };
                    }
                    Err(e) => {
                        self.verify_result = Some(VerifyResult {
                            hash: proof.hash.clone(),
                            valid: false,
                            network: att.network.to_string(),
                            block_height: att.block_height,
                            timestamp: att.timestamp().to_rfc3339(),
                            txid: att.txid_hex().to_string(),
                            explorer_link: att.explorer_link(),
                            error: Some(format!("Verification error: {}", e)),
                            file_hash_matches,
                        });
                        self.operation_phase = OperationPhase::Failed;
                    }
                }

                self.verify_step = VerifyStep::Results;
                self.status_message = "Verification complete".to_string();
            }
            VerifyStep::Verifying | VerifyStep::Results => {
                // Do nothing - operation in progress or already showing results
            }
        }
    }

    /// Get network name for display
    pub fn network_name(&self) -> &str {
        self.config
            .as_ref()
            .map(|c| c.network.name())
            .unwrap_or("unknown")
    }
}
