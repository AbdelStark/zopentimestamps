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

/// Check if a wallet exists
#[tauri::command]
pub async fn check_wallet_exists() -> Result<bool, String> {
    let data_dir = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join(".zopentimestamps");

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

/// Initialize wallet with seed phrase
#[tauri::command]
pub async fn init_wallet(state: State<'_, AppState>, seed: String) -> Result<WalletInfo, String> {
    let config = ZcashConfig::from_seed(&seed)
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

/// Load existing wallet
#[tauri::command]
pub async fn load_wallet(state: State<'_, AppState>, seed: String) -> Result<WalletInfo, String> {
    let config = ZcashConfig::from_seed(&seed)
        .map_err(|e| format!("Invalid seed phrase: {}", e))?;

    let mut wallet = ZotsWallet::new(config)
        .await
        .map_err(|e| format!("Failed to load wallet: {}", e))?;

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
