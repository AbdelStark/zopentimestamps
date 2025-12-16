//! # zots-zcash
//!
//! Zcash blockchain integration for zOpenTimestamps.
//!
//! This crate provides wallet management, transaction creation, and
//! lightwalletd communication for timestamping on the Zcash blockchain.
//!
//! ## Overview
//!
//! - **Wallet**: HD wallet with unified addresses (Orchard + Sapling)
//! - **Transactions**: Self-send transactions with memo-encoded timestamps
//! - **Sync**: Compact block scanning via lightwalletd gRPC
//! - **Verification**: Fetch and decrypt transactions to verify timestamps
//!
//! ## Example
//!
//! ```rust,ignore
//! use zots_zcash::{ZcashConfig, ZotsWallet};
//!
//! async fn example() -> anyhow::Result<()> {
//!     // Load configuration from environment
//!     let config = ZcashConfig::from_env()?;
//!
//!     // Create and initialize wallet
//!     let mut wallet = ZotsWallet::new(config).await?;
//!     wallet.init_account().await?;
//!
//!     // Sync with blockchain
//!     wallet.sync().await?;
//!
//!     // Check balance
//!     let balance = wallet.get_balance()?;
//!     let total = balance.orchard + balance.sapling + balance.transparent;
//!     println!("Balance: {} ZEC", total as f64 / 100_000_000.0);
//!
//!     // Create timestamp transaction
//!     let hash = [0u8; 32]; // 32-byte hash to timestamp
//!     let result = wallet.create_timestamp_tx(&hash).await?;
//!     println!("TXID: {}", result.txid);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Security Warning
//!
//! This is experimental software. Do not use on mainnet with real funds.
//! The code has not been audited. Always use testnet for development.
//!
//! ## Network Support
//!
//! Currently supports Zcash testnet via lightwalletd servers.
//! Mainnet support is intentionally disabled for safety.

pub mod config;
pub mod memo;
pub mod wallet;

pub use config::*;
pub use memo::*;
pub use wallet::*;
