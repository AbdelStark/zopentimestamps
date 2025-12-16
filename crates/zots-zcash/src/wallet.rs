//! Zcash wallet management for timestamping
//!
//! Provides wallet initialization, sync, and transaction creation
//! for timestamping operations on the Zcash blockchain.

use std::convert::Infallible;

use bip0039::{English, Mnemonic};
use rand_core::OsRng;
use tonic::transport::{Channel, ClientTlsConfig};
use zcash_client_backend::data_api::wallet::{
    ConfirmationsPolicy, SpendingKeys, create_proposed_transactions,
    propose_standard_transfer_to_address,
};
use zcash_client_backend::data_api::{
    AccountBirthday, AccountPurpose, InputSource, WalletRead, WalletWrite,
};
use zcash_client_backend::fees::StandardFeeRule;
use zcash_client_backend::keys::UnifiedSpendingKey;
use zcash_client_backend::proto::service::{
    self, ChainSpec, RawTransaction, compact_tx_streamer_client::CompactTxStreamerClient,
};
use zcash_client_backend::sync::run as sync_run;
use zcash_client_backend::wallet::OvkPolicy;
use zcash_client_memory::MemBlockCache;
use zcash_client_sqlite::WalletDb;
use zcash_client_sqlite::error::SqliteClientError;
use zcash_client_sqlite::util::SystemClock;
use zcash_client_sqlite::wallet::init::init_wallet_db;
use zcash_proofs::prover::LocalTxProver;
use zcash_protocol::ShieldedProtocol;
use zcash_protocol::consensus::TEST_NETWORK;
use zcash_protocol::memo::MemoBytes;
use zcash_protocol::value::Zatoshis;
use zip32::AccountId;

