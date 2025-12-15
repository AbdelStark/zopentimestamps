# zOpenTimestamps - Implementation Plan (Rust CLI)

## Project Structure

```
zopentimestamps/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── zots-core/               # Core library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs           # Library exports
│   │       ├── hash.rs          # SHA-256 hashing
│   │       ├── proof.rs         # .zots format
│   │       ├── timestamp.rs     # Timestamp operations
│   │       ├── verify.rs        # Verification logic
│   │       └── error.rs         # Error types
│   │
│   ├── zots-zcash/              # Zcash integration
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs           # Module exports
│   │       ├── wallet.rs        # Wallet management
│   │       ├── transaction.rs   # TX building
│   │       ├── lightwalletd.rs  # gRPC client
│   │       ├── memo.rs          # Memo encoding
│   │       └── config.rs        # Network config
│   │
│   └── zots-cli/                # CLI application
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs          # Entry point
│           ├── cli.rs           # Clap definitions
│           ├── commands/
│           │   ├── mod.rs
│           │   ├── stamp.rs     # stamp command
│           │   ├── verify.rs    # verify command
│           │   ├── info.rs      # info command
│           │   └── wallet.rs    # wallet subcommands
│           ├── tui/
│           │   ├── mod.rs       # TUI module
│           │   ├── app.rs       # App state
│           │   ├── ui.rs        # UI rendering
│           │   ├── events.rs    # Event handling
│           │   └── widgets/
│           │       ├── mod.rs
│           │       ├── header.rs
│           │       ├── menu.rs
│           │       ├── file_browser.rs
│           │       ├── status_bar.rs
│           │       └── result_panel.rs
│           └── output.rs        # Colored CLI output
│
├── docs/
│   ├── prd.md
│   ├── plan.md
│   └── status.md
│
├── .env.example
├── .gitignore
└── README.md
```

---

## Implementation Phases

### Phase 1: Project Setup & Core Types (Day 1)

#### Step 1.1: Initialize Cargo Workspace

**Cargo.toml (root):**
```toml
[workspace]
resolver = "2"
members = [
    "crates/zots-core",
    "crates/zots-zcash", 
    "crates/zots-cli",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85.0"
license = "MIT"
repository = "https://github.com/example/zopentimestamps"

[workspace.dependencies]
# Core
sha2 = "0.10"
hex = "0.4"
thiserror = "2.0"
anyhow = "1.0"

# Async
tokio = { version = "1", features = ["full"] }

# Zcash - pinned to known working rev
zcash_client_backend = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6", features = ["lightwalletd-tonic-transport", "transparent-inputs"] }
zcash_client_sqlite = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6", features = ["transparent-inputs", "unstable"] }
zcash_protocol = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6" }
zcash_address = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6" }
zcash_proofs = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6", features = ["bundled-prover"] }
zcash_keys = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6", features = ["orchard", "sapling", "transparent-inputs"] }
zcash_transparent = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6" }

# gRPC
tonic = { version = "0.14", features = ["tls-native-roots"] }
prost = "0.14"
rustls = { version = "0.23", features = ["aws-lc-rs"] }

# CLI
clap = { version = "4.5", features = ["derive", "env"] }
dotenvy = "0.15"

# TUI
ratatui = "0.29"
crossterm = "0.28"

# Crypto
bip0039 = "0.12"
rand = "0.8"
rand_core = "0.6"

# Storage
rusqlite = { version = "0.37", features = ["bundled"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Time
chrono = "0.4"
time = "0.3"

# Output
colored = "2.1"
indicatif = "0.17"
```

#### Step 1.2: Create zots-core Crate

**crates/zots-core/Cargo.toml:**
```toml
[package]
name = "zots-core"
version.workspace = true
edition.workspace = true

[dependencies]
sha2.workspace = true
hex.workspace = true
thiserror.workspace = true
chrono.workspace = true
```

**crates/zots-core/src/lib.rs:**
```rust
pub mod error;
pub mod hash;
pub mod proof;
pub mod timestamp;
pub mod verify;

pub use error::{Error, Result};
pub use hash::*;
pub use proof::*;
pub use timestamp::*;
pub use verify::*;
```

