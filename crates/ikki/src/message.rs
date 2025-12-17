//! Application messages and types

/// Main application message type
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavigateTo(View),

    // Wallet lifecycle
    WalletLoaded {
        balance: u64,
        address: String,
        block_height: u64,
    },
    WalletLoadFailed(String),

    // Sync
    SyncWallet,
    SyncProgress {
        current: u64,
        target: u64,
    },
    SyncComplete {
        block_height: u64,
        balance: u64,
    },
    SyncFailed(String),

    // Send
    SendAmountChanged(String),
    SendAddressChanged(String),
    SendMemoChanged(String),
    SendMaxAmount,
    SendPreview,
    SendConfirm,
    SendComplete {
        txid: String,
        amount: u64,
        fee: u64,
    },
    SendFailed(String),
    SendCancel,
    SendReset,

    // Receive
    GenerateNewAddress,
    AddressGenerated(String),
    CopyAddress,

    // Transaction history
    LoadTransactions,
    TransactionsLoaded(Vec<Transaction>),
    TransactionSelected(String),

    // Settings
    SeedInputChanged(String),
    SaveSeed,
    SeedSaved,
    SeedSaveFailed(String),
    ShowSeed,
    HideSeed,
    LightwalletdChanged(String),
    NetworkChanged(NetworkChoice),
    SaveSettings,

    // Onboarding
    StartOnboarding,
    OnboardingImportSeed,
    OnboardingCreateNew,
    GenerateNewSeed,
    SeedGenerated(String),
    ConfirmSeedBackup,
    OnboardingComplete,

    // UI
    Tick,
    CopyToClipboard(String),
    Copied,
    ShowToast(String, ToastType),
    DismissToast,
    OpenExplorer(String),
    ToggleMenu,
    CloseMenu,
}

/// Application views/screens
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Home,
    Send,
    Receive,
    History,
    Settings,
    Onboarding,
}

impl View {
    pub fn title(&self) -> &'static str {
        match self {
            View::Home => "Home",
            View::Send => "Send",
            View::Receive => "Receive",
            View::History => "Activity",
            View::Settings => "Settings",
            View::Onboarding => "Welcome",
        }
    }
}

/// Network choice for settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetworkChoice {
    #[default]
    Testnet,
    Mainnet,
}

impl NetworkChoice {
    pub fn name(&self) -> &'static str {
        match self {
            NetworkChoice::Testnet => "Testnet",
            NetworkChoice::Mainnet => "Mainnet",
        }
    }
}

/// Send flow state
#[derive(Debug, Clone, Default)]
pub struct SendState {
    pub amount: String,
    pub address: String,
    pub memo: String,
    pub phase: SendPhase,
    pub error: Option<String>,
    pub fee_estimate: u64,
}

/// Send operation phases
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SendPhase {
    #[default]
    Input,
    Preview,
    Sending,
    Complete {
        txid: String,
    },
    Failed,
}

impl SendPhase {
    pub fn is_busy(&self) -> bool {
        matches!(self, SendPhase::Sending)
    }
}

/// Transaction representation
#[derive(Debug, Clone)]
pub struct Transaction {
    pub txid: String,
    pub amount: i64, // Negative for sent, positive for received
    pub fee: Option<u64>,
    pub timestamp: u64,
    pub block_height: Option<u32>,
    pub memo: Option<String>,
    pub address: Option<String>,
    pub tx_type: TransactionType,
    pub status: TransactionStatus,
}

/// Transaction type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Sent,
    Received,
    Shielding,
    Internal,
}

impl TransactionType {
    pub fn label(&self) -> &'static str {
        match self {
            TransactionType::Sent => "Sent",
            TransactionType::Received => "Received",
            TransactionType::Shielding => "Shielded",
            TransactionType::Internal => "Internal",
        }
    }
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

impl TransactionStatus {
    pub fn label(&self) -> &'static str {
        match self {
            TransactionStatus::Pending => "Pending",
            TransactionStatus::Confirmed => "Confirmed",
            TransactionStatus::Failed => "Failed",
        }
    }
}

/// Toast notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

/// Onboarding state
#[derive(Debug, Clone, Default)]
pub struct OnboardingState {
    pub step: OnboardingStep,
    pub seed_input: String,
    pub generated_seed: Option<String>,
    pub seed_confirmed: bool,
    pub error: Option<String>,
}

/// Onboarding steps
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnboardingStep {
    #[default]
    Welcome,
    ImportOrCreate,
    ImportSeed,
    CreateSeed,
    ConfirmSeed,
    Complete,
}

/// Sync state
#[derive(Debug, Clone, Default)]
pub struct SyncState {
    pub syncing: bool,
    pub current_block: u64,
    pub target_block: u64,
    pub error: Option<String>,
}

impl SyncState {
    pub fn progress(&self) -> f32 {
        if self.target_block == 0 {
            0.0
        } else if self.current_block >= self.target_block {
            1.0
        } else {
            self.current_block as f32 / self.target_block as f32
        }
    }
}
