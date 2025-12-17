//! TUI application state
//!
//! Manages the state machine for the TUI application, including:
//! - Navigation between screens (Menu, Stamp, Verify, Wallet)
//! - Multi-step input flows with progress tracking
//! - Async operation phases with spinner animation
//! - Background task execution for non-blocking UI

use anyhow::Result;
use crossterm::event::KeyCode;
use std::path::PathBuf;
use tokio::sync::mpsc;
use zots_core::{
    HashAlgorithm, TimestampProof, ZcashAttestation, hash_file_with, hash_from_hex_with,
    hash_to_hex,
};
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
    /// Syncing wallet with blockchain
    Syncing,
    /// Creating and broadcasting transaction
    Broadcasting,
    /// Waiting for block confirmation
    WaitingConfirmation { txid: String },
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

/// Input kind for verification flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyInputKind {
    File,
    Hash,
}

/// Messages sent from background tasks to update UI
#[derive(Debug, Clone)]
pub enum TaskMessage {
    /// Update operation phase
    Phase(OperationPhase),
    /// Update status message
    Status(String),
    /// Stamp operation completed successfully
    StampComplete(StampResult),
    /// Stamp operation failed
    StampFailed(String),
    /// Verify operation completed
    VerifyComplete(VerifyResult),
    /// Verify operation failed
    VerifyFailed(String),
    /// Wallet sync completed (explicit user action)
    SyncComplete { block_height: u64, balance: u64 },
    /// Wallet sync failed
    SyncFailed(String),
    /// Initial background sync completed (silent, just updates balance)
    InitialSyncComplete { block_height: u64, balance: u64 },
    /// Initial sync failed (silent)
    InitialSyncFailed,
}

/// TUI application state
pub struct App {
    /// Current screen
    pub state: AppState,
    /// Configuration (if available)
    pub config: Option<ZcashConfig>,
    /// Selected hash algorithm for stamping
    pub hash_algorithm: HashAlgorithm,
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
    /// Whether verify input was a file path or hash string
    pub verify_input_kind: Option<VerifyInputKind>,
    /// Stored hash bytes for verify (computed from file or parsed from hex)
    pub verify_hash: Option<[u8; 32]>,
    /// Stamp result details for display
    pub stamp_result: Option<StampResult>,
    /// Verify result details for display
    pub verify_result: Option<VerifyResult>,
    /// Whether QR overlay is showing
    pub qr_visible: bool,
    /// Cached compact proof for QR rendering
    pub qr_data: Option<String>,
    /// Channel receiver for background task updates
    task_rx: mpsc::Receiver<TaskMessage>,
    /// Channel sender for background tasks (cloned when spawning)
    task_tx: mpsc::Sender<TaskMessage>,
    /// Whether a background task is currently running
    pub task_running: bool,
}

/// Result of a successful stamp operation
#[derive(Debug, Clone)]
pub struct StampResult {
    pub hash: String,
    pub algorithm: HashAlgorithm,
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
    pub algorithm: HashAlgorithm,
    pub compact: String,
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

        // Create channel for background task communication
        let (task_tx, task_rx) = mpsc::channel(32);

        let (status, task_running) = if config.is_some() {
            // Spawn initial sync task to get balance
            let tx = task_tx.clone();
            let cfg = config.clone().unwrap();
            tokio::spawn(async move {
                run_initial_sync_task(tx, cfg).await;
            });
            ("Syncing wallet...".to_string(), true)
        } else {
            ("No wallet configured (set ZOTS_SEED)".to_string(), false)
        };

