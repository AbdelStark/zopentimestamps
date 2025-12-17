//! Application messages for iced

use std::path::PathBuf;
use zots_core::HashAlgorithm;

/// Main application message type
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavigateTo(View),

    // Wallet operations
    SeedInputChanged(String),
    SaveSeed,
    SyncWallet,
    WalletSynced {
        block_height: u64,
        balance: u64,
    },
    WalletSyncFailed(String),
    InitialSyncComplete {
        block_height: u64,
        balance: u64,
    },
    InitialSyncFailed,

    // Stamp operations
    StampInputChanged(String),
    SelectFile,
    FileSelected(Option<PathBuf>),
    ToggleAlgorithm,
    StartStamp,
    StampProgress(StampPhase),
    StampComplete(StampResult),
    StampFailed(String),

    // Verify operations
    VerifyFileInputChanged(String),
    VerifyProofInputChanged(String),
    SelectVerifyFile,
    SelectProofFile,
    VerifyFileSelected(Option<PathBuf>),
    ProofFileSelected(Option<PathBuf>),
    StartVerify,
    VerifyComplete(VerifyResult),
    VerifyFailed(String),

    // History
    LoadHistory,
    HistoryLoaded(Vec<HistoryEntry>),
    DeleteProof(PathBuf),
    ProofDeleted(PathBuf),

    // Settings
    ExplorerUrlChanged(String),
    LightwalletdUrlChanged(String),
    SaveSettings,
    ResetSettings,

    // UI
    Tick,
    CopyToClipboard(String),
    Copied,
    OpenExplorer(String),
    DismissResult,
}

/// Application views/screens
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Home,
    Stamp,
    Verify,
    History,
    Wallet,
    Settings,
}

impl View {
    pub fn title(&self) -> &'static str {
        match self {
            View::Home => "Home",
            View::Stamp => "Stamp",
            View::Verify => "Verify",
            View::History => "History",
            View::Wallet => "Wallet",
            View::Settings => "Settings",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            View::Home => "🏠",
            View::Stamp => "📝",
            View::Verify => "🔍",
            View::History => "📋",
            View::Wallet => "💰",
            View::Settings => "⚙️",
        }
    }
}

/// Stamp operation phases
#[derive(Debug, Clone, PartialEq)]
pub enum StampPhase {
    Idle,
    Syncing,
    Broadcasting,
    WaitingConfirmation { txid: String },
    Complete,
    Failed,
}

impl Default for StampPhase {
    fn default() -> Self {
        Self::Idle
    }
}

impl StampPhase {
    pub fn is_busy(&self) -> bool {
        matches!(
            self,
            StampPhase::Syncing | StampPhase::Broadcasting | StampPhase::WaitingConfirmation { .. }
        )
    }

    pub fn message(&self) -> &str {
        match self {
            StampPhase::Idle => "Ready",
            StampPhase::Syncing => "Syncing wallet...",
            StampPhase::Broadcasting => "Broadcasting transaction...",
            StampPhase::WaitingConfirmation { .. } => "Waiting for confirmation...",
            StampPhase::Complete => "Timestamp created!",
            StampPhase::Failed => "Operation failed",
        }
    }
}

/// Result of a successful stamp operation
#[derive(Debug, Clone)]
pub struct StampResult {
    pub hash: String,
    pub algorithm: HashAlgorithm,
    pub txid: String,
    pub block_height: u32,
    pub block_time: u64,
    pub output_path: PathBuf,
    pub compact: String,
    pub explorer_link: String,
    pub pending: bool, // True if not yet confirmed
}

/// Result of a verify operation
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

/// History entry for past proofs
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub path: PathBuf,
    pub hash: String,
    pub algorithm: HashAlgorithm,
    pub created: String,
    pub confirmed: bool,
    pub network: Option<String>,
    pub block_height: Option<u32>,
}
