//! Wallet-related Tauri commands

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use zots_zcash::{ZcashConfig, ZotsWallet};

/// Wallet information returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    pub balance: u64,
    pub block_height: u64,
}

/// Balance breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub total: u64,
    pub shielded: u64,
    pub transparent: u64,
}

/// Sync result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub block_height: u64,
    pub balance: u64,
}

/// Get wallet data directory path
fn get_data_dir() -> Result<std::path::PathBuf, String> {
    let data_dir = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join(".zopentimestamps");
    Ok(data_dir)
}

/// Check if a wallet exists
#[tauri::command]
pub async fn check_wallet_exists() -> Result<bool, String> {
    let data_dir = get_data_dir()?;
    let wallet_db = data_dir.join("wallet.db");
    Ok(wallet_db.exists())
}

/// Generate a new seed phrase
#[tauri::command]
pub async fn generate_seed() -> Result<String, String> {
    use bip0039::{Count, English, Mnemonic};

    let mnemonic = Mnemonic::<English>::generate(Count::Words24);
    Ok(mnemonic.phrase().to_string())
}

/// Delete all wallet data (reset wallet)
#[tauri::command]
pub async fn reset_wallet(state: State<'_, AppState>) -> Result<(), String> {
    // Clear wallet from state first
    {
        let mut wallet_lock = state.wallet.lock().await;
        *wallet_lock = None;
    }

    let data_dir = get_data_dir()?;

    // Remove wallet database
    let wallet_db = data_dir.join("wallet.db");
    if wallet_db.exists() {
        std::fs::remove_file(&wallet_db)
            .map_err(|e| format!("Failed to delete wallet.db: {}", e))?;
    }

    // Remove any other wallet-related files
    let files_to_remove = ["wallet.db-shm", "wallet.db-wal", "wallet_cache.db"];
    for file in files_to_remove {
        let path = data_dir.join(file);
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }

    Ok(())
}

/// Initialize wallet with seed phrase (new wallet)
#[tauri::command]
pub async fn init_wallet(
    state: State<'_, AppState>,
    seed: String,
    birthday_height: Option<u64>,
) -> Result<WalletInfo, String> {
    let config = ZcashConfig::from_seed_with_birthday(&seed, birthday_height)
        .map_err(|e| format!("Invalid seed phrase: {}", e))?;

    let mut wallet = ZotsWallet::new(config)
        .await
        .map_err(|e| format!("Failed to create wallet: {}", e))?;

    wallet
        .init_account()
        .await
        .map_err(|e| format!("Failed to initialize account: {}", e))?;

    let address = wallet
        .get_address()
        .map_err(|e| format!("Failed to get address: {}", e))?;

    let balance = wallet.get_balance().unwrap_or(0);

    let block_height = wallet
        .get_block_height()
        .await
        .map_err(|e| format!("Failed to get block height: {}", e))?;

    // Store wallet in state
    let mut wallet_lock = state.wallet.lock().await;
    *wallet_lock = Some(wallet);

    Ok(WalletInfo {
        address,
        balance,
        block_height,
    })
}

/// Load existing wallet (import)
#[tauri::command]
pub async fn load_wallet(
    state: State<'_, AppState>,
    seed: String,
    birthday_height: Option<u64>,
) -> Result<WalletInfo, String> {
    let config = ZcashConfig::from_seed_with_birthday(&seed, birthday_height)
        .map_err(|e| format!("Invalid seed phrase: {}", e))?;

    let mut wallet = ZotsWallet::new(config)
        .await
        .map_err(|e| format!("Failed to load wallet: {}", e))?;

    // Initialize account to ensure we have an address
    wallet
        .init_account()
        .await
        .map_err(|e| format!("Failed to initialize account: {}", e))?;

    let address = wallet
        .get_address()
        .map_err(|e| format!("Failed to get address: {}", e))?;

    let balance = wallet.get_balance().unwrap_or(0);

    let block_height = wallet
        .get_block_height()
        .await
        .map_err(|e| format!("Failed to get block height: {}", e))?;

    // Store wallet in state
    let mut wallet_lock = state.wallet.lock().await;
    *wallet_lock = Some(wallet);

    Ok(WalletInfo {
        address,
        balance,
        block_height,
    })
}

/// Get current balance
#[tauri::command]
pub async fn get_balance(state: State<'_, AppState>) -> Result<BalanceInfo, String> {
    let wallet_lock = state.wallet.lock().await;
    let wallet = wallet_lock
        .as_ref()
        .ok_or("Wallet not initialized")?;

    let breakdown = wallet
        .get_balance_breakdown()
        .map_err(|e| format!("Failed to get balance: {}", e))?;

    Ok(BalanceInfo {
        total: breakdown.sapling + breakdown.orchard + breakdown.transparent,
        shielded: breakdown.sapling + breakdown.orchard,
        transparent: breakdown.transparent,
    })
}

/// Get wallet address
#[tauri::command]
pub async fn get_address(state: State<'_, AppState>) -> Result<String, String> {
    let wallet_lock = state.wallet.lock().await;
    let wallet = wallet_lock
        .as_ref()
        .ok_or("Wallet not initialized")?;

    wallet
        .get_address()
        .map_err(|e| format!("Failed to get address: {}", e))
}

/// Sync wallet with blockchain
#[tauri::command]
pub async fn sync_wallet(state: State<'_, AppState>) -> Result<SyncResult, String> {
    let mut wallet_lock = state.wallet.lock().await;
    let wallet = wallet_lock
        .as_mut()
        .ok_or("Wallet not initialized")?;

    wallet
        .sync()
        .await
        .map_err(|e| format!("Sync failed: {}", e))?;

    let balance = wallet.get_balance().unwrap_or(0);
    let block_height = wallet
        .get_block_height()
        .await
        .map_err(|e| format!("Failed to get block height: {}", e))?;

    Ok(SyncResult {
        block_height,
        balance,
    })
}
