//! Transaction-related Tauri commands

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Transaction type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Sent,
    Received,
    Shielding,
    Internal,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

/// Transaction data for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub txid: String,
    pub tx_type: TransactionType,
    pub amount: i64,
    pub timestamp: u64,
    pub address: Option<String>,
    pub memo: Option<String>,
    pub status: TransactionStatus,
    pub confirmations: u32,
}

/// Send result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub txid: String,
    pub amount: u64,
    pub fee: u64,
}

/// Send transaction
#[tauri::command]
pub async fn send_transaction(
    state: State<'_, AppState>,
    to_address: String,
    amount: u64,
    memo: Option<String>,
) -> Result<SendResult, String> {
    let mut wallet_lock = state.wallet.lock().await;
    let wallet = wallet_lock
        .as_mut()
        .ok_or("Wallet not initialized")?;

    let memo_bytes = memo.map(|m| m.into_bytes());

    let result = wallet
        .send_to_address(&to_address, amount, memo_bytes)
        .await
        .map_err(|e| format!("Send failed: {}", e))?;

    Ok(SendResult {
        txid: result.txid,
        amount,
        fee: result.fee,
    })
}

/// Get transaction history
#[tauri::command]
pub async fn get_transactions(_state: State<'_, AppState>) -> Result<Vec<Transaction>, String> {
    // TODO: Implement actual transaction history retrieval from wallet
    // For now, return empty list - the zots-zcash crate needs to expose transaction history
    Ok(vec![])
}
