//! Zcash wallet management for timestamping
//!
//! Provides wallet initialization, sync, and transaction creation
//! for timestamping operations on the Zcash blockchain.

use std::collections::HashMap;

use bip0039::{English, Mnemonic};
use rand_core::OsRng;
use tonic::transport::{Channel, ClientTlsConfig};
use tracing::{debug, info, warn};
use zcash_client_backend::data_api::wallet::input_selection::GreedyInputSelector;
use zcash_client_backend::data_api::wallet::{
    ConfirmationsPolicy, SpendingKeys, create_proposed_transactions, propose_shielding,
    propose_standard_transfer_to_address,
};
use zcash_client_backend::data_api::{AccountBirthday, AccountPurpose, WalletRead, WalletWrite};
use zcash_client_backend::decrypt_transaction;
use zcash_client_backend::fees::standard::SingleOutputChangeStrategy;
use zcash_client_backend::fees::{DustOutputPolicy, StandardFeeRule};
use zcash_client_backend::keys::UnifiedSpendingKey;
use zcash_client_backend::proto::service::{
    self, ChainSpec, RawTransaction, TxFilter, compact_tx_streamer_client::CompactTxStreamerClient,
};
use zcash_client_backend::sync::run as sync_run;
use zcash_client_backend::wallet::OvkPolicy;
use zcash_client_memory::MemBlockCache;
use zcash_client_sqlite::WalletDb;
use zcash_client_sqlite::error::SqliteClientError;
use zcash_client_sqlite::util::SystemClock;
use zcash_client_sqlite::wallet::init::init_wallet_db;
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_primitives::transaction::Transaction;
use zcash_proofs::prover::LocalTxProver;
use zcash_protocol::ShieldedProtocol;
use zcash_protocol::consensus::{BlockHeight, BranchId, TEST_NETWORK};
use zcash_protocol::memo::MemoBytes;
use zcash_protocol::value::Zatoshis;
use zip32::AccountId;

use crate::config::ZcashConfig;
use crate::memo::{create_timestamp_memo, parse_timestamp_memo};

const SYNC_BATCH_SIZE: u32 = 1000;

/// Result of creating a timestamp transaction
pub struct TimestampTxResult {
    /// Transaction ID as string
    pub txid: String,
    /// Transaction ID as bytes (internal byte order)
    pub txid_bytes: [u8; 32],
}

