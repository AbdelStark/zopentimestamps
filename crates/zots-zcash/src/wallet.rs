//! Zcash wallet management for timestamping
//!
//! Provides wallet initialization, sync, and transaction creation
//! for timestamping operations on the Zcash blockchain.

use bip0039::{English, Mnemonic};
use rand_core::OsRng;
use tonic::transport::{Channel, ClientTlsConfig};
use zcash_client_backend::data_api::wallet::{
    ConfirmationsPolicy, propose_shielding,
};
use zcash_client_backend::data_api::{
    AccountBirthday, AccountPurpose, WalletRead, WalletWrite,
};
use zcash_client_backend::fees::standard::SingleOutputChangeStrategy;
use zcash_client_backend::fees::{DustOutputPolicy, StandardFeeRule};
use zcash_client_backend::data_api::wallet::input_selection::GreedyInputSelector;
use zcash_client_sqlite::error::SqliteClientError;
use zcash_protocol::ShieldedProtocol;
use zcash_client_backend::keys::UnifiedSpendingKey;
use zcash_client_backend::proto::service::{
    self, ChainSpec, compact_tx_streamer_client::CompactTxStreamerClient,
};
use zcash_client_backend::sync::run as sync_run;
use zcash_client_memory::MemBlockCache;
use zcash_client_sqlite::WalletDb;
use zcash_client_sqlite::util::SystemClock;
use zcash_client_sqlite::wallet::init::init_wallet_db;
use zcash_protocol::consensus::TEST_NETWORK;
use zcash_protocol::value::Zatoshis;
use zip32::AccountId;

use crate::config::ZcashConfig;

const SYNC_BATCH_SIZE: u32 = 1000;

/// Result of creating a timestamp transaction
pub struct TimestampTxResult {
    /// Transaction ID as string
    pub txid: String,
    /// Transaction ID as bytes (internal byte order)
    pub txid_bytes: [u8; 32],
}

/// Result of waiting for transaction confirmation
pub struct ConfirmationResult {
    /// Block height where transaction was confirmed
    pub block_height: u32,
    /// Block timestamp (Unix timestamp)
    pub block_time: u32,
}

/// Balance breakdown by shielded pool
#[derive(Default)]
pub struct BalanceBreakdown {
    /// Transparent pool balance in zatoshis
    pub transparent: u64,
    /// Sapling pool balance in zatoshis
    pub sapling: u64,
    /// Orchard pool balance in zatoshis
    pub orchard: u64,
}

type ZotsWalletDb =
    WalletDb<rusqlite::Connection, zcash_protocol::consensus::TestNetwork, SystemClock, OsRng>;

/// Zcash wallet for timestamping operations
pub struct ZotsWallet {
    config: ZcashConfig,
    db: ZotsWalletDb,
    client: CompactTxStreamerClient<Channel>,
    seed: [u8; 64],
}

impl ZotsWallet {
    /// Create a new wallet instance
    ///
    /// Initializes the wallet database and connects to lightwalletd.
    pub async fn new(config: ZcashConfig) -> anyhow::Result<Self> {
        // Create data directory
        config.ensure_data_dir()?;

        // Parse seed phrase
        let mnemonic = Mnemonic::<English>::from_phrase(&config.seed_phrase)
            .map_err(|e| anyhow::anyhow!("Invalid seed phrase: {:?}", e))?;
        let seed = mnemonic.to_seed("");

        // Initialize wallet database
        let db_path = config.wallet_db_path();
        let mut db = WalletDb::for_path(&db_path, TEST_NETWORK, SystemClock, OsRng)?;
        init_wallet_db(&mut db, None)?;

        // Connect to lightwalletd with TLS
        let tls_config = ClientTlsConfig::new().with_native_roots();
        let channel = tonic::transport::Endpoint::from_shared(config.lightwalletd_url.clone())?
            .tls_config(tls_config)?
            .connect()
            .await?;
        let client = CompactTxStreamerClient::new(channel);

        Ok(Self {
            config,
            db,
            client,
            seed,
        })
    }