        Ok(Self {
            state: AppState::Menu,
            config,
            hash_algorithm: HashAlgorithm::Sha256,
            block_height: 0,
            balance: 0,
            status_message: status,
            input_buffer: String::new(),
            result_message: String::new(),
            result_is_error: false,
            operation_phase: if task_running {
                OperationPhase::Syncing
            } else {
                OperationPhase::Input
            },
            spinner_frame: 0,
            verify_step: VerifyStep::FileOrHash,
            verify_file_input: String::new(),
            verify_input_kind: None,
            verify_hash: None,
            stamp_result: None,
            verify_result: None,
            qr_visible: false,
            qr_data: None,
            task_rx,
            task_tx,
            task_running,
        })
    }

    /// Handle periodic updates - polls for background task messages
    pub async fn tick(&mut self) -> Result<()> {
        // Advance spinner for animation during async operations
        if self.task_running {
            self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
        }

        // Poll for messages from background tasks (non-blocking)
        while let Ok(msg) = self.task_rx.try_recv() {
            match msg {
                TaskMessage::Phase(phase) => {
                    self.operation_phase = phase;
                }
                TaskMessage::Status(status) => {
                    self.status_message = status;
                }
                TaskMessage::StampComplete(result) => {
                    self.stamp_result = Some(result);
                    self.operation_phase = OperationPhase::Complete;
                    self.qr_data = self.stamp_result.as_ref().map(|r| r.compact.clone());
                    self.qr_visible = false;
                    self.task_running = false;
                }
                TaskMessage::StampFailed(error) => {
                    self.result_message = error;
                    self.result_is_error = true;
                    self.operation_phase = OperationPhase::Failed;
                    self.qr_data = None;
                    self.qr_visible = false;
                    self.task_running = false;
                }
                TaskMessage::VerifyComplete(result) => {
                    self.verify_result = Some(result);
                    self.qr_data = self.verify_result.as_ref().map(|r| r.compact.clone());
                    self.qr_visible = false;
                    self.verify_step = VerifyStep::Results;
                    self.operation_phase = if self
                        .verify_result
                        .as_ref()
                        .map(|r| r.valid)
                        .unwrap_or(false)
                    {
                        OperationPhase::Complete
                    } else {
                        OperationPhase::Failed
                    };
                    self.task_running = false;
                }
                TaskMessage::VerifyFailed(error) => {
                    self.result_message = error;
                    self.result_is_error = true;
                    self.operation_phase = OperationPhase::Failed;
                    self.qr_data = None;
                    self.qr_visible = false;
                    self.task_running = false;
                }
                TaskMessage::SyncComplete {
                    block_height,
                    balance,
                } => {
                    self.block_height = block_height;
                    self.balance = balance;
                    self.status_message = "Synced".to_string();
                    self.result_message = "Wallet synced successfully".to_string();
                    self.result_is_error = false;
                    self.operation_phase = OperationPhase::Complete;
                    self.task_running = false;
                }
                TaskMessage::SyncFailed(error) => {
                    self.result_message = format!("Sync failed: {error}");
                    self.result_is_error = true;
                    self.operation_phase = OperationPhase::Failed;
                    self.task_running = false;
                }
                TaskMessage::InitialSyncComplete {
                    block_height,
                    balance,
                } => {
                    // Silent update - just set values and return to ready state
                    self.block_height = block_height;
                    self.balance = balance;
                    self.status_message = "Ready".to_string();
                    self.operation_phase = OperationPhase::Input;
                    self.task_running = false;
                }
                TaskMessage::InitialSyncFailed => {
                    // Silent failure - just return to ready state
                    self.status_message = "Ready (sync failed)".to_string();
                    self.operation_phase = OperationPhase::Input;
                    self.task_running = false;
                }
            }
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
        self.verify_input_kind = None;
        self.verify_hash = None;
        self.stamp_result = None;
        self.verify_result = None;
        self.task_running = false;
        self.qr_visible = false;
        self.qr_data = None;
    }

    /// Toggle between supported hash algorithms for stamping
    fn toggle_hash_algorithm(&mut self) {
        self.hash_algorithm = match self.hash_algorithm {
            HashAlgorithm::Sha256 => HashAlgorithm::Blake3,
            HashAlgorithm::Blake3 => HashAlgorithm::Sha256,
        };
        self.result_message = format!("Using {}", self.hash_algorithm.name());
        self.result_is_error = false;
    }

    /// Handle keyboard input in current state
    pub fn handle_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                if self.can_toggle_qr() {
                    self.qr_visible = !self.qr_visible;
                    return Ok(());
                }
                if !self.task_running {
                    self.input_buffer.push('q');
                }
            }
            KeyCode::Tab => {
                if matches!(self.state, AppState::Stamp) && !self.task_running {
                    self.toggle_hash_algorithm();
                }
            }
            KeyCode::Char(c) => {
                // Don't accept input while task is running
                if !self.task_running {
                    self.input_buffer.push(c);
                }
            }
            KeyCode::Backspace => {
                if !self.task_running {
                    self.input_buffer.pop();
                }
            }
            KeyCode::Enter => {
                self.process_input()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn can_toggle_qr(&self) -> bool {
        matches!(
            self.operation_phase,
            OperationPhase::Complete | OperationPhase::Failed
        ) && (self.state == AppState::Stamp || self.state == AppState::Verify)
            && self.qr_data.is_some()
    }

    /// Process the current input based on state
    fn process_input(&mut self) -> Result<()> {
        // Don't process if a task is already running
        if self.task_running {
            return Ok(());
        }

        match self.state {
            AppState::Stamp => {
                if matches!(self.operation_phase, OperationPhase::Input) {
                    self.start_stamp_task();
                }
            }
            AppState::Verify => {
                self.process_verify();
            }
            AppState::Wallet => {
                if self.input_buffer.to_lowercase() == "s"
                    || self.input_buffer.to_lowercase() == "sync"
                {
                    self.start_sync_task();
                }
            }
            AppState::Menu => {}
        }
        self.input_buffer.clear();
        Ok(())
    }

    /// Start stamp operation as background task
    fn start_stamp_task(&mut self) {
        let input = self.input_buffer.trim().to_string();
        if input.is_empty() {
            self.result_message = "Please enter a file path or hash".to_string();
            self.result_is_error = true;
            return;
        }

        // Check if config is available
        let config = match &self.config {
            Some(c) => c.clone(),
            None => {
                self.result_message = "No wallet configured (set ZOTS_SEED)".to_string();
                self.result_is_error = true;
                self.operation_phase = OperationPhase::Failed;
                return;
            }
        };

        // Validate input and compute hash (fast, synchronous)
        let path = PathBuf::from(&input);
        let (hash_bytes, output_path) = if path.exists() {
            match hash_file_with(&path, self.hash_algorithm) {
                Ok(h) => {
                    let output = PathBuf::from(format!(
                        "{}.zots",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                    (h, output)
                }
                Err(e) => {
                    self.result_message = format!("Hash error: {e}");
                    self.result_is_error = true;
                    self.operation_phase = OperationPhase::Failed;
                    return;
                }
            }
        } else if input.len() >= 40 {
            match hash_from_hex_with(&input, self.hash_algorithm) {
                Ok(h) => {
                    let output = PathBuf::from(format!("{}.zots", &input[..16]));
                    (h, output)
                }
                Err(e) => {
                    self.result_message = format!("Invalid hash: {e}");
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

        // Mark as running and update UI
        self.task_running = true;
        self.operation_phase = OperationPhase::Syncing;
        self.status_message = "Starting stamp operation...".to_string();
        self.qr_visible = false;
        self.qr_data = None;

        // Clone sender for background task
        let tx = self.task_tx.clone();
        let network = config.network;
        let algorithm = self.hash_algorithm;

        // Spawn background task
        tokio::spawn(async move {
            run_stamp_task(tx, config, hash_bytes, output_path, network, algorithm).await;
        });
    }

    /// Start wallet sync as background task
    fn start_sync_task(&mut self) {
        let config = match &self.config {
            Some(c) => c.clone(),
            None => {
                self.result_message = "No wallet configured".to_string();
                self.result_is_error = true;
                self.operation_phase = OperationPhase::Failed;
                return;
            }
        };

        self.task_running = true;
        self.operation_phase = OperationPhase::Syncing;
        self.status_message = "Syncing wallet...".to_string();

        let tx = self.task_tx.clone();

        tokio::spawn(async move {
            run_sync_task(tx, config).await;
        });
    }

    /// Process verify input - multi-step: file/hash, then proof path
    fn process_verify(&mut self) {
        let input = self.input_buffer.trim().to_string();

        match self.verify_step {
            VerifyStep::FileOrHash => {
                if input.is_empty() {
                    self.result_message = "Please enter a file path or hash to verify".to_string();
                    self.result_is_error = true;
                    return;
                }

                self.verify_hash = None;
                self.verify_input_kind = None;

                // Determine if input is a file or hash (synchronous, fast)
                let path = PathBuf::from(&input);
                if path.exists() {
                    self.verify_input_kind = Some(VerifyInputKind::File);
                    self.verify_file_input = input;
                    self.verify_hash = None;
                    self.verify_step = VerifyStep::ProofPath;
                    self.result_message.clear();
                    self.result_is_error = false;
                } else if input.len() >= 40 {
                    match hash_from_hex_with(&input, HashAlgorithm::Sha256) {
                        Ok(_) => {
                            self.verify_input_kind = Some(VerifyInputKind::Hash);
                            self.verify_file_input = input;
                            self.verify_hash = None;
                            self.verify_step = VerifyStep::ProofPath;
                            self.result_message.clear();
                            self.result_is_error = false;
                        }
                        Err(e) => {
                            self.result_message = format!("Invalid hash: {e}");
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
                if self.task_running {
                    return;
                }

                self.qr_visible = false;
                self.qr_data = None;

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

                // Load and validate proof (fast, synchronous)
                let proof = match TimestampProof::load(&path) {
                    Ok(p) => p,
                    Err(e) => {
                        self.result_message = format!("Load error: {e}");
                        self.result_is_error = true;
                        return;
                    }
                };
                let proof_compact = proof.to_compact().unwrap_or_default();

                let proof_hash_bytes = match proof.hash_bytes() {
                    Ok(h) => h,
                    Err(e) => {
                        self.result_message = format!("Invalid proof hash: {e}");
                        self.result_is_error = true;
                        return;
                    }
                };

                let proof_algorithm = proof.hash_algorithm();
                let recomputed_hash = match self.verify_input_kind {
                    Some(VerifyInputKind::File) => {
                        let path = PathBuf::from(&self.verify_file_input);
                        hash_file_with(&path, proof_algorithm)
                    }
                    Some(VerifyInputKind::Hash) => {
                        hash_from_hex_with(&self.verify_file_input, proof_algorithm)
                    }
                    None => Ok(proof_hash_bytes),
                };

                let verify_hash = match recomputed_hash {
                    Ok(hash) => hash,
                    Err(e) => {
                        self.result_message = format!("Hash error: {e}");
                        self.result_is_error = true;
                        return;
                    }
                };
                self.verify_hash = Some(verify_hash);
                self.qr_data = Some(proof_compact.clone());

                let file_hash_matches = self.verify_hash.map(|h| h == proof_hash_bytes);
                if let Some(matches) = file_hash_matches
                    && !matches
                {
                    self.verify_result = Some(VerifyResult {
                        hash: proof.hash.clone(),
                        algorithm: proof_algorithm,
                        compact: proof_compact.clone(),
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

                if proof.attestations.is_empty() {
                    self.verify_result = Some(VerifyResult {
                        hash: proof.hash.clone(),
                        algorithm: proof_algorithm,
                        compact: proof_compact.clone(),
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

                // Start blockchain verification as background task
                let config = match &self.config {
                    Some(c) => c.clone(),
                    None => {
                        // No wallet, show proof info only
                        let att = &proof.attestations[0];
                        self.verify_result = Some(VerifyResult {
                            hash: proof.hash.clone(),
                            algorithm: proof_algorithm,
                            compact: proof_compact.clone(),
                            valid: true,
                            network: att.network.to_string(),
                            block_height: att.block_height,
                            timestamp: att.timestamp().to_rfc3339(),
                            txid: att.txid_hex().to_string(),
                            explorer_link: att.explorer_link(),
                            error: Some(
                                "Cannot verify on-chain (no wallet configured)".to_string(),
                            ),
                            file_hash_matches,
                        });
                        self.verify_step = VerifyStep::Results;
                        self.operation_phase = OperationPhase::Complete;
                        return;
                    }
                };

                let att = &proof.attestations[0];
                let txid_bytes = match att.txid_bytes() {
                    Ok(b) => b,
                    Err(e) => {
                        self.result_message = format!("Invalid TXID: {e}");
                        self.result_is_error = true;
                        self.operation_phase = OperationPhase::Failed;
                        return;
                    }
                };

                // Prepare data for background task
                let verify_data = VerifyTaskData {
                    proof_hash: proof.hash.clone(),
                    algorithm: proof_algorithm,
                    compact: proof_compact.clone(),
                    proof_hash_bytes,
                    txid_bytes,
                    block_height: att.block_height,
                    network: att.network.to_string(),
                    timestamp: att.timestamp().to_rfc3339(),
                    txid: att.txid_hex().to_string(),
                    explorer_link: att.explorer_link(),
                    file_hash_matches,
                };
                self.qr_data = Some(proof_compact);

                self.task_running = true;
                self.verify_step = VerifyStep::Verifying;
                self.operation_phase = OperationPhase::Syncing;
                self.status_message = "Verifying against blockchain...".to_string();

                let tx = self.task_tx.clone();

                tokio::spawn(async move {
                    run_verify_task(tx, config, verify_data).await;
                });
            }
            VerifyStep::Verifying | VerifyStep::Results => {
                // Do nothing
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

/// Data needed for verify background task
struct VerifyTaskData {
    proof_hash: String,
    algorithm: HashAlgorithm,
    compact: String,
    proof_hash_bytes: [u8; 32],
    txid_bytes: [u8; 32],
    block_height: u32,
    network: String,
    timestamp: String,
    txid: String,
    explorer_link: String,
    file_hash_matches: Option<bool>,
}

/// Background task for stamp operation
async fn run_stamp_task(
    tx: mpsc::Sender<TaskMessage>,
    config: ZcashConfig,
    hash_bytes: [u8; 32],
    output_path: PathBuf,
    network: zots_core::Network,
    hash_algorithm: HashAlgorithm,
) {
    let hash_hex = hash_to_hex(&hash_bytes);

    // Syncing phase
    let _ = tx.send(TaskMessage::Phase(OperationPhase::Syncing)).await;
    let _ = tx
        .send(TaskMessage::Status("Syncing wallet...".to_string()))
        .await;

    let mut wallet = match ZotsWallet::new(config).await {
        Ok(w) => w,
        Err(e) => {
            let _ = tx
                .send(TaskMessage::StampFailed(format!("Wallet error: {e}")))
                .await;
            return;
        }
    };

    if let Err(e) = wallet.init_account().await {
        let _ = tx
            .send(TaskMessage::StampFailed(format!("Account init error: {e}")))
            .await;
        return;
    }

    if let Err(e) = wallet.sync().await {
        let _ = tx
            .send(TaskMessage::StampFailed(format!("Sync failed: {e}")))
            .await;
        return;
    }

    // Broadcasting phase
    let _ = tx
        .send(TaskMessage::Phase(OperationPhase::Broadcasting))
        .await;
    let _ = tx
        .send(TaskMessage::Status(
            "Creating and broadcasting transaction...".to_string(),
        ))
        .await;

    let tx_result = match wallet.create_timestamp_tx(&hash_bytes).await {
        Ok(r) => r,
        Err(e) => {
            let _ = tx
                .send(TaskMessage::StampFailed(format!("Transaction failed: {e}")))
                .await;
            return;
        }
    };

    let txid = tx_result.txid.clone();

    // Waiting for confirmation phase
    let _ = tx
        .send(TaskMessage::Phase(OperationPhase::WaitingConfirmation {
            txid: txid.clone(),
        }))
        .await;
    let _ = tx
        .send(TaskMessage::Status(format!(
            "Waiting for confirmation (TXID: {}...)",
            &txid[..12]
        )))
        .await;

    let confirmation = match wallet.wait_confirmation(&txid, 10).await {
        Ok(c) => c,
        Err(e) => {
            // Save pending proof
            let proof = TimestampProof::new_with_algorithm(hash_bytes, hash_algorithm);
            let _ = proof.save(&output_path);

            let _ = tx
                .send(TaskMessage::StampFailed(format!(
                    "TX broadcast but confirmation timed out: {e}\nPending proof saved: {}",
                    output_path.display()
                )))
                .await;
            return;
        }
    };

    // Create and save proof
    let mut proof = TimestampProof::new_with_algorithm(hash_bytes, hash_algorithm);
    proof.add_attestation(ZcashAttestation::new(
        network,
        tx_result.txid_bytes,
        confirmation.block_height,
        confirmation.block_time,
        0,
    ));

    if let Err(e) = proof.save(&output_path) {
        let _ = tx
            .send(TaskMessage::StampFailed(format!("Save error: {e}")))
            .await;
        return;
    }

    let compact = proof
        .to_compact()
        .unwrap_or_else(|_| "Error generating compact format".to_string());

    let _ = tx
        .send(TaskMessage::StampComplete(StampResult {
            hash: hash_hex,
            algorithm: hash_algorithm,
            txid,
            block_height: confirmation.block_height,
            block_time: confirmation.block_time as u64,
            output_path: output_path.display().to_string(),
            compact,
        }))
        .await;
}

/// Background task for wallet sync (explicit user action)
async fn run_sync_task(tx: mpsc::Sender<TaskMessage>, config: ZcashConfig) {
    let _ = tx.send(TaskMessage::Phase(OperationPhase::Syncing)).await;
    let _ = tx
        .send(TaskMessage::Status("Syncing wallet...".to_string()))
        .await;

    let mut wallet = match ZotsWallet::new(config).await {
        Ok(w) => w,
        Err(e) => {
            let _ = tx
                .send(TaskMessage::SyncFailed(format!("Wallet error: {e}")))
                .await;
            return;
        }
    };

    if let Err(e) = wallet.init_account().await {
        let _ = tx
            .send(TaskMessage::SyncFailed(format!("Account init error: {e}")))
            .await;
        return;
    }

    if let Err(e) = wallet.sync().await {
        let _ = tx.send(TaskMessage::SyncFailed(e.to_string())).await;
        return;
    }

    let block_height = wallet.get_block_height().await.unwrap_or(0);
    let balance = wallet.get_balance().unwrap_or(0);

    let _ = tx
        .send(TaskMessage::SyncComplete {
            block_height,
            balance,
        })
        .await;
}

/// Background task for initial wallet sync (silent, at app startup)
async fn run_initial_sync_task(tx: mpsc::Sender<TaskMessage>, config: ZcashConfig) {
    let mut wallet = match ZotsWallet::new(config).await {
        Ok(w) => w,
        Err(_) => {
            let _ = tx.send(TaskMessage::InitialSyncFailed).await;
            return;
        }
    };

    if wallet.init_account().await.is_err() {
        let _ = tx.send(TaskMessage::InitialSyncFailed).await;
        return;
    }

    if wallet.sync().await.is_err() {
        let _ = tx.send(TaskMessage::InitialSyncFailed).await;
        return;
    }

    let block_height = wallet.get_block_height().await.unwrap_or(0);
    let balance = wallet.get_balance().unwrap_or(0);

    let _ = tx
        .send(TaskMessage::InitialSyncComplete {
            block_height,
            balance,
        })
        .await;
}

/// Background task for verify operation
async fn run_verify_task(tx: mpsc::Sender<TaskMessage>, config: ZcashConfig, data: VerifyTaskData) {
    let _ = tx.send(TaskMessage::Phase(OperationPhase::Syncing)).await;
    let _ = tx
        .send(TaskMessage::Status(
            "Verifying against blockchain...".to_string(),
        ))
        .await;

    let mut wallet = match ZotsWallet::new(config).await {
        Ok(w) => w,
        Err(e) => {
            let _ = tx
                .send(TaskMessage::VerifyFailed(format!("Wallet error: {e}")))
                .await;
            return;
        }
    };

    if let Err(e) = wallet.init_account().await {
        let _ = tx
            .send(TaskMessage::VerifyFailed(format!(
                "Account init error: {e}"
            )))
            .await;
        return;
    }

    let result = wallet
        .verify_timestamp_tx(
            &data.txid_bytes,
            &data.proof_hash_bytes,
            Some(data.block_height),
        )
        .await;

    match result {
        Ok(vr) => {
            let _ = tx
                .send(TaskMessage::VerifyComplete(VerifyResult {
                    hash: data.proof_hash,
                    algorithm: data.algorithm,
                    compact: data.compact.clone(),
                    valid: vr.valid,
                    network: data.network,
                    block_height: data.block_height,
                    timestamp: data.timestamp,
                    txid: data.txid,
                    explorer_link: data.explorer_link,
                    error: vr.error,
                    file_hash_matches: data.file_hash_matches,
                }))
                .await;
        }
        Err(e) => {
            let _ = tx
                .send(TaskMessage::VerifyComplete(VerifyResult {
                    hash: data.proof_hash,
                    algorithm: data.algorithm,
                    compact: data.compact,
                    valid: false,
                    network: data.network,
                    block_height: data.block_height,
                    timestamp: data.timestamp,
                    txid: data.txid,
                    explorer_link: data.explorer_link,
                    error: Some(format!("Verification error: {e}")),
                    file_hash_matches: data.file_hash_matches,
                }))
                .await;
        }
    }
}