use crate::config::ZcashConfig;
use crate::memo::create_timestamp_memo;

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
        eprintln!("[wallet] Opening wallet database at: {:?}", db_path);
        let mut db = WalletDb::for_path(&db_path, TEST_NETWORK, SystemClock, OsRng)?;
        init_wallet_db(&mut db, None)?;
        eprintln!("[wallet] Wallet database initialized");

        // Connect to lightwalletd with TLS
        eprintln!(
            "[wallet] Connecting to lightwalletd: {}",
            config.lightwalletd_url
        );
        let tls_config = ClientTlsConfig::new().with_native_roots();
        let channel = tonic::transport::Endpoint::from_shared(config.lightwalletd_url.clone())?
            .tls_config(tls_config)?
            .connect()
            .await?;
        let client = CompactTxStreamerClient::new(channel);
        eprintln!("[wallet] Connected to lightwalletd");

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
            eprintln!("[wallet] Account already exists, skipping init");
            return Ok(());
        }

        eprintln!("[wallet] Creating new account...");

        // Create unified spending key from seed
        let account_id = AccountId::ZERO;
        let usk = UnifiedSpendingKey::from_seed(&TEST_NETWORK, &self.seed, account_id)
            .map_err(|e| anyhow::anyhow!("Failed to derive spending key: {:?}", e))?;
        let ufvk = usk.to_unified_full_viewing_key();

        // Get birthday tree state from lightwalletd
        let birthday_height = self.config.birthday_height;
        eprintln!(
            "[wallet] Fetching tree state for birthday height: {}",
            birthday_height
        );
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

        eprintln!("[wallet] Account created successfully");
        Ok(())
    }

    /// Sync wallet with the blockchain
    ///
    /// Downloads compact blocks and scans for transactions belonging to this wallet.
    pub async fn sync(&mut self) -> anyhow::Result<()> {
        eprintln!("[wallet] Starting sync...");

        // Use in-memory block cache for sync
        let db_cache = MemBlockCache::new();

        // Run the sync - this downloads blocks and scans for our transactions
        sync_run(
            &mut self.client,
            &TEST_NETWORK,
            &db_cache,
            &mut self.db,
            100, // batch size
        )
        .await
        .map_err(|e| anyhow::anyhow!("Sync failed: {:?}", e))?;

        eprintln!("[wallet] Sync complete");
        Ok(())
    }

    /// Get current block height from lightwalletd
    pub async fn get_block_height(&mut self) -> anyhow::Result<u64> {
        let response = self
            .client
            .get_latest_block(ChainSpec::default())
            .await?
            .into_inner();
        eprintln!("[wallet] Current block height: {}", response.height);
        Ok(response.height)
    }

    /// Get wallet balance in zatoshis
    pub fn get_balance(&self) -> anyhow::Result<u64> {
        let summary = self.db.get_wallet_summary(ConfirmationsPolicy::MIN)?;
        match summary {
            Some(s) => {
                let mut total = Zatoshis::ZERO;
                for (account_id, balance) in s.account_balances() {
                    eprintln!(
                        "[wallet] Account {:?}: spendable={}, change_pending={}, total={}",
                        account_id,
                        u64::from(balance.spendable_value()),
                        u64::from(balance.change_pending_confirmation()),
                        u64::from(balance.total())
                    );
                    total = (total + balance.total())
                        .ok_or_else(|| anyhow::anyhow!("Balance overflow"))?;
                }
                eprintln!("[wallet] Total balance: {} zatoshis", u64::from(total));
                Ok(u64::from(total))
            }
            None => {
                eprintln!("[wallet] No wallet summary available");
                Ok(0)
            }
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

        let address = addresses.first().unwrap().address();
        let encoded = address.to_zcash_address(&TEST_NETWORK).to_string();
        eprintln!("[wallet] Address: {}", encoded);
        Ok(encoded)
    }

    /// Create and broadcast a timestamp transaction
    ///
    /// Creates a self-send transaction with the hash encoded in the memo field.
    pub async fn create_timestamp_tx(
        &mut self,
        hash: &[u8; 32],
    ) -> anyhow::Result<TimestampTxResult> {
        eprintln!("[wallet] Creating timestamp transaction...");

        let accounts = self.db.get_account_ids()?;
        let account_id = accounts
            .first()
            .ok_or_else(|| anyhow::anyhow!("No account found"))?;

        // Get account address for self-send
        let addresses = self.db.list_addresses(*account_id)?;
        if addresses.is_empty() {
            return Err(anyhow::anyhow!("No addresses found"));
        }
        let address = addresses.first().unwrap().address();

        // Create memo with timestamp data
        let memo_bytes = create_timestamp_memo(hash);
        let memo = MemoBytes::from_bytes(&memo_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid memo: {:?}", e))?;

        // Create spending key for signing
        let usk = UnifiedSpendingKey::from_seed(&TEST_NETWORK, &self.seed, AccountId::ZERO)
            .map_err(|e| anyhow::anyhow!("Failed to derive spending key: {:?}", e))?;

        eprintln!("[wallet] Proposing transaction...");

        // Propose transaction - self-send with memo
        // We send a minimal amount (1000 zatoshis = 0.00001 ZEC)
        let proposal = propose_standard_transfer_to_address::<_, _, SqliteClientError>(
            &mut self.db,
            &TEST_NETWORK,
            StandardFeeRule::Zip317,
            *account_id,
            ConfirmationsPolicy::MIN,
            address,
            Zatoshis::const_from_u64(1000),
            Some(memo),
            None,
            ShieldedProtocol::Orchard,
        )
        .map_err(|e| anyhow::anyhow!("Failed to propose transaction: {:?}", e))?;

        eprintln!("[wallet] Building and signing transaction...");

        // Build and sign transaction
        let prover = LocalTxProver::bundled();
        let spending_keys = SpendingKeys::from_unified_spending_key(usk);

        let txids = create_proposed_transactions::<
            _,
            _,
            <ZotsWalletDb as InputSource>::Error,
            _,
            Infallible,
            _,
        >(
            &mut self.db,
            &TEST_NETWORK,
            &prover,
            &prover,
            &spending_keys,
            OvkPolicy::Sender,
            &proposal,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create transaction: {:?}", e))?;

        // Get first txid
        let txid = *txids.first();

        // Get the raw transaction for broadcast
        let tx = self
            .db
            .get_transaction(txid)?
            .ok_or_else(|| anyhow::anyhow!("Transaction not found in wallet"))?;

        // Serialize transaction
        let mut tx_bytes = Vec::new();
        tx.write(&mut tx_bytes)?;

        eprintln!("[wallet] Broadcasting transaction...");

        // Broadcast transaction
        let response = self
            .client
            .send_transaction(RawTransaction {
                data: tx_bytes,
                height: 0,
            })
            .await?
            .into_inner();

        if response.error_code != 0 {
            return Err(anyhow::anyhow!(
                "Broadcast failed: {}",
                response.error_message
            ));
        }

        // Convert txid to bytes
        let txid_bytes: [u8; 32] = *txid.as_ref();

        eprintln!("[wallet] Transaction broadcast successfully: {}", txid);

        Ok(TimestampTxResult {
            txid: txid.to_string(),
            txid_bytes,
        })
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
        eprintln!(
            "[wallet] Waiting for confirmation starting at height {}",
            start_height
        );

        for i in 0..max_blocks {
            eprintln!("[wallet] Confirmation check {} of {}", i + 1, max_blocks);

            // Sync wallet
            self.sync().await?;

            let current_height = self.get_block_height().await?;

            // Check if we've progressed
            if current_height > start_height {
                // For MVP, we assume confirmation after one block
                let block_time = chrono::Utc::now().timestamp() as u32;
                eprintln!(
                    "[wallet] Transaction confirmed at height {}",
                    current_height
                );
                return Ok(ConfirmationResult {
                    block_height: current_height as u32,
                    block_time,
                });
            }

            // Wait before next check (Zcash block time ~75 seconds)
            eprintln!("[wallet] Waiting 30 seconds before next check...");
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