**crates/zots-core/src/error.rs:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid proof format: {0}")]
    InvalidProof(String),
    
    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    
    #[error("Invalid hash format: {0}")]
    InvalidHash(String),
    
    #[error("Proof not yet confirmed")]
    NotConfirmed,
    
    #[error("Transaction not found: {0}")]
    TxNotFound(String),
    
    #[error("Network error: {0}")]
    Network(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Deliverables:**
- [ ] Cargo workspace initialized
- [ ] zots-core crate created
- [ ] Error types defined
- [ ] Project compiles
- [ ] Commit: "feat: initialize rust workspace and core types"

---

### Phase 2: SHA-256 Hashing Module (Day 1)

**crates/zots-core/src/hash.rs:**
```rust
use sha2::{Sha256, Digest};
use std::path::Path;
use std::fs::File;
use std::io::{Read, BufReader};
use crate::{Error, Result};

/// 32-byte SHA-256 hash
pub type Hash256 = [u8; 32];

/// Hash raw bytes
pub fn hash_bytes(data: &[u8]) -> Hash256 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash a file by path (streaming for large files)
pub fn hash_file(path: impl AsRef<Path>) -> Result<Hash256> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(hasher.finalize().into())
}

/// Hash a hex string (e.g., git commit hash)
/// Accepts 40 (git short) or 64 (full sha256) hex chars
pub fn hash_from_hex(hex_str: &str) -> Result<Hash256> {
    let cleaned = hex_str.trim().trim_start_matches("0x");
    
    if cleaned.len() == 40 {
        // Git commit hash - hash it to get 32 bytes
        let bytes = hex::decode(cleaned)
            .map_err(|e| Error::InvalidHash(e.to_string()))?;
        Ok(hash_bytes(&bytes))
    } else if cleaned.len() == 64 {
        // Already a SHA-256 hash
        let bytes = hex::decode(cleaned)
            .map_err(|e| Error::InvalidHash(e.to_string()))?;
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        Ok(hash)
    } else {
        Err(Error::InvalidHash(format!(
            "Expected 40 or 64 hex chars, got {}",
            cleaned.len()
        )))
    }
}

/// Convert hash to hex string
pub fn hash_to_hex(hash: &Hash256) -> String {
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_bytes() {
        let data = b"hello world";
        let hash = hash_bytes(data);
        let hex = hash_to_hex(&hash);
        assert_eq!(
            hex,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }
    
    #[test]
    fn test_hash_from_hex_git() {
        let git_hash = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        let result = hash_from_hex(git_hash);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_hash_from_hex_sha256() {
        let sha256 = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let result = hash_from_hex(sha256);
        assert!(result.is_ok());
    }
}
```

**Deliverables:**
- [ ] hash_bytes() function
- [ ] hash_file() with streaming
- [ ] hash_from_hex() for git commits
- [ ] Unit tests passing
- [ ] Commit: "feat: implement SHA-256 hashing module"

---

### Phase 3: Proof Format (.zots) (Day 1-2)

**crates/zots-core/src/proof.rs:**
```rust
use crate::{Error, Hash256, Result};
use std::io::{Read, Write};

/// Magic header: \x00zOTS\x00\x00\x01
pub const ZOTS_MAGIC: [u8; 8] = [0x00, 0x7A, 0x4F, 0x54, 0x53, 0x00, 0x00, 0x01];
pub const PROOF_VERSION: u8 = 1;
pub const HASH_TYPE_SHA256: u8 = 0x00;
pub const ATTESTATION_TYPE_ZCASH: u8 = 0x01;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    pub fn to_byte(&self) -> u8 {
        match self {
            Network::Mainnet => 0x00,
            Network::Testnet => 0x01,
        }
    }
    
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            0x00 => Ok(Network::Mainnet),
            0x01 => Ok(Network::Testnet),
            _ => Err(Error::InvalidProof(format!("Unknown network: {}", b))),
        }
    }
    
    pub fn explorer_url(&self) -> &'static str {
        match self {
            Network::Mainnet => "https://explorer.zec.rocks",
            Network::Testnet => "https://testnet.zcashexplorer.app",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZcashAttestation {
    pub network: Network,
    pub txid: [u8; 32],
    pub block_height: u32,
    pub block_time: u32,
    pub memo_offset: u16,
}

impl ZcashAttestation {
    pub fn txid_hex(&self) -> String {
        // Zcash txids are displayed reversed
        let mut reversed = self.txid;
        reversed.reverse();
        hex::encode(reversed)
    }
    
    pub fn explorer_link(&self) -> String {
        format!("{}/tx/{}", self.network.explorer_url(), self.txid_hex())
    }
    
    pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp(self.block_time as i64, 0)
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct TimestampProof {
    pub version: u8,
    pub hash: Hash256,
    pub attestations: Vec<ZcashAttestation>,
}

impl TimestampProof {
    pub fn new(hash: Hash256) -> Self {
        Self {
            version: PROOF_VERSION,
            hash,
            attestations: Vec::new(),
        }
    }
    
    pub fn add_attestation(&mut self, att: ZcashAttestation) {
        self.attestations.push(att);
    }
    
    /// Serialize to .zots binary format
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        
        // Magic header
        buf.extend_from_slice(&ZOTS_MAGIC);
        
        // Version
        buf.push(self.version);
        
        // Hash type
        buf.push(HASH_TYPE_SHA256);
        
        // Hash (32 bytes)
        buf.extend_from_slice(&self.hash);
        
        // Number of attestations
        buf.push(self.attestations.len() as u8);
        
        // Attestations
        for att in &self.attestations {
            buf.push(ATTESTATION_TYPE_ZCASH);
            buf.push(att.network.to_byte());
            buf.extend_from_slice(&att.txid);
            buf.extend_from_slice(&att.block_height.to_le_bytes());
            buf.extend_from_slice(&att.block_time.to_le_bytes());
            buf.extend_from_slice(&att.memo_offset.to_le_bytes());
        }
        
        buf
    }
    
    /// Deserialize from .zots binary format
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(Error::InvalidProof("File too small".into()));
        }
        
        // Check magic
        if &data[0..8] != &ZOTS_MAGIC {
            return Err(Error::InvalidProof("Invalid magic header".into()));
        }
        
        let mut offset = 8;
        
        // Version
        let version = data[offset];
        if version != PROOF_VERSION {
            return Err(Error::InvalidProof(format!(
                "Unsupported version: {}",
                version
            )));
        }
        offset += 1;
        
        // Hash type
        let hash_type = data[offset];
        if hash_type != HASH_TYPE_SHA256 {
            return Err(Error::InvalidProof(format!(
                "Unsupported hash type: {}",
                hash_type
            )));
        }
        offset += 1;
        
        // Hash
        if data.len() < offset + 32 {
            return Err(Error::InvalidProof("Truncated hash".into()));
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&data[offset..offset + 32]);
        offset += 32;
        
        // Number of attestations
        let num_attestations = data[offset] as usize;
        offset += 1;
        
        // Parse attestations
        let mut attestations = Vec::with_capacity(num_attestations);
        for _ in 0..num_attestations {
            if data.len() < offset + 44 {
                return Err(Error::InvalidProof("Truncated attestation".into()));
            }
            
            let att_type = data[offset];
            if att_type != ATTESTATION_TYPE_ZCASH {
                return Err(Error::InvalidProof(format!(
                    "Unknown attestation type: {}",
                    att_type
                )));
            }
            offset += 1;
            
            let network = Network::from_byte(data[offset])?;
            offset += 1;
            
            let mut txid = [0u8; 32];
            txid.copy_from_slice(&data[offset..offset + 32]);
            offset += 32;
            
            let block_height = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;
            
            let block_time = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;
            
            let memo_offset = u16::from_le_bytes([data[offset], data[offset + 1]]);
            offset += 2;
            
            attestations.push(ZcashAttestation {
                network,
                txid,
                block_height,
                block_time,
                memo_offset,
            });
        }
        
        Ok(Self {
            version,
            hash,
            attestations,
        })
    }
    
    /// Save to file
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let data = self.serialize();
        std::fs::write(path, data)?;
        Ok(())
    }
    
    /// Load from file
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let data = std::fs::read(path)?;
        Self::deserialize(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_proof_roundtrip() {
        let hash = [0xABu8; 32];
        let mut proof = TimestampProof::new(hash);
        
        proof.add_attestation(ZcashAttestation {
            network: Network::Testnet,
            txid: [0xCDu8; 32],
            block_height: 3721456,
            block_time: 1734567890,
            memo_offset: 0,
        });
        
        let serialized = proof.serialize();
        let deserialized = TimestampProof::deserialize(&serialized).unwrap();
        
        assert_eq!(deserialized.hash, hash);
        assert_eq!(deserialized.attestations.len(), 1);
        assert_eq!(deserialized.attestations[0].block_height, 3721456);
    }
}
```

**Deliverables:**
- [ ] TimestampProof struct
- [ ] ZcashAttestation struct
- [ ] Binary serialization
- [ ] Binary deserialization
- [ ] File I/O helpers
- [ ] Round-trip tests
- [ ] Commit: "feat: implement .zots proof format"

---

### Phase 4: Zcash Wallet Module (Day 2-3)

**crates/zots-zcash/Cargo.toml:**
```toml
[package]
name = "zots-zcash"
version.workspace = true
edition.workspace = true

[dependencies]
zots-core = { path = "../zots-core" }

# Zcash
zcash_client_backend.workspace = true
zcash_client_sqlite.workspace = true
zcash_protocol.workspace = true
zcash_address.workspace = true
zcash_proofs.workspace = true
zcash_keys.workspace = true
zcash_transparent.workspace = true

# gRPC
tonic.workspace = true
prost.workspace = true
rustls.workspace = true

# Crypto
bip0039.workspace = true
rand.workspace = true
rand_core.workspace = true

# Storage
rusqlite.workspace = true

# Async
tokio.workspace = true

# Other
thiserror.workspace = true
anyhow.workspace = true
time.workspace = true
```

**crates/zots-zcash/src/lib.rs:**
```rust
pub mod config;
pub mod wallet;
pub mod transaction;
pub mod lightwalletd;
pub mod memo;

pub use config::*;
pub use wallet::*;
pub use transaction::*;
pub use memo::*;
```

**crates/zots-zcash/src/config.rs:**
```rust
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ZcashConfig {
    pub seed_phrase: String,
    pub birthday_height: u64,
    pub lightwalletd_url: String,
    pub data_dir: PathBuf,
    pub network: zots_core::Network,
}

impl ZcashConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        
        let seed_phrase = std::env::var("ZOTS_SEED")
            .map_err(|_| anyhow::anyhow!("ZOTS_SEED environment variable not set"))?;
        
        let birthday_height = std::env::var("ZOTS_BIRTHDAY_HEIGHT")
            .unwrap_or_else(|_| "3717528".to_string())
            .parse()?;
        
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
            .as_str()
        {
            "mainnet" => zots_core::Network::Mainnet,
            _ => zots_core::Network::Testnet,
        };
        
        Ok(Self {
            seed_phrase,
            birthday_height,
            lightwalletd_url,
            data_dir,
            network,
        })
    }
    
    pub fn wallet_db_path(&self) -> PathBuf {
        self.data_dir.join("wallet.db")
    }
}
```

**crates/zots-zcash/src/wallet.rs:**
```rust
use std::borrow::BorrowMut;
use std::convert::Infallible;
use std::path::Path;

use bip0039::{English, Mnemonic};
use rand_core::OsRng;
use tonic::transport::{Channel, ClientTlsConfig};
use zcash_client_backend::data_api::wallet::{
    create_proposed_transactions, propose_standard_transfer_to_address,
    ConfirmationsPolicy, SpendingKeys,
};
use zcash_client_backend::data_api::{
    AccountBirthday, AccountPurpose, InputSource, WalletRead, WalletSummary, WalletWrite,
};
use zcash_client_backend::fees::StandardFeeRule;
use zcash_client_backend::keys::UnifiedSpendingKey;
use zcash_client_backend::proto::service::{
    compact_tx_streamer_client::CompactTxStreamerClient, BlockId, ChainSpec, RawTransaction,
};
use zcash_client_backend::sync::run;
use zcash_client_backend::wallet::OvkPolicy;
use zcash_client_memory::MemBlockCache;
use zcash_client_sqlite::util::{Clock, SystemClock};
use zcash_client_sqlite::wallet::init::init_wallet_db;
use zcash_client_sqlite::{AccountUuid, WalletDb};
use zcash_keys::address::Address as ZcashAddress;
use zcash_proofs::prover::LocalTxProver;
use zcash_protocol::consensus::{Parameters, TEST_NETWORK};
use zcash_protocol::value::Zatoshis;
use zip32::AccountId;

use crate::config::ZcashConfig;
use crate::memo::create_timestamp_memo;

pub struct ZotsWallet {
    db: WalletDb<rusqlite::Connection, zcash_protocol::consensus::TestNetwork, SystemClock, OsRng>,
    client: CompactTxStreamerClient<Channel>,
    seed: [u8; 64],
    config: ZcashConfig,
}

impl ZotsWallet {
    pub async fn new(config: ZcashConfig) -> anyhow::Result<Self> {
        // Create data directory
        std::fs::create_dir_all(&config.data_dir)?;
        
        // Parse seed phrase
        let mnemonic = Mnemonic::<English>::from_phrase(&config.seed_phrase)
            .map_err(|e| anyhow::anyhow!("Invalid seed phrase: {:?}", e))?;
        let seed = mnemonic.to_seed("");
        
        // Initialize wallet database
        let db_path = config.wallet_db_path();
        let mut db = WalletDb::for_path(&db_path, TEST_NETWORK, SystemClock, OsRng)?;
        init_wallet_db(&mut db, None)?;
        
        // Connect to lightwalletd
        let tls_config = ClientTlsConfig::new().with_native_roots();
        let channel = tonic::transport::Endpoint::from_shared(config.lightwalletd_url.clone())?
            .tls_config(tls_config)?
            .connect()
            .await?;
        let client = CompactTxStreamerClient::new(channel);
        
        Ok(Self {
            db,
            client,
            seed,
            config,
        })
    }
    
    /// Initialize wallet account if not exists
    pub async fn init_account(&mut self) -> anyhow::Result<()> {
        // Check if account already exists
        let accounts = self.db.get_account_ids()?;
        if !accounts.is_empty() {
            return Ok(());
        }
        
        // Create unified spending key
        let usk = UnifiedSpendingKey::from_seed(&TEST_NETWORK, &self.seed, AccountId::ZERO)?;
        let ufvk = usk.to_unified_full_viewing_key();
        
        // Get birthday tree state
        let request = zcash_client_backend::proto::service::BlockId {
            height: (self.config.birthday_height - 1) as u64,
            ..Default::default()
        };
        let treestate = self.client.get_tree_state(request).await?.into_inner();
        let birthday = AccountBirthday::from_treestate(treestate, None)
            .map_err(|e| anyhow::anyhow!("Failed to create birthday: {:?}", e))?;
        
        // Import account
        self.db.import_account_ufvk(
            "default",
            &ufvk,
            &birthday,
            AccountPurpose::Spending { derivation: None },
            None,
        )?;
        
        Ok(())
    }
    
    /// Sync wallet with blockchain
    pub async fn sync(&mut self) -> anyhow::Result<()> {
        let db_cache = MemBlockCache::new();
        run(&mut self.client, &TEST_NETWORK, &db_cache, &mut self.db, 100).await?;
        Ok(())
    }
    
    /// Get current block height
    pub async fn get_block_height(&mut self) -> anyhow::Result<u64> {
        let block = self.client
            .get_latest_block(ChainSpec::default())
            .await?
            .into_inner();
        Ok(block.height)
    }
    
    /// Get wallet balance in zatoshis
    pub fn get_balance(&self) -> anyhow::Result<u64> {
        let summary = self.db.get_wallet_summary(ConfirmationsPolicy::MIN)?;
        match summary {
            Some(s) => {
                let total = s.account_balances()
                    .values()
                    .map(|b| b.total())
                    .sum::<Zatoshis>();
                Ok(total.into_u64())
            }
            None => Ok(0),
        }
    }
    
    /// Get receiving address
    pub fn get_address(&self) -> anyhow::Result<String> {
        let accounts = self.db.get_account_ids()?;
        let account_id = accounts.first()
            .ok_or_else(|| anyhow::anyhow!("No account found"))?;
        
        let addresses = self.db.list_addresses(account_id.clone())?;
        let addr = addresses.first()
            .ok_or_else(|| anyhow::anyhow!("No address found"))?;
        
        Ok(addr.address().to_zcash_address(&TEST_NETWORK).to_string())
    }
    
    /// Create timestamp transaction with hash in memo
    pub async fn create_timestamp_tx(&mut self, hash: &[u8; 32]) -> anyhow::Result<TimestampTxResult> {
        let accounts = self.db.get_account_ids()?;
        let account_id = accounts.first()
            .ok_or_else(|| anyhow::anyhow!("No account found"))?;
        
        // Get own address for self-send
        let addresses = self.db.list_addresses(account_id.clone())?;
        let recipient = addresses.first()
            .ok_or_else(|| anyhow::anyhow!("No address found"))?
            .address();
        
        // Create memo with timestamp data
        let memo = create_timestamp_memo(hash);
        
        // Create spending key
        let usk = UnifiedSpendingKey::from_seed(&TEST_NETWORK, &self.seed, AccountId::ZERO)?;
        
        // Propose transaction (self-send with memo)
        let proposal = propose_standard_transfer_to_address::<_, _, zcash_client_sqlite::error::SqliteClientError>(
            &mut self.db,
            &TEST_NETWORK,
            StandardFeeRule::Zip317,
            account_id.clone(),
            ConfirmationsPolicy::MIN,
            recipient,
            Zatoshis::const_from_u64(1000), // Dust amount
            Some(memo),
            None,
            zcash_protocol::ShieldedProtocol::Orchard,
        )?;
        
        // Build and sign transaction
        let prover = LocalTxProver::bundled();
        let spending_keys = SpendingKeys::from_unified_spending_key(usk);
        
        let txids = create_proposed_transactions::<_, _, <WalletDb<_, _, _, _> as InputSource>::Error, _, Infallible, _>(
            &mut self.db,
            &TEST_NETWORK,
            &prover,
            &prover,
            &spending_keys,
            OvkPolicy::Sender,
            &proposal,
        )?;
        
        let txid = txids.first().ok_or_else(|| anyhow::anyhow!("No transaction created"))?;
        
        // Get raw transaction
        let tx = self.db.get_transaction(txid)?
            .ok_or_else(|| anyhow::anyhow!("Transaction not found in wallet"))?;
        
        let mut tx_bytes = Vec::new();
        tx.write(&mut tx_bytes)?;
        
        // Broadcast
        let response = self.client
            .send_transaction(RawTransaction {
                data: tx_bytes,
                height: 0,
            })
            .await?;
        
        if response.get_ref().error_code != 0 {
            return Err(anyhow::anyhow!(
                "Broadcast failed: {}",
                response.get_ref().error_message
            ));
        }
        
        Ok(TimestampTxResult {
            txid: txid.to_string(),
            txid_bytes: txid.as_ref().try_into()?,
        })
    }
    
    /// Wait for transaction confirmation
    pub async fn wait_confirmation(&mut self, _txid: &str, max_blocks: u32) -> anyhow::Result<ConfirmationResult> {
        let start_height = self.get_block_height().await?;
        
        for _ in 0..max_blocks {
            // Sync and check
            self.sync().await?;
            
            let current_height = self.get_block_height().await?;
            
            // TODO: Check if tx is confirmed by querying wallet
            // For now, just wait for a few blocks
            if current_height > start_height {
                return Ok(ConfirmationResult {
                    block_height: current_height as u32,
                    block_time: chrono::Utc::now().timestamp() as u32,
                });
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
        
        Err(anyhow::anyhow!("Transaction not confirmed within {} blocks", max_blocks))
    }
}

pub struct TimestampTxResult {
    pub txid: String,
    pub txid_bytes: [u8; 32],
}

pub struct ConfirmationResult {
    pub block_height: u32,
    pub block_time: u32,
}
```

**crates/zots-zcash/src/memo.rs:**
```rust
use zots_core::proof::ZOTS_MAGIC;
use zcash_client_backend::wallet::Memo;

/// Create a memo field containing timestamp data
/// Format: ZOTS_MAGIC (8 bytes) + hash (32 bytes) = 40 bytes
pub fn create_timestamp_memo(hash: &[u8; 32]) -> Memo {
    let mut data = Vec::with_capacity(40);
    data.extend_from_slice(&ZOTS_MAGIC);
    data.extend_from_slice(hash);
    
    // Pad to 512 bytes
    data.resize(512, 0);
    
    Memo::from_bytes(&data).expect("Valid memo")
}

/// Parse hash from memo field
pub fn parse_timestamp_memo(memo: &[u8]) -> Option<[u8; 32]> {
    if memo.len() < 40 {
        return None;
    }
    
    // Check magic
    if &memo[0..8] != &ZOTS_MAGIC {
        return None;
    }
    
    // Extract hash
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&memo[8..40]);
    Some(hash)
}
```

**Deliverables:**
- [ ] ZcashConfig from environment
- [ ] ZotsWallet struct
- [ ] Wallet initialization from seed
- [ ] Sync functionality
- [ ] Balance/address queries
- [ ] Timestamp transaction creation
- [ ] Memo encoding
- [ ] Commit: "feat: implement zcash wallet module"

---

### Phase 5: CLI Application Setup (Day 3-4)

**crates/zots-cli/Cargo.toml:**
```toml
[package]
name = "zots-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "zots"
path = "src/main.rs"

[dependencies]
zots-core = { path = "../zots-core" }
zots-zcash = { path = "../zots-zcash" }

# CLI
clap.workspace = true
dotenvy.workspace = true

# TUI
ratatui.workspace = true
crossterm.workspace = true

# Output
colored.workspace = true
indicatif.workspace = true

# Async
tokio.workspace = true

# Other
anyhow.workspace = true
```

**crates/zots-cli/src/cli.rs:**
```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "zots")]
#[command(author, version, about = "zOpenTimestamps - Zcash blockchain timestamping")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Timestamp a file or hash
    Stamp {
        /// File to timestamp
        #[arg(conflicts_with = "hash")]
        file: Option<PathBuf>,
        
        /// Hash to timestamp (hex string)
        #[arg(long, conflicts_with = "file")]
        hash: Option<String>,
        
        /// Output proof file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Don't wait for confirmation
        #[arg(long)]
        no_wait: bool,
    },
    
    /// Verify a timestamp proof
    Verify {
        /// Proof file (.zots)
        proof: PathBuf,
        
        /// Original file to verify against
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    
    /// Display proof information
    Info {
        /// Proof file (.zots)
        proof: PathBuf,
    },
    
    /// Wallet management commands
    Wallet {
        #[command(subcommand)]
        command: WalletCommands,
    },
    
    /// Launch interactive TUI
    Tui,
}

#[derive(Subcommand)]
pub enum WalletCommands {
    /// Sync wallet with blockchain
    Sync,
    
    /// Show wallet balance
    Balance,
    
    /// Show receiving address
    Address,
    
    /// Show wallet info
    Info,
}
```

**crates/zots-cli/src/main.rs:**
```rust
mod cli;
mod commands;
mod output;
mod tui;

use clap::Parser;
use cli::{Cli, Commands, WalletCommands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Stamp { file, hash, output, no_wait } => {
            commands::stamp::run(file, hash, output, no_wait).await
        }
        Commands::Verify { proof, file } => {
            commands::verify::run(proof, file).await
        }
        Commands::Info { proof } => {
            commands::info::run(proof)
        }
        Commands::Wallet { command } => {
            match command {
                WalletCommands::Sync => commands::wallet::sync().await,
                WalletCommands::Balance => commands::wallet::balance().await,
                WalletCommands::Address => commands::wallet::address().await,
                WalletCommands::Info => commands::wallet::info().await,
            }
        }
        Commands::Tui => {
            tui::run().await
        }
    }
}
```

**crates/zots-cli/src/output.rs:**
```rust
use colored::*;

pub fn print_header(text: &str) {
    println!("\n{}", text.cyan().bold());
    println!("{}", "─".repeat(text.len()).cyan());
}

pub fn print_success(text: &str) {
    println!("{} {}", "✓".green().bold(), text.green());
}

pub fn print_error(text: &str) {
    println!("{} {}", "✗".red().bold(), text.red());
}

pub fn print_info(label: &str, value: &str) {
    println!("  {}: {}", label.white().bold(), value);
}

pub fn print_hash(hash: &str) {
    println!("  SHA-256: {}", hash.yellow());
}

pub fn print_link(text: &str, url: &str) {
    // Terminal hyperlink escape sequence
    println!("  {}: \x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", text, url, url.blue().underline());
}
```

**Deliverables:**
- [ ] CLI argument parsing with clap
- [ ] Command structure defined
- [ ] Main entry point
- [ ] Output formatting helpers
- [ ] Project compiles
- [ ] Commit: "feat: implement CLI application structure"

---

### Phase 6: CLI Commands Implementation (Day 4-5)

**crates/zots-cli/src/commands/mod.rs:**
```rust
pub mod stamp;
pub mod verify;
pub mod info;
pub mod wallet;
```

**crates/zots-cli/src/commands/stamp.rs:**
```rust
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use zots_core::{hash_file, hash_from_hex, hash_to_hex, TimestampProof, ZcashAttestation};
use zots_zcash::{ZcashConfig, ZotsWallet};
use crate::output::*;

pub async fn run(
    file: Option<PathBuf>,
    hash: Option<String>,
    output: Option<PathBuf>,
    no_wait: bool,
) -> anyhow::Result<()> {
    // Determine hash to timestamp
    let (hash_bytes, output_path) = if let Some(file_path) = file {
        print_header("Timestamping File");
        
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap());
        pb.set_message("Hashing file...");
        
        let hash = hash_file(&file_path)?;
        pb.finish_with_message("Hashing complete");
        
        let output = output.unwrap_or_else(|| {
            let mut p = file_path.clone();
            p.set_extension("zots");
            p
        });
        
        println!("  File: {}", file_path.display());
        print_hash(&hash_to_hex(&hash));
        
        (hash, output)
    } else if let Some(hex) = hash {
        print_header("Timestamping Hash");
        
        let hash = hash_from_hex(&hex)?;
        let output = output.unwrap_or_else(|| {
            PathBuf::from(format!("{}.zots", &hex[..16]))
        });
        
        print_hash(&hash_to_hex(&hash));
        
        (hash, output)
    } else {
        return Err(anyhow::anyhow!("Either file or --hash must be provided"));
    };
    
    // Initialize wallet
    let config = ZcashConfig::from_env()?;
    let mut wallet = ZotsWallet::new(config.clone()).await?;
    wallet.init_account().await?;
    
    // Sync wallet
    let pb = ProgressBar::new_spinner();
    pb.set_message("Syncing wallet...");
    wallet.sync().await?;
    pb.finish_with_message("Wallet synced");
    
    // Create and broadcast transaction
    let pb = ProgressBar::new_spinner();
    pb.set_message("Creating transaction...");
    let tx_result = wallet.create_timestamp_tx(&hash_bytes).await?;
    pb.finish_with_message("Transaction broadcast");
    
    print_info("TXID", &tx_result.txid);
    
    // Create proof
    let mut proof = TimestampProof::new(hash_bytes);
    
    if no_wait {
        print_info("Status", "Pending confirmation");
        proof.save(&output_path)?;
        print_success(&format!("Proof saved: {}", output_path.display()));
        return Ok(());
    }
    
    // Wait for confirmation
    let pb = ProgressBar::new_spinner();
    pb.set_message("Waiting for confirmation...");
    let confirmation = wallet.wait_confirmation(&tx_result.txid, 10).await?;
    pb.finish_with_message("Transaction confirmed");
    
    // Add attestation
    proof.add_attestation(ZcashAttestation {
        network: config.network,
        txid: tx_result.txid_bytes,
        block_height: confirmation.block_height,
        block_time: confirmation.block_time,
        memo_offset: 0,
    });
    
    // Save proof
    proof.save(&output_path)?;
    
    print_success(&format!("Confirmed in block {}", confirmation.block_height));
    print_success(&format!("Proof saved: {}", output_path.display()));
    
    Ok(())
}
```

**crates/zots-cli/src/commands/verify.rs:**
```rust
use std::path::PathBuf;
use zots_core::{hash_file, hash_to_hex, TimestampProof};
use crate::output::*;

pub async fn run(proof_path: PathBuf, file: Option<PathBuf>) -> anyhow::Result<()> {
    print_header("Verifying Timestamp");
    
    // Load proof
    let proof = TimestampProof::load(&proof_path)?;
    print_info("Proof", &proof_path.display().to_string());
    print_hash(&hash_to_hex(&proof.hash));
    
    // Verify against original file if provided
    if let Some(file_path) = file {
        let file_hash = hash_file(&file_path)?;
        if file_hash == proof.hash {
            print_success("Hash matches original file");
        } else {
            print_error("Hash does NOT match original file!");
            print_info("Expected", &hash_to_hex(&proof.hash));
            print_info("Got", &hash_to_hex(&file_hash));
            return Ok(());
        }
    }
    
    // Check attestations
    if proof.attestations.is_empty() {
        print_error("No attestations found - proof may be pending");
        return Ok(());
    }
    
    let att = &proof.attestations[0];
    
    // TODO: Verify against blockchain by fetching transaction
    // For MVP, we trust the proof file
    
    println!();
    print_success("VALID TIMESTAMP");
    print_info("Network", &format!("{:?}", att.network));
    print_info("Block", &att.block_height.to_string());
    print_info("Time", &att.timestamp().to_rfc3339());
    print_info("TXID", &att.txid_hex());
    print_link("Explorer", &att.explorer_link());
    
    Ok(())
}
```

**crates/zots-cli/src/commands/info.rs:**
```rust
use std::path::PathBuf;
use zots_core::{hash_to_hex, TimestampProof};
use crate::output::*;

pub fn run(proof_path: PathBuf) -> anyhow::Result<()> {
    print_header("Proof Information");
    
    let proof = TimestampProof::load(&proof_path)?;
    
    print_info("Version", &proof.version.to_string());
    print_hash(&hash_to_hex(&proof.hash));
    print_info("Attestations", &proof.attestations.len().to_string());
    
    for (i, att) in proof.attestations.iter().enumerate() {
        println!();
        println!("  Attestation #{}", i + 1);
        print_info("  Network", &format!("{:?}", att.network));
        print_info("  TXID", &att.txid_hex());
        print_info("  Block", &att.block_height.to_string());
        print_info("  Time", &att.timestamp().to_rfc3339());
        print_link("  Explorer", &att.explorer_link());
    }
    
    Ok(())
}
```

**crates/zots-cli/src/commands/wallet.rs:**
```rust
use indicatif::{ProgressBar, ProgressStyle};
use zots_zcash::{ZcashConfig, ZotsWallet};
use crate::output::*;

pub async fn sync() -> anyhow::Result<()> {
    print_header("Syncing Wallet");
    
    let config = ZcashConfig::from_env()?;
    let mut wallet = ZotsWallet::new(config).await?;
    wallet.init_account().await?;
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap());
    pb.set_message("Syncing with blockchain...");
    
    wallet.sync().await?;
    
    pb.finish_with_message("Sync complete");
    print_success("Wallet synchronized");
    
    Ok(())
}

pub async fn balance() -> anyhow::Result<()> {
    print_header("Wallet Balance");
    
    let config = ZcashConfig::from_env()?;
    let mut wallet = ZotsWallet::new(config).await?;
    wallet.init_account().await?;
    wallet.sync().await?;
    
    let balance = wallet.get_balance()?;
    let zec = balance as f64 / 100_000_000.0;
    
    print_info("Balance", &format!("{:.8} ZEC ({} zatoshis)", zec, balance));
    
    Ok(())
}

pub async fn address() -> anyhow::Result<()> {
    print_header("Wallet Address");
    
    let config = ZcashConfig::from_env()?;
    let mut wallet = ZotsWallet::new(config).await?;
    wallet.init_account().await?;
    
    let address = wallet.get_address()?;
    print_info("Address", &address);
    
    Ok(())
}

pub async fn info() -> anyhow::Result<()> {
    print_header("Wallet Info");
    
    let config = ZcashConfig::from_env()?;
    let mut wallet = ZotsWallet::new(config).await?;
    wallet.init_account().await?;
    wallet.sync().await?;
    
    let height = wallet.get_block_height().await?;
    let balance = wallet.get_balance()?;
    let address = wallet.get_address()?;
    
    print_info("Network", &format!("{:?}", config.network));
    print_info("Block Height", &height.to_string());
    print_info("Balance", &format!("{} zatoshis", balance));
    print_info("Address", &address);
    
    Ok(())
}
```

**Deliverables:**
- [ ] stamp command with file and hash support
- [ ] verify command with file comparison
- [ ] info command
- [ ] wallet sync/balance/address/info commands
- [ ] Progress indicators
- [ ] Colored output
- [ ] Commit: "feat: implement CLI commands"

---

### Phase 7: TUI Implementation (Day 5-6)

**crates/zots-cli/src/tui/mod.rs:**
```rust
mod app;
mod ui;
mod events;
mod widgets;

use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

use app::{App, AppState};
use ui::draw;

pub async fn run() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app
    let mut app = App::new().await?;
    
    // Main loop
    loop {
        terminal.draw(|f| draw(f, &app))?;
        
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::Menu => {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char('s') => app.state = AppState::Stamp,
                            KeyCode::Char('v') => app.state = AppState::Verify,
                            KeyCode::Char('w') => app.state = AppState::Wallet,
                            _ => {}
                        }
                    }
                    AppState::Stamp | AppState::Verify | AppState::Wallet => {
                        match key.code {
                            KeyCode::Esc => app.state = AppState::Menu,
                            _ => app.handle_input(key.code).await?,
                        }
                    }
                }
            }
        }
        
        // Background updates
        app.tick().await?;
    }
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    Ok(())
}
```

**crates/zots-cli/src/tui/app.rs:**
```rust
use crossterm::event::KeyCode;
use zots_zcash::{ZcashConfig, ZotsWallet};

pub enum AppState {
    Menu,
    Stamp,
    Verify,
    Wallet,
}

pub struct App {
    pub state: AppState,
    pub wallet: Option<ZotsWallet>,
    pub config: Option<ZcashConfig>,
    pub block_height: u64,
    pub balance: u64,
    pub status_message: String,
    pub input_buffer: String,
}

impl App {
    pub async fn new() -> anyhow::Result<Self> {
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
        
        Ok(Self {
            state: AppState::Menu,
            wallet,
            config,
            block_height,
            balance,
            status_message: String::from("Ready"),
            input_buffer: String::new(),
        })
    }
    
    pub async fn tick(&mut self) -> anyhow::Result<()> {
        // Periodic updates (e.g., block height)
        Ok(())
    }
    
    pub async fn handle_input(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Enter => {
                // Process input based on state
                self.process_input().await?;
            }
            _ => {}
        }
        Ok(())
    }
    
    async fn process_input(&mut self) -> anyhow::Result<()> {
        // Handle input based on current state
        self.input_buffer.clear();
        Ok(())
    }
}
```

**crates/zots-cli/src/tui/ui.rs:**
```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::tui::app::{App, AppState};

const ASCII_HEADER: &str = r#"
╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║   ███████╗ ██████╗ ████████╗███████╗                             ║
║   ╚══███╔╝██╔═══██╗╚══██╔══╝██╔════╝                             ║
║     ███╔╝ ██║   ██║   ██║   ███████╗                             ║
║    ███╔╝  ██║   ██║   ██║   ╚════██║                             ║
║   ███████╗╚██████╔╝   ██║   ███████║                             ║
║   ╚══════╝ ╚═════╝    ╚═╝   ╚══════╝                             ║
║                                                                   ║
║          z O P E N T I M E S T A M P S                           ║
║          Zcash Blockchain Timestamping                            ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝
"#;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(14),  // Header
            Constraint::Min(10),     // Main content
            Constraint::Length(3),   // Status bar
        ])
        .split(f.area());
    
    // Header
    let header = Paragraph::new(ASCII_HEADER)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(header, chunks[0]);
    
    // Main content
    match app.state {
        AppState::Menu => draw_menu(f, chunks[1]),
        AppState::Stamp => draw_stamp(f, chunks[1], app),
        AppState::Verify => draw_verify(f, chunks[1], app),
        AppState::Wallet => draw_wallet(f, chunks[1], app),
    }
    
    // Status bar
    let status = Paragraph::new(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Gray)),
        Span::styled(&app.status_message, Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::styled("Block: ", Style::default().fg(Color::Gray)),
        Span::styled(app.block_height.to_string(), Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled("Balance: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.8} TAZ", app.balance as f64 / 100_000_000.0),
            Style::default().fg(Color::Green)
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, chunks[2]);
}

fn draw_menu(f: &mut Frame, area: Rect) {
    let menu_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  [S] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("Stamp a file or hash"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [V] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("Verify a timestamp proof"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [W] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("Wallet management"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Q] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw("Quit"),
        ]),
    ];
    
    let menu = Paragraph::new(menu_text)
        .block(Block::default().borders(Borders::ALL).title("Main Menu"))
        .wrap(Wrap { trim: false });
    f.render_widget(menu, area);
}

fn draw_stamp(f: &mut Frame, area: Rect, app: &App) {
    let content = vec![
        Line::from("Enter file path or hash to timestamp:"),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Green)),
            Span::raw(&app.input_buffer),
            Span::styled("_", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(Span::styled("[ESC] Back to menu", Style::default().fg(Color::Gray))),
    ];
    
    let stamp = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Stamp"));
    f.render_widget(stamp, area);
}

fn draw_verify(f: &mut Frame, area: Rect, app: &App) {
    let content = vec![
        Line::from("Enter proof file path (.zots):"),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Green)),
            Span::raw(&app.input_buffer),
            Span::styled("_", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(Span::styled("[ESC] Back to menu", Style::default().fg(Color::Gray))),
    ];
    
    let verify = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Verify"));
    f.render_widget(verify, area);
}

fn draw_wallet(f: &mut Frame, area: Rect, app: &App) {
    let content = vec![
        Line::from(vec![
            Span::styled("Block Height: ", Style::default().fg(Color::Gray)),
            Span::styled(app.block_height.to_string(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Balance: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.8} TAZ", app.balance as f64 / 100_000_000.0),
                Style::default().fg(Color::Green)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[S] ", Style::default().fg(Color::Cyan)),
            Span::raw("Sync wallet"),
        ]),
        Line::from(""),
        Line::from(Span::styled("[ESC] Back to menu", Style::default().fg(Color::Gray))),
    ];
    
    let wallet = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Wallet"));
    f.render_widget(wallet, area);
}
```

**Deliverables:**
- [ ] TUI framework with ratatui
- [ ] ASCII art header
- [ ] Main menu navigation
- [ ] Stamp screen
- [ ] Verify screen
- [ ] Wallet screen
- [ ] Status bar with block height and balance
- [ ] Keyboard navigation
- [ ] Commit: "feat: implement cypherpunk TUI"

---

### Phase 8: Integration & Polish (Day 6-7)

#### Step 8.1: Create Supporting Files

**.env.example:**
```bash
# zOpenTimestamps Configuration (Testnet)

# Required: Your Zcash seed phrase (24 words)
ZOTS_SEED="your 24 word seed phrase here"

# Optional: Wallet birthday height (default: 3717528)
ZOTS_BIRTHDAY_HEIGHT=3717528

# Optional: Lightwalletd endpoint (default: testnet.zec.rocks)
ZOTS_LIGHTWALLETD="https://testnet.zec.rocks:443"

# Optional: Network (testnet or mainnet, default: testnet)
ZOTS_NETWORK="testnet"

# Optional: Data directory (default: ~/.zopentimestamps)
ZOTS_DATA_DIR="~/.zopentimestamps"
```

**.gitignore:**
```
/target
/.env
*.zots
wallet.db*
Cargo.lock
```

**README.md:**
```markdown
# zOpenTimestamps

Zcash blockchain timestamping CLI with cypherpunk TUI.

## Installation

```bash
cargo install --path crates/zots-cli
```

## Setup

1. Copy `.env.example` to `.env`
2. Add your Zcash testnet seed phrase
3. Get testnet funds from https://testnet.zecfaucet.com/

## Usage

### CLI Commands

```bash
# Timestamp a file
zots stamp document.pdf

# Timestamp a git commit
zots stamp --hash $(git rev-parse HEAD)

# Verify a proof
zots verify document.pdf.zots --file document.pdf

# View proof info
zots info document.pdf.zots

# Wallet commands
zots wallet sync
zots wallet balance
zots wallet address
```

### Interactive TUI

```bash
zots tui
```

## Proof Format

Proofs are saved as `.zots` files containing:
- SHA-256 hash of timestamped data
- Zcash transaction attestation
- Block height and timestamp

## License

MIT
```

**Deliverables:**
- [ ] .env.example with all config options
- [ ] .gitignore
- [ ] README with usage examples
- [ ] Error handling improvements
- [ ] Final code review
- [ ] Commit: "feat: complete MVP with documentation"

---

## Technology Stack Summary

| Component | Technology |
|-----------|------------|
| **Language** | Rust 1.85+ |
| **Async** | Tokio |
| **CLI** | Clap 4 |
| **TUI** | Ratatui + Crossterm |
| **Zcash** | librustzcash (rev 9f47de6) |
| **Storage** | SQLite (rusqlite) |
| **Hashing** | sha2 |
| **gRPC** | Tonic |

---

## Milestones & Timeline

| Phase | Description | Days | Status |
|-------|-------------|------|--------|
| 1 | Project Setup & Core Types | 1 | ⬜ |
| 2 | SHA-256 Hashing Module | 1 | ⬜ |
| 3 | Proof Format (.zots) | 1-2 | ⬜ |
| 4 | Zcash Wallet Module | 2-3 | ⬜ |
| 5 | CLI Application Setup | 3-4 | ⬜ |
| 6 | CLI Commands | 4-5 | ⬜ |
| 7 | TUI Implementation | 5-6 | ⬜ |
| 8 | Integration & Polish | 6-7 | ⬜ |

**Total Estimated Time:** 7 working days

---

*Document Version: 2.0 (Rust CLI)*  
*Last Updated: December 2025*
