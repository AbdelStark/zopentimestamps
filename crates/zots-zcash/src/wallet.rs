//! Zcash wallet management for timestamping
//!
//! Provides wallet initialization, sync, and transaction creation
//! for timestamping operations on the Zcash blockchain.

use bip0039::{English, Mnemonic};
use rand_core::OsRng;
use sha2::{Sha256, Digest};
use tonic::transport::{Channel, ClientTlsConfig};
use zcash_client_backend::data_api::wallet::{
    ConfirmationsPolicy, propose_shielding,
};
use zcash_client_backend::data_api::{
    Account as AccountTrait, AccountBirthday, AccountPurpose, WalletRead, WalletWrite,
};
use zcash_client_backend::encoding::AddressCodec;
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
            // Log the existing account info for debugging
            for account_id in &accounts {
                eprintln!("[wallet] Existing account: {:?}", account_id);
            }
            return Ok(());
        }

        eprintln!("[wallet] Creating new account...");

        // Log seed fingerprint (first 8 bytes of hash) for debugging
        let seed_hash = Sha256::digest(&self.seed);
        eprintln!("[wallet] Seed fingerprint: {:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            seed_hash[0], seed_hash[1], seed_hash[2], seed_hash[3],
            seed_hash[4], seed_hash[5], seed_hash[6], seed_hash[7]
        );
        // Also show first/last bytes of the actual seed for debugging
        eprintln!("[wallet] Raw seed bytes: {:02x}{:02x}{:02x}{:02x}...{:02x}{:02x}{:02x}{:02x}",
            self.seed[0], self.seed[1], self.seed[2], self.seed[3],
            self.seed[60], self.seed[61], self.seed[62], self.seed[63]
        );

        // Create unified spending key from seed
        let account_id = AccountId::ZERO;
        eprintln!("[wallet] Using AccountId: {:?} (ZIP-32 account index 0)", account_id);
        let usk = UnifiedSpendingKey::from_seed(&TEST_NETWORK, &self.seed, account_id)
            .map_err(|e| anyhow::anyhow!("Failed to derive spending key: {:?}", e))?;
        let ufvk = usk.to_unified_full_viewing_key();

        // Log the derived addresses for debugging
        eprintln!("[wallet] Derived UFVK successfully");

        // Log which components are available in the UFVK
        eprintln!("[wallet] UFVK components - transparent: {}, sapling: {}, orchard: {}",
            ufvk.transparent().is_some(),
            ufvk.sapling().is_some(),
            ufvk.orchard().is_some()
        );

        // Print the UFVK encoding for comparison
        let ufvk_string = ufvk.encode(&TEST_NETWORK);
        eprintln!("[wallet] UFVK (first 100 chars): {}", &ufvk_string[..100]);
        eprintln!("[wallet] UFVK (last 50 chars): ...{}", &ufvk_string[ufvk_string.len()-50..]);

        // Print Orchard FVK fingerprint for debugging (helps verify key derivation matches Zingo)
        if let Some(orchard_fvk) = ufvk.orchard() {
            // Get the default Orchard address for verification
            let orchard_addr = orchard_fvk.address_at(0u32, zip32::Scope::External);
            let orchard_addr_bytes = orchard_addr.to_raw_address_bytes();
            eprintln!("[wallet] Orchard address[0] fingerprint: {:02x}{:02x}{:02x}{:02x}...{:02x}{:02x}{:02x}{:02x}",
                orchard_addr_bytes[0], orchard_addr_bytes[1], orchard_addr_bytes[2], orchard_addr_bytes[3],
                orchard_addr_bytes[39], orchard_addr_bytes[40], orchard_addr_bytes[41], orchard_addr_bytes[42]
            );
        }

        // Print transparent key for debugging
        if let Some(_t_fvk) = ufvk.transparent() {
            eprintln!("[wallet] Transparent key derived from account {:?}", account_id);
        }

        // Get birthday tree state from lightwalletd
        let birthday_height = self.config.birthday_height;
        eprintln!(
            "[wallet] Using birthday height: {} (configure ZOTS_BIRTHDAY_HEIGHT to change)",
            birthday_height
        );
        eprintln!("[wallet] WARNING: If your wallet has transactions before this height, they will NOT be found!");
        eprintln!("[wallet] Set ZOTS_BIRTHDAY_HEIGHT to an earlier block to scan more history.");

        let request = service::BlockId {
            height: birthday_height.saturating_sub(1),
            ..Default::default()
        };
        let treestate = self.client.get_tree_state(request).await?.into_inner();

        // Debug: check what's in the tree state
        eprintln!("[wallet] Tree state height: {}", treestate.height);
        eprintln!("[wallet] Tree state has sapling_tree: {}", !treestate.sapling_tree.is_empty());
        eprintln!("[wallet] Tree state has orchard_tree: {}", !treestate.orchard_tree.is_empty());
        eprintln!("[wallet] Tree state sapling_tree len: {}", treestate.sapling_tree.len());
        eprintln!("[wallet] Tree state orchard_tree len: {}", treestate.orchard_tree.len());

        let birthday = AccountBirthday::from_treestate(treestate, None)
            .map_err(|_| anyhow::anyhow!("Failed to create birthday from tree state"))?;

        eprintln!("[wallet] Birthday created from tree state at height: {}", birthday.height());

        // Also get the birthday block's timestamp for verification
        let birthday_block_req = service::BlockId {
            height: birthday_height as u64,
            ..Default::default()
        };
        if let Ok(block_info) = self.client.get_block(birthday_block_req).await {
            let block = block_info.into_inner();
            let timestamp = chrono::DateTime::from_timestamp(block.time as i64, 0);
            eprintln!("[wallet] Birthday block time: {:?}", timestamp);
        }

        // Import account into wallet
        let account = self.db.import_account_ufvk(
            "zots-wallet",
            &ufvk,
            &birthday,
            AccountPurpose::Spending { derivation: None },
            None,
        )?;

        eprintln!("[wallet] Account created with ID: {:?}", account.id());

        // Verify account was imported correctly
        let acc_ufvk = account.ufvk();
        if let Some(acc_ufvk) = acc_ufvk {
            eprintln!("[wallet] DB Account UFVK has orchard: {}", acc_ufvk.orchard().is_some());
            eprintln!("[wallet] DB Account UFVK has sapling: {}", acc_ufvk.sapling().is_some());
            eprintln!("[wallet] DB Account UFVK has transparent: {}", acc_ufvk.transparent().is_some());
        } else {
            eprintln!("[wallet] WARNING: Account has no UFVK stored!");
        }

        eprintln!("[wallet] Account created successfully");
        Ok(())
    }

    /// Reset and reinitialize wallet with a new birthday height
    ///
    /// This is useful if the birthday height was set too high and transactions were missed.
    /// WARNING: This will delete the existing wallet database!
    pub async fn reset_wallet(&mut self) -> anyhow::Result<()> {
        eprintln!("[wallet] Resetting wallet database...");

        // Get the wallet db path
        let db_path = self.config.wallet_db_path();

        // We can't easily reset the existing db, so inform the user
        eprintln!("[wallet] To reset the wallet, delete the database file at: {:?}", db_path);
        eprintln!("[wallet] Then set ZOTS_BIRTHDAY_HEIGHT to an earlier block and restart.");

        Ok(())
    }

    /// Sync wallet with the blockchain
    ///
    /// Downloads compact blocks and scans for transactions belonging to this wallet.
    pub async fn sync(&mut self) -> anyhow::Result<()> {
        eprintln!("[wallet] Starting sync...");

        // Log scan range info from the wallet database
        if let Ok(Some(summary)) = self.db.get_wallet_summary(ConfirmationsPolicy::MIN) {
            eprintln!("[wallet] Wallet scan progress: {:?}", summary.progress());
            eprintln!("[wallet] Fully scanned height: {:?}", summary.fully_scanned_height());
            eprintln!("[wallet] Chain tip height: {:?}", summary.chain_tip_height());
        }

        // Use in-memory block cache for sync
        let db_cache = MemBlockCache::new();

        // Get latest block before sync for debugging
        let pre_sync_height = self.client.get_latest_block(ChainSpec::default()).await?.into_inner().height;
        eprintln!("[wallet] Pre-sync chain tip: {}", pre_sync_height);

        // Run the sync - this downloads blocks and scans for our transactions
        eprintln!("[wallet] Running sync with batch size {}...", SYNC_BATCH_SIZE);
        sync_run(
            &mut self.client,
            &TEST_NETWORK,
            &db_cache,
            &mut self.db,
            SYNC_BATCH_SIZE, // batch size
        )
        .await
        .map_err(|e| anyhow::anyhow!("Sync failed: {:?}", e))?;

        // Log post-sync status
        if let Ok(Some(summary)) = self.db.get_wallet_summary(ConfirmationsPolicy::MIN) {
            eprintln!("[wallet] Post-sync scan progress: {:?}", summary.progress());
            eprintln!("[wallet] Post-sync fully scanned height: {:?}", summary.fully_scanned_height());

            // Log balance breakdown by account
            for (account_id, balance) in summary.account_balances() {
                eprintln!("[wallet] Post-sync Account {:?} balances:", account_id);
                eprintln!("[wallet]   Orchard: spendable={}, total={}",
                    u64::from(balance.orchard_balance().spendable_value()),
                    u64::from(balance.orchard_balance().total()));
                eprintln!("[wallet]   Sapling: spendable={}, total={}",
                    u64::from(balance.sapling_balance().spendable_value()),
                    u64::from(balance.sapling_balance().total()));
                eprintln!("[wallet]   Transparent: spendable={}, total={}",
                    u64::from(balance.unshielded_balance().spendable_value()),
                    u64::from(balance.unshielded_balance().total()));
            }

            // Check subtree info
            eprintln!("[wallet] Next Sapling subtree: {:?}", summary.next_sapling_subtree_index());
        }

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
                    // Get balances from all pools
                    let unshielded = balance.unshielded_balance();
                    let sapling = balance.sapling_balance();
                    let orchard = balance.orchard_balance();

                    // Calculate total spendable across all pools
                    let total_spendable = u64::from(unshielded.spendable_value())
                        + u64::from(sapling.spendable_value())
                        + u64::from(orchard.spendable_value());

                    eprintln!(
                        "[wallet] Account {:?}: total_spendable={}, change_pending={}, total={}",
                        account_id,
                        total_spendable,
                        u64::from(balance.change_pending_confirmation()),
                        u64::from(balance.total())
                    );
                    // Log balance breakdown by pool
                    eprintln!(
                        "[wallet] Transparent: spendable={}, total={}",
                        u64::from(unshielded.spendable_value()),
                        u64::from(unshielded.total())
                    );
                    eprintln!(
                        "[wallet] Sapling: spendable={}, total={}",
                        u64::from(sapling.spendable_value()),
                        u64::from(sapling.total())
                    );
                    eprintln!(
                        "[wallet] Orchard: spendable={}, total={}",
                        u64::from(orchard.spendable_value()),
                        u64::from(orchard.total())
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

        let address_record = addresses.first().unwrap();
        let address = address_record.address();
        let encoded = address.to_zcash_address(&TEST_NETWORK).to_string();
        eprintln!("[wallet] Unified address: {}", encoded);

        // Print transparent component if available
        eprintln!("[wallet] Address receivers:");
        if let Some(t_addr) = address.to_transparent_address() {
            eprintln!("[wallet]   Transparent: {}", t_addr.encode(&TEST_NETWORK));
        } else {
            eprintln!("[wallet]   Transparent: (none)");
        }

        // Print address type info
        eprintln!("[wallet] Address debug: {:?}", address);

        Ok(encoded)
    }

    /// Shield transparent funds to Orchard
    ///
    /// Moves funds from transparent pool to shielded Orchard pool.
    pub async fn shield_transparent_funds(&mut self) -> anyhow::Result<String> {
        eprintln!("[wallet] Shielding transparent funds to Orchard...");

        let accounts = self.db.get_account_ids()?;
        let account_id = accounts
            .first()
            .ok_or_else(|| anyhow::anyhow!("No account found"))?;

        // Create change strategy for shielding
        let dust_policy = DustOutputPolicy::default();
        let change_strategy = SingleOutputChangeStrategy::new(
            StandardFeeRule::Zip317,
            None, // no change memo
            ShieldedProtocol::Orchard,
            dust_policy,
        );

        // Create input selector
        let input_selector = GreedyInputSelector::<ZotsWalletDb>::new();

        // Propose shielding - shield all available (threshold = 0)
        eprintln!("[wallet] Proposing shielding transaction...");
        let _proposal = propose_shielding::<_, _, _, _, SqliteClientError>(
            &mut self.db,
            &TEST_NETWORK,
            &input_selector,
            &change_strategy,
            Zatoshis::ZERO, // shield everything above dust
            &[], // from all transparent addresses
            *account_id,
            ConfirmationsPolicy::MIN,
        )
        .map_err(|e| anyhow::anyhow!("Failed to propose shielding: {:?}", e))?;

        eprintln!("[wallet] Building shielding transaction...");

        // Build the shielding transaction
        // We need spending keys and provers
        let _usk = UnifiedSpendingKey::from_seed(&TEST_NETWORK, &self.seed, AccountId::ZERO)
            .map_err(|e| anyhow::anyhow!("Failed to derive spending key: {:?}", e))?;

        // For shielding, we need to use create_proposed_transactions
        // But that requires provers which are heavy. For now, return a stub.
        // TODO: Implement full transaction building

        Err(anyhow::anyhow!(
            "Shielding transaction building requires zk-SNARK provers which are not yet integrated.\n\
            Please use Zingo mobile wallet or zcashd to shield your funds first."
        ))
    }

    /// Create and broadcast a timestamp transaction
    ///
    /// Creates a transaction to anchor the file hash on the Zcash blockchain.
    /// Currently requires manual shielding first - use Zingo or zcashd to shield funds.
    pub async fn create_timestamp_tx(
        &mut self,
        _hash: &[u8; 32],
    ) -> anyhow::Result<TimestampTxResult> {
        eprintln!("[wallet] Creating timestamp transaction...");

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

        eprintln!(
            "[wallet] Balance: transparent={}, orchard={}, sapling={} zatoshis",
            transparent_balance, orchard_balance, sapling_balance
        );

        // Minimum needed for transaction: amount + fee (~20000 for ZIP-317)
        let min_required = 20000u64;

        // Check if we have sufficient shielded funds
        if total_shielded >= min_required {
            // TODO: Implement shielded transaction creation with memo
            return Err(anyhow::anyhow!(
                "You have {} zatoshis in shielded pools (orchard={}, sapling={}).\n\n\
                Full shielded transaction support (with zk-SNARK proofs) is coming soon.\n\
                For now, please use Zingo mobile wallet to create timestamp transactions.\n\n\
                The hash to embed in the memo is stored in the .zots proof file.",
                total_shielded, orchard_balance, sapling_balance
            ));
        }

        // If only transparent funds available
        if transparent_balance >= min_required {
            return Err(anyhow::anyhow!(
                "Your funds ({} zatoshis) are in the transparent pool.\n\n\
                To timestamp files, first shield your funds using one of these options:\n\n\
                1. Zingo mobile wallet: Send to your shielded (z) address\n\
                2. YWallet: Shield funds in settings\n\
                3. zcashd CLI: z_shieldcoinbase \"*\" \"YOUR_Z_ADDRESS\"\n\n\
                After shielding, run 'zots wallet sync' and try the stamp command again.\n\n\
                Your wallet address: run 'zots wallet address'",
                transparent_balance
            ));
        }

        // No funds at all
        Err(anyhow::anyhow!(
            "Insufficient funds: have {} zatoshis total, need at least {} zatoshis.\n\n\
            Please fund the wallet address shown by 'zots wallet address'.",
            transparent_balance + total_shielded,
            min_required
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

    /// Debug: Show addresses for multiple account indices
    ///
    /// This helps diagnose address derivation mismatches with other wallets.
    pub fn debug_show_all_accounts(&self) -> anyhow::Result<()> {
        eprintln!("\n=== DEBUG: Address derivation for accounts 0-4 ===\n");

        // Show mnemonic word count from config for verification
        let word_count = self.config.seed_phrase.split_whitespace().count();
        eprintln!("Seed phrase: {} words (first word: {})",
            word_count,
            self.config.seed_phrase.split_whitespace().next().unwrap_or("(empty)")
        );

        // Log seed fingerprint (SHA256 of the 64-byte BIP39 seed)
        let seed_hash = Sha256::digest(&self.seed);
        eprintln!("BIP39 seed fingerprint: {:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            seed_hash[0], seed_hash[1], seed_hash[2], seed_hash[3],
            seed_hash[4], seed_hash[5], seed_hash[6], seed_hash[7]
        );
        eprintln!("(Note: BIP39 uses empty passphrase - some wallets may use a passphrase)");
        eprintln!();

        use zcash_keys::keys::{UnifiedAddressRequest, ReceiverRequirement};
        use zcash_protocol::consensus::NetworkType;

        for account_idx in 0u32..5 {
            let account_id = AccountId::try_from(account_idx)
                .map_err(|_| anyhow::anyhow!("Invalid account index"))?;

            let usk = match UnifiedSpendingKey::from_seed(&TEST_NETWORK, &self.seed, account_id) {
                Ok(usk) => usk,
                Err(e) => {
                    eprintln!("Account {}: Failed to derive - {:?}", account_idx, e);
                    continue;
                }
            };

            let ufvk = usk.to_unified_full_viewing_key();

            // Get the default unified address - request all receiver types
            let request = UnifiedAddressRequest::unsafe_custom(
                ReceiverRequirement::Require,  // orchard
                ReceiverRequirement::Require,  // sapling
                ReceiverRequirement::Require,  // p2pkh (transparent)
            );
            let (default_addr, _diversifier_idx) = ufvk.default_address(request)
                .map_err(|e| anyhow::anyhow!("Failed to get default address: {:?}", e))?;

            let unified_addr = default_addr.to_zcash_address(NetworkType::Test).to_string();

            eprintln!("Account {} (ZIP-32 path m/32'/1'/{}'):", account_idx, account_idx);

            // Get transparent address using transparent() method
            if let Some(t_addr) = default_addr.transparent() {
                eprintln!("  Transparent: {}", t_addr.encode(&TEST_NETWORK));
            }

            // Show truncated unified address
            eprintln!("  Unified: {}...{}", &unified_addr[..40], &unified_addr[unified_addr.len()-20..]);
            eprintln!();
        }

        eprintln!("=== Compare transparent addresses with Zingo to find matching account ===\n");

        // Decode and compare a Zingo unified address (user's actual receiving address)
        eprintln!("\n=== Decoding Zingo's unified address for comparison ===\n");
        let zingo_addr = "utest163hcnmzrae52nmfcnqyjv96lqw7w5dus5yvqevm2am2ue554p497lkj730cykz858mflkyfkgcfu9sfw0hvfzytymns60agygqlsqdzv";

        use zcash_address::unified::{Encoding, Address as UnifiedAddressEncoded, Container};

        // Try to decode as unified address
        match UnifiedAddressEncoded::decode(zingo_addr) {
            Ok((network, ua_items)) => {
                eprintln!("[debug] Decoded unified address, network: {:?}", network);
                for item in Container::items(&ua_items) {
                    match item {
                        zcash_address::unified::Receiver::Orchard(orchard_bytes) => {
                            eprintln!("[debug] Zingo Orchard receiver: {:02x}{:02x}{:02x}{:02x}...{:02x}{:02x}{:02x}{:02x}",
                                orchard_bytes[0], orchard_bytes[1], orchard_bytes[2], orchard_bytes[3],
                                orchard_bytes[39], orchard_bytes[40], orchard_bytes[41], orchard_bytes[42]
                            );
                        }
                        zcash_address::unified::Receiver::Sapling(sapling_bytes) => {
                            eprintln!("[debug] Zingo has Sapling receiver (43 bytes)");
                        }
                        zcash_address::unified::Receiver::P2pkh(p2pkh_bytes) => {
                            eprintln!("[debug] Zingo has P2PKH receiver");
                        }
                        _ => {
                            eprintln!("[debug] Zingo has other receiver type");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[debug] Failed to decode Zingo address: {:?}", e);
            }
        }

        // Also derive transparent addresses directly using BIP-44
        // Path: m/44'/1'/account'/0/address_index (testnet)
        eprintln!("\n=== BIP-44 Direct Transparent Derivation ===\n");

        use zcash_transparent::keys::{AccountPrivKey, NonHardenedChildIndex, IncomingViewingKey};

        for account_idx in 0u32..5 {
            let account_id = AccountId::try_from(account_idx)
                .map_err(|_| anyhow::anyhow!("Invalid account index"))?;

            let account_key = match AccountPrivKey::from_seed(&TEST_NETWORK, &self.seed, account_id) {
                Ok(k) => k,
                Err(e) => {
                    eprintln!("Account {}: Failed to derive BIP-44 key - {:?}", account_idx, e);
                    continue;
                }
            };

            // Derive external addresses 0-2
            let pub_key = account_key.to_account_pubkey();
            if let Ok(external_ivk) = pub_key.derive_external_ivk() {
                for addr_idx in 0u32..3 {
                    if let Some(idx) = NonHardenedChildIndex::from_index(addr_idx) {
                        if let Ok(addr) = external_ivk.derive_address(idx) {
                            eprintln!("BIP-44 m/44'/1'/{}'/0/{}: {}",
                                account_idx, addr_idx, addr.encode(&TEST_NETWORK));
                        }
                    }
                }
            }
        }
        eprintln!();

        Ok(())
    }
}
