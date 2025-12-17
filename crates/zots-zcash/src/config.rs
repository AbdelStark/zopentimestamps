//! Configuration for Zcash wallet operations.
//!
//! Configuration is loaded from environment variables or a `.env` file.
//!
//! ## Environment Variables
//!
//! | Variable | Required | Default | Description |
//! |----------|----------|---------|-------------|
//! | `ZOTS_SEED` | Yes | - | 24-word BIP-39 seed phrase |
//! | `ZOTS_BIRTHDAY_HEIGHT` | No | 3717528 | Wallet birthday for faster sync |
//! | `ZOTS_LIGHTWALLETD` | No | testnet.zec.rocks:443 | Lightwalletd server URL |
//! | `ZOTS_NETWORK` | No | testnet | Network: "testnet" or "mainnet" |
//! | `ZOTS_DATA_DIR` | No | ~/.zopentimestamps | Data directory path |
//!
//! ## Security Warning
//!
//! - Never commit your seed phrase to version control
//! - Use environment variables or a `.env` file (add to .gitignore)
//! - Only use testnet - mainnet is not recommended

use std::path::PathBuf;
use zots_core::Network;

/// Configuration for Zcash wallet and network operations
#[derive(Debug, Clone)]
pub struct ZcashConfig {
    /// BIP-39 seed phrase (24 words)
    pub seed_phrase: String,
    /// Wallet birthday height for faster sync
    pub birthday_height: u64,
    /// Lightwalletd server URL
    pub lightwalletd_url: String,
    /// Directory for wallet data storage
    pub data_dir: PathBuf,
    /// Network (mainnet or testnet)
    pub network: Network,
}

impl ZcashConfig {
    /// Load configuration from environment variables
    ///
    /// Required:
    /// - `ZOTS_SEED`: 24-word BIP-39 seed phrase
    ///
    /// Optional (with defaults):
    /// - `ZOTS_BIRTHDAY_HEIGHT`: Wallet birthday (default: 3717528)
    /// - `ZOTS_LIGHTWALLETD`: Server URL (default: https://testnet.zec.rocks:443)
    /// - `ZOTS_NETWORK`: Network type (default: testnet)
    /// - `ZOTS_DATA_DIR`: Data directory (default: ~/.zopentimestamps)
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let seed_phrase = std::env::var("ZOTS_SEED").map_err(|_| {
            anyhow::anyhow!("ZOTS_SEED environment variable not set. Set your 24-word seed phrase.")
        })?;

        let birthday_height = std::env::var("ZOTS_BIRTHDAY_HEIGHT")
            .unwrap_or_else(|_| "3717528".to_string())
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid ZOTS_BIRTHDAY_HEIGHT: {e}"))?;

        let lightwalletd_url = std::env::var("ZOTS_LIGHTWALLETD")
            .unwrap_or_else(|_| "https://testnet.zec.rocks:443".to_string());

        let data_dir = std::env::var("ZOTS_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".zopentimestamps")
            });

        let network = match std::env::var("ZOTS_NETWORK")
            .unwrap_or_else(|_| "testnet".to_string())
            .to_lowercase()
            .as_str()
        {
            "mainnet" | "main" => Network::Mainnet,
            _ => Network::Testnet,
        };

        Ok(Self {
            seed_phrase,
            birthday_height,
            lightwalletd_url,
            data_dir,
            network,
        })
    }

    /// Get the path to the wallet database file
    pub fn wallet_db_path(&self) -> PathBuf {
        self.data_dir.join("wallet.db")
    }

    /// Get the path to the data directory, creating it if needed
    pub fn ensure_data_dir(&self) -> anyhow::Result<PathBuf> {
        std::fs::create_dir_all(&self.data_dir)?;
        Ok(self.data_dir.clone())
    }

    /// Create configuration from a seed phrase with defaults
    pub fn from_seed(seed_phrase: &str) -> anyhow::Result<Self> {
        // Validate seed phrase (basic check)
        let words: Vec<&str> = seed_phrase.split_whitespace().collect();
        if words.len() != 24 {
            anyhow::bail!("Seed phrase must be 24 words, got {}", words.len());
        }

        let data_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".zopentimestamps");

        Ok(Self {
            seed_phrase: seed_phrase.to_string(),
            birthday_height: 3717528,
            lightwalletd_url: "https://testnet.zec.rocks:443".to_string(),
            data_dir,
            network: Network::Testnet,
        })
    }
}