/// Result of sending a transaction
pub struct SendResult {
    /// Transaction ID as string
    pub txid: String,
    /// Fee paid in zatoshis
    pub fee: u64,
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

/// Result of verifying a timestamp transaction
pub struct VerificationResult {
    /// Whether the verification was successful
    pub valid: bool,
    /// The hash found in the memo (if any)
    pub memo_hash: Option<[u8; 32]>,
    /// Error message if verification failed
    pub error: Option<String>,
}

type ZotsWalletDb =
    WalletDb<rusqlite::Connection, zcash_protocol::consensus::TestNetwork, SystemClock, OsRng>;

/// Helper to build and sign transaction with proper type annotations
fn build_and_sign_transaction(
    db: &mut ZotsWalletDb,
    params: &zcash_protocol::consensus::TestNetwork,
    prover: &LocalTxProver,
    spending_keys: &SpendingKeys,
    proposal: &zcash_client_backend::proposal::Proposal<
        StandardFeeRule,
        zcash_client_sqlite::ReceivedNoteId,
    >,
) -> anyhow::Result<::nonempty::NonEmpty<zcash_protocol::TxId>> {
    create_proposed_transactions::<
        ZotsWalletDb,
        zcash_protocol::consensus::TestNetwork,
        zcash_client_backend::data_api::wallet::input_selection::GreedyInputSelectorError,
        StandardFeeRule,
        zcash_client_backend::fees::ChangeError<
            SqliteClientError,
            zcash_client_sqlite::ReceivedNoteId,
        >,
        zcash_client_sqlite::ReceivedNoteId,
    >(
        db,
        params,
        prover,
        prover,
        spending_keys,
        OvkPolicy::Sender,
        proposal,
    )
    .map_err(|e| anyhow::anyhow!("Failed to create transaction: {e:?}"))
}

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
            .map_err(|e| anyhow::anyhow!("Invalid seed phrase: {e:?}"))?;
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
            .map_err(|e| anyhow::anyhow!("Failed to derive spending key: {e:?}"))?;
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
            "To reset the wallet, delete the database file at: {db_path:?}\n\
            Then set ZOTS_BIRTHDAY_HEIGHT to an earlier block and restart."
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
        .map_err(|e| anyhow::anyhow!("Sync failed: {e:?}"))?;

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
                for balance in s.account_balances().values() {
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
                for balance in s.account_balances().values() {
                    breakdown.transparent +=
                        u64::from(balance.unshielded_balance().spendable_value());
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
        .map_err(|e| anyhow::anyhow!("Failed to propose shielding: {e:?}"))?;

        // TODO: Implement full transaction building with zk-SNARK provers
        Err(anyhow::anyhow!(
            "Shielding requires zk-SNARK provers (not yet integrated).\n\
            Please use Zingo or zcashd to shield your funds."
        ))
    }

    /// Create and broadcast a timestamp transaction
    ///
    /// Creates a shielded transaction with the file hash in the memo field,
    /// then broadcasts it to the Zcash network.
    pub async fn create_timestamp_tx(
        &mut self,
        hash: &[u8; 32],
    ) -> anyhow::Result<TimestampTxResult> {
        let accounts = self.db.get_account_ids()?;
        let account_id = accounts
            .first()
            .ok_or_else(|| anyhow::anyhow!("No account found"))?;
        info!(
            "Creating timestamp transaction for account {:?}",
            account_id
        );

        // Check balance
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
        debug!(
            transparent_balance,
            orchard_balance, sapling_balance, total_shielded, "Wallet balance snapshot (zatoshis)"
        );

        // Need shielded funds to send memo
        if total_shielded < min_required {
            warn!("Insufficient shielded funds for timestamp transaction");
            if transparent_balance >= min_required {
                return Err(anyhow::anyhow!(
                    "Your funds are in the transparent pool.\n\
                    Shield them first using Zingo or zcashd, then try again."
                ));
            }
            return Err(anyhow::anyhow!(
                "Insufficient funds. Run 'zots wallet address' and fund your wallet."
            ));
        }

        // Get the wallet's own address to send to self
        let addresses = self.db.list_addresses(*account_id)?;
        let address = addresses
            .first()
            .ok_or_else(|| anyhow::anyhow!("No address found"))?
            .address()
            .clone();
        debug!("Using internal address {:?} for self-send", address);

        // Create memo with timestamp data
        debug!("Creating timestamp memo payload");
        let memo_data = create_timestamp_memo(hash);
        let memo = MemoBytes::from_bytes(&memo_data)
            .map_err(|_| anyhow::anyhow!("Failed to create memo"))?;

        // Derive spending key
        debug!("Deriving unified spending key for transaction");
        let usk = UnifiedSpendingKey::from_seed(&TEST_NETWORK, &self.seed, AccountId::ZERO)
            .map_err(|e| anyhow::anyhow!("Failed to derive spending key: {e:?}"))?;

        // Create proposal for self-send with memo
        // Send dust amount (just to carry the memo)
        let dust_amount = Zatoshis::from_u64(10000).unwrap(); // 0.0001 ZEC

        let proposal = propose_standard_transfer_to_address::<_, _, SqliteClientError>(
            &mut self.db,
            &TEST_NETWORK,
            StandardFeeRule::Zip317,
            *account_id,
            ConfirmationsPolicy::MIN,
            &address,
            dust_amount,
            Some(memo),
            None, // no change memo
            ShieldedProtocol::Orchard,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create transaction proposal: {e:?}"))?;
        debug!("Proposal created for self-send with memo");

        // Load bundled Sapling prover (includes proving parameters)
        let prover = LocalTxProver::bundled();
        let spending_keys = SpendingKeys::from_unified_spending_key(usk);
        debug!("Loaded proving parameters and spending keys");

        // Build the transaction using helper to handle complex type inference
        let txids = build_and_sign_transaction(
            &mut self.db,
            &TEST_NETWORK,
            &prover,
            &spending_keys,
            &proposal,
        )?;
        debug!("Transaction built and signed");

        // NonEmpty guarantees at least one element
        let txid = *txids.first();
        info!("Timestamp transaction built with txid {}", txid);

        // Get the transaction from the database
        let tx = self
            .db
            .get_transaction(txid)?
            .ok_or_else(|| anyhow::anyhow!("Transaction not found in database"))?;

        // Serialize the transaction to bytes
        let mut tx_bytes = Vec::new();
        tx.write(&mut tx_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to serialize transaction: {e:?}"))?;

        // Broadcast transaction
        let raw_tx = RawTransaction {
            data: tx_bytes,
            height: 0, // Will be set by lightwalletd
        };

        let response = self
            .client
            .send_transaction(raw_tx)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to broadcast transaction: {e:?}"))?;

        let send_response = response.into_inner();
        // error_code 0 means success, error_message may contain txid on success
        if send_response.error_code != 0 {
            return Err(anyhow::anyhow!(
                "Transaction rejected (code {}): {}",
                send_response.error_code,
                send_response.error_message
            ));
        }
        debug!(
            "Broadcast response accepted (code {}): {}",
            send_response.error_code, send_response.error_message
        );

        // Return the transaction ID (use Display formatting which reverses bytes for user display)
        let txid_bytes: [u8; 32] = txid.into();
        Ok(TimestampTxResult {
            txid: txid.to_string(), // Uses the Display impl which gives explorer-friendly format
            txid_bytes,
        })
    }

    /// Send ZEC to an address
    ///
    /// Creates and broadcasts a shielded transaction to the specified address.
    /// Optionally includes a memo.
    pub async fn send_to_address(
        &mut self,
        to_address: &str,
        amount_zatoshi: u64,
        memo: Option<Vec<u8>>,
    ) -> anyhow::Result<SendResult> {
        let accounts = self.db.get_account_ids()?;
        let account_id = accounts
            .first()
            .ok_or_else(|| anyhow::anyhow!("No account found"))?;
        info!(
            "Sending {} zatoshis to {} from account {:?}",
            amount_zatoshi, to_address, account_id
        );

        // Check balance
        let summary = self.db.get_wallet_summary(ConfirmationsPolicy::MIN)?;
        let (orchard_balance, sapling_balance) = summary
            .as_ref()
            .and_then(|s| s.account_balances().get(account_id))
            .map(|b| {
                (
                    u64::from(b.orchard_balance().spendable_value()),
                    u64::from(b.sapling_balance().spendable_value()),
                )
            })
            .unwrap_or((0, 0));

        let total_shielded = orchard_balance + sapling_balance;
        let min_required = amount_zatoshi + 20000; // amount + ZIP-317 fee estimate
        debug!(
            orchard_balance,
            sapling_balance, total_shielded, amount_zatoshi, "Balance check for send"
        );

        if total_shielded < min_required {
            return Err(anyhow::anyhow!(
                "Insufficient shielded funds. Need {min_required} zatoshis, have {total_shielded} zatoshis"
            ));
        }

        // Parse the destination address
        use zcash_address::ZcashAddress;
        let parsed_address: ZcashAddress = to_address
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid address: {e:?}"))?;
        let address = parsed_address
            .convert::<zcash_keys::address::Address>()
            .map_err(|e| anyhow::anyhow!("Address conversion failed: {e:?}"))?;

        // Create memo if provided
        let memo_bytes = if let Some(data) = memo {
            MemoBytes::from_bytes(&data).map_err(|_| anyhow::anyhow!("Invalid memo"))?
        } else {
            MemoBytes::empty()
        };

        // Derive spending key
        debug!("Deriving unified spending key for transaction");
        let usk = UnifiedSpendingKey::from_seed(&TEST_NETWORK, &self.seed, AccountId::ZERO)
            .map_err(|e| anyhow::anyhow!("Failed to derive spending key: {e:?}"))?;

        // Create proposal
        let send_amount =
            Zatoshis::from_u64(amount_zatoshi).map_err(|_| anyhow::anyhow!("Invalid amount"))?;

        let proposal = propose_standard_transfer_to_address::<_, _, SqliteClientError>(
            &mut self.db,
            &TEST_NETWORK,
            StandardFeeRule::Zip317,
            *account_id,
            ConfirmationsPolicy::MIN,
            &address,
            send_amount,
            Some(memo_bytes),
            None,
            ShieldedProtocol::Orchard,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create transaction proposal: {e:?}"))?;

        // Estimate fee (ZIP-317 standard fee)
        let fee = 10000u64; // 0.0001 ZEC - standard minimum fee

        // Load prover and build transaction
        let prover = LocalTxProver::bundled();
        let spending_keys = SpendingKeys::from_unified_spending_key(usk);
        debug!("Building and signing transaction");

        let txids = build_and_sign_transaction(
            &mut self.db,
            &TEST_NETWORK,
            &prover,
            &spending_keys,
            &proposal,
        )?;

        let txid = *txids.first();
        info!("Transaction built with txid {}", txid);

        // Get the transaction and broadcast
        let tx = self
            .db
            .get_transaction(txid)?
            .ok_or_else(|| anyhow::anyhow!("Transaction not found in database"))?;

        let mut tx_bytes = Vec::new();
        tx.write(&mut tx_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to serialize transaction: {e:?}"))?;

        let raw_tx = RawTransaction {
            data: tx_bytes,
            height: 0,
        };

        let response = self
            .client
            .send_transaction(raw_tx)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to broadcast transaction: {e:?}"))?;

        let send_response = response.into_inner();
        if send_response.error_code != 0 {
            return Err(anyhow::anyhow!(
                "Transaction rejected (code {}): {}",
                send_response.error_code,
                send_response.error_message
            ));
        }

        info!("Transaction {} broadcast successfully", txid);
        Ok(SendResult {
            txid: txid.to_string(),
            fee,
        })
    }

    /// Wait for transaction confirmation.
    ///
    /// Polls the lightwalletd tip until the height advances, then returns the
    /// observed height plus a wall-clock timestamp for UX purposes. This is a
    /// lightweight heuristic rather than a consensus-proof of inclusion.
    pub async fn wait_confirmation(
        &mut self,
        txid: &str,
        max_blocks: u32,
    ) -> anyhow::Result<ConfirmationResult> {
        let start_height = self.get_block_height().await?;
        info!(
            "Waiting for confirmation of txid {} starting at height {}",
            txid, start_height
        );

        for _ in 0..max_blocks {
            self.sync().await?;
            let current_height = self.get_block_height().await?;
            debug!(current_height, start_height, "Synced height while waiting");

            if current_height > start_height {
                // Use wall-clock time here; lightwalletd does not return block metadata
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
            "Transaction {txid} not confirmed within {max_blocks} blocks"
        ))
    }

    /// Get the wallet configuration
    pub fn config(&self) -> &ZcashConfig {
        &self.config
    }

    /// Verify a timestamp transaction by fetching it from the blockchain
    /// and checking that the memo contains the expected hash.
    ///
    /// The memo is decrypted with the wallet's viewing keys, so callers must
    /// use the same seed (or an exported viewing key) that was used to create
    /// the timestamp transaction. This provides cryptographic verification
    /// that the hash was committed to the Zcash blockchain in the specified
    /// transaction.
    pub async fn verify_timestamp_tx(
        &mut self,
        txid_bytes: &[u8; 32],
        expected_hash: &[u8; 32],
        block_height: Option<u32>,
    ) -> anyhow::Result<VerificationResult> {
        info!("Verifying timestamp transaction");
        debug!(
            block_height,
            "Fetching transaction with expected memo hash ({} bytes)",
            expected_hash.len()
        );

        // Fetch transaction from lightwalletd
        // Note: lightwalletd expects txid in internal byte order (not display order)
        let tx_filter = TxFilter {
            block: None,
            index: 0,
            hash: txid_bytes.to_vec(),
        };

        let response = self
            .client
            .get_transaction(tx_filter)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch transaction: {e:?}"))?;

        let raw_tx = response.into_inner();
        debug!("Fetched raw transaction bytes: {}", raw_tx.data.len());
        if raw_tx.data.is_empty() {
            return Ok(VerificationResult {
                valid: false,
                memo_hash: None,
                error: Some("Transaction not found on blockchain".to_string()),
            });
        }

        // Parse the raw transaction
        let tx = Transaction::read(&raw_tx.data[..], BranchId::Nu6)
            .map_err(|e| anyhow::anyhow!("Failed to parse transaction: {e:?}"))?;
        debug!("Transaction parsed; scanning outputs for memo");

        // Get the viewing key for decryption
        let usk = UnifiedSpendingKey::from_seed(&TEST_NETWORK, &self.seed, AccountId::ZERO)
            .map_err(|e| anyhow::anyhow!("Failed to derive spending key: {e:?}"))?;
        let ufvk = usk.to_unified_full_viewing_key();

        // Create a map of viewing keys for decrypt_transaction
        let mut ufvks: HashMap<u32, UnifiedFullViewingKey> = HashMap::new();
        ufvks.insert(0, ufvk);

        // Get block height for decryption context
        let mined_height = block_height.map(BlockHeight::from_u32);
        let chain_tip = self
            .get_block_height()
            .await
            .ok()
            .map(|h| BlockHeight::from_u32(h as u32));

        // Decrypt the transaction to extract memos
        let decrypted = decrypt_transaction(&TEST_NETWORK, mined_height, chain_tip, &tx, &ufvks);

        // Check all decrypted outputs for matching memo
        for output in decrypted.sapling_outputs() {
            if let Some(hash) = parse_timestamp_memo(output.memo().as_slice())
                && hash == *expected_hash
            {
                info!("Found matching memo in Sapling output");
                return Ok(VerificationResult {
                    valid: true,
                    memo_hash: Some(hash),
                    error: None,
                });
            }
        }

        // Check Orchard outputs
        for output in decrypted.orchard_outputs() {
            if let Some(hash) = parse_timestamp_memo(output.memo().as_slice())
                && hash == *expected_hash
            {
                info!("Found matching memo in Orchard output");
                return Ok(VerificationResult {
                    valid: true,
                    memo_hash: Some(hash),
                    error: None,
                });
            }
        }

        // If we have decrypted outputs but none match, verification failed
        let total_outputs = decrypted.sapling_outputs().len() + decrypted.orchard_outputs().len();
        debug!(total_outputs, "No matching memo found in decrypted outputs");

        if total_outputs > 0 {
            Ok(VerificationResult {
                valid: false,
                memo_hash: None,
                error: Some("Transaction found but memo hash does not match".to_string()),
            })
        } else {
            // No outputs could be decrypted - might be from a different wallet
            Ok(VerificationResult {
                valid: false,
                memo_hash: None,
                error: Some(
                    "Could not decrypt transaction outputs. \
                    This may be a transaction from a different wallet."
                        .to_string(),
                ),
            })
        }
    }
}