    /// Initialize the wallet account if it doesn't exist
    ///
    /// Creates the account from seed and imports it into the wallet database.
    pub async fn init_account(&mut self) -> anyhow::Result<()> {
        // Check if account already exists
        let accounts = self.db.get_account_ids()?;
        if !accounts.is_empty() {
            return Ok(());
        }

        // Create unified spending key from seed
        let account_id = AccountId::ZERO;
        let usk = UnifiedSpendingKey::from_seed(&TEST_NETWORK, &self.seed, account_id)
            .map_err(|e| anyhow::anyhow!("Failed to derive spending key: {:?}", e))?;
        let ufvk = usk.to_unified_full_viewing_key();

        // Get birthday tree state from lightwalletd
        let birthday_height = self.config.birthday_height;
        let request = service::BlockId {
            height: birthday_height.saturating_sub(1),
            ..Default::default()
        };
        let treestate = self.client.get_tree_state(request).await?.into_inner();

        let birthday = AccountBirthday::from_treestate(treestate, None)
            .map_err(|_| anyhow::anyhow!("Failed to create birthday from tree state"))?;

        // Import account into wallet
        self.db.import_account_ufvk(
            "zots-wallet",
            &ufvk,
            &birthday,
            AccountPurpose::Spending { derivation: None },
            None,
        )?;

        Ok(())
    }

    /// Reset and reinitialize wallet with a new birthday height
    ///
    /// This is useful if the birthday height was set too high and transactions were missed.
    /// WARNING: This will delete the existing wallet database!
    pub async fn reset_wallet(&mut self) -> anyhow::Result<()> {
        let db_path = self.config.wallet_db_path();
        Err(anyhow::anyhow!(
            "To reset the wallet, delete the database file at: {:?}\n\
            Then set ZOTS_BIRTHDAY_HEIGHT to an earlier block and restart.",
            db_path
        ))
    }

