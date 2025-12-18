//! Configuration for Zcash wallet operations.

use std::path::PathBuf;

/// Network type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

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
    /// Get the path to the wallet database file
    pub fn wallet_db_path(&self) -> PathBuf {
        self.data_dir.join("wallet.db")
    }

    /// Get the path to the data directory, creating it if needed
    pub fn ensure_data_dir(&self) -> anyhow::Result<PathBuf> {
        std::fs::create_dir_all(&self.data_dir)?;
        Ok(self.data_dir.clone())
    }

    /// Create configuration from a seed phrase with optional birthday height
    ///
    /// Birthday height is the block height when the wallet was created.
    /// Using the correct birthday significantly speeds up initial sync.
    /// If not provided, defaults to a recent testnet block.
    pub fn from_seed_with_birthday(
        seed_phrase: &str,
        birthday_height: Option<u64>,
    ) -> anyhow::Result<Self> {
        // Validate seed phrase (basic check)
        let words: Vec<&str> = seed_phrase.split_whitespace().collect();
        if words.len() != 24 {
            anyhow::bail!("Seed phrase must be 24 words, got {}", words.len());
        }

        let data_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".ikki");

        Ok(Self {
            seed_phrase: seed_phrase.to_string(),
            birthday_height: birthday_height.unwrap_or(3717528),
            lightwalletd_url: "https://testnet.zec.rocks:443".to_string(),
            data_dir,
            network: Network::Testnet,
        })
    }
}