    /// Sync wallet with the blockchain
    ///
    /// Downloads compact blocks and scans for transactions belonging to this wallet.
    pub async fn sync(&mut self) -> anyhow::Result<()> {
        // Use in-memory block cache for sync
        let db_cache = MemBlockCache::new();

        // Run the sync - downloads blocks and scans for our transactions
        sync_run(
            &mut self.client,
            &TEST_NETWORK,
            &db_cache,
            &mut self.db,
            SYNC_BATCH_SIZE,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Sync failed: {:?}", e))?;

        Ok(())
    }

    /// Get current block height from lightwalletd
    pub async fn get_block_height(&mut self) -> anyhow::Result<u64> {
        let response = self
            .client
            .get_latest_block(ChainSpec::default())
            .await?
            .into_inner();
        Ok(response.height)
    }

    /// Get wallet balance in zatoshis
    pub fn get_balance(&self) -> anyhow::Result<u64> {
        let summary = self.db.get_wallet_summary(ConfirmationsPolicy::MIN)?;
        match summary {
            Some(s) => {
                let mut total = Zatoshis::ZERO;
                for (_account_id, balance) in s.account_balances() {
                    total = (total + balance.total())
                        .ok_or_else(|| anyhow::anyhow!("Balance overflow"))?;
                }
                Ok(u64::from(total))
            }
            None => Ok(0),
        }
    }

    /// Get detailed balance breakdown by pool
    pub fn get_balance_breakdown(&self) -> anyhow::Result<BalanceBreakdown> {
        let summary = self.db.get_wallet_summary(ConfirmationsPolicy::MIN)?;
        match summary {
            Some(s) => {
                let mut breakdown = BalanceBreakdown::default();
                for (_account_id, balance) in s.account_balances() {
                    breakdown.transparent += u64::from(balance.unshielded_balance().spendable_value());
                    breakdown.sapling += u64::from(balance.sapling_balance().spendable_value());
                    breakdown.orchard += u64::from(balance.orchard_balance().spendable_value());
                }
                Ok(breakdown)
            }
            None => Ok(BalanceBreakdown::default()),
        }
    }

    /// Get receiving address
    pub fn get_address(&self) -> anyhow::Result<String> {
        let accounts = self.db.get_account_ids()?;
        let account_id = accounts
            .first()
            .ok_or_else(|| anyhow::anyhow!("No account found - run init_account first"))?;

        let addresses = self.db.list_addresses(*account_id)?;

        if addresses.is_empty() {
            return Err(anyhow::anyhow!("No addresses found for account"));
        }

        let address_record = addresses.first().unwrap();
        let address = address_record.address();
        Ok(address.to_zcash_address(&TEST_NETWORK).to_string())
    }

    /// Shield transparent funds to Orchard
    ///
    /// Moves funds from transparent pool to shielded Orchard pool.
    #[allow(dead_code)]
    pub async fn shield_transparent_funds(&mut self) -> anyhow::Result<String> {
        let accounts = self.db.get_account_ids()?;
        let account_id = accounts
            .first()
            .ok_or_else(|| anyhow::anyhow!("No account found"))?;

        // Create change strategy for shielding
        let dust_policy = DustOutputPolicy::default();
        let change_strategy = SingleOutputChangeStrategy::new(
            StandardFeeRule::Zip317,
            None,
            ShieldedProtocol::Orchard,
            dust_policy,
        );

        let input_selector = GreedyInputSelector::<ZotsWalletDb>::new();

        // Propose shielding
        let _proposal = propose_shielding::<_, _, _, _, SqliteClientError>(
            &mut self.db,
            &TEST_NETWORK,
            &input_selector,
            &change_strategy,
            Zatoshis::ZERO,
            &[],
            *account_id,
            ConfirmationsPolicy::MIN,
        )
        .map_err(|e| anyhow::anyhow!("Failed to propose shielding: {:?}", e))?;

        // TODO: Implement full transaction building with zk-SNARK provers
        Err(anyhow::anyhow!(
            "Shielding requires zk-SNARK provers (not yet integrated).\n\
            Please use Zingo or zcashd to shield your funds."
        ))
    }

    /// Create and broadcast a timestamp transaction
    ///
    /// Creates a transaction to anchor the file hash on the Zcash blockchain.
    pub async fn create_timestamp_tx(
        &mut self,
        _hash: &[u8; 32],
    ) -> anyhow::Result<TimestampTxResult> {
        let accounts = self.db.get_account_ids()?;
        let account_id = accounts
            .first()
            .ok_or_else(|| anyhow::anyhow!("No account found"))?;

        // Check balance in each pool
        let summary = self.db.get_wallet_summary(ConfirmationsPolicy::MIN)?;
        let (transparent_balance, orchard_balance, sapling_balance) = summary
            .as_ref()
            .and_then(|s| s.account_balances().get(account_id))
            .map(|b| {
                (
                    u64::from(b.unshielded_balance().spendable_value()),
                    u64::from(b.orchard_balance().spendable_value()),
                    u64::from(b.sapling_balance().spendable_value()),
                )
            })
            .unwrap_or((0, 0, 0));

        let total_shielded = orchard_balance + sapling_balance;
        let min_required = 20000u64; // ZIP-317 minimum fee

        // Check if we have sufficient shielded funds
        if total_shielded >= min_required {
            // TODO: Implement shielded transaction creation with memo
            return Err(anyhow::anyhow!(
                "You have {:.8} ZEC in shielded pools.\n\n\
                Full transaction support (with zk-SNARK proofs) is coming soon.\n\
                For now, use Zingo wallet to send a memo transaction with your file hash.",
                total_shielded as f64 / 100_000_000.0
            ));
        }

        // If only transparent funds available
        if transparent_balance >= min_required {
            return Err(anyhow::anyhow!(
                "Your funds are in the transparent pool.\n\
                Shield them first using Zingo or zcashd, then try again."
            ));
        }

        // No funds at all
        Err(anyhow::anyhow!(
            "Insufficient funds. Run 'zots wallet address' and fund your wallet."
        ))
    }

    /// Wait for transaction confirmation
    ///
    /// Polls the blockchain until the transaction is confirmed or max_blocks is reached.
    pub async fn wait_confirmation(
        &mut self,
        txid: &str,
        max_blocks: u32,
    ) -> anyhow::Result<ConfirmationResult> {
        let start_height = self.get_block_height().await?;

        for _ in 0..max_blocks {
            self.sync().await?;
            let current_height = self.get_block_height().await?;

            if current_height > start_height {
                let block_time = chrono::Utc::now().timestamp() as u32;
                return Ok(ConfirmationResult {
                    block_height: current_height as u32,
                    block_time,
                });
            }

            // Wait before next check (Zcash block time ~75 seconds)
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }

        Err(anyhow::anyhow!(
            "Transaction {} not confirmed within {} blocks",
            txid,
            max_blocks
        ))
    }

    /// Get the wallet configuration
    pub fn config(&self) -> &ZcashConfig {
        &self.config
    }
}
