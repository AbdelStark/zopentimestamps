//! Zcash wallet core functionality
//!
//! Provides wallet initialization, sync, and transaction operations.

use bip0039::{English, Mnemonic};
use rand_core::OsRng;
use tonic::transport::{Channel, ClientTlsConfig};
use tracing::{debug, info};
use zcash_client_backend::data_api::wallet::{
    ConfirmationsPolicy, SpendingKeys, create_proposed_transactions,
    propose_standard_transfer_to_address,
};
use zcash_client_backend::data_api::{AccountBirthday, AccountPurpose, WalletRead, WalletWrite};
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

use crate::wallet::ZcashConfig;

const SYNC_BATCH_SIZE: u32 = 1000;

/// Result of sending a transaction
pub struct SendResult {
    /// Transaction ID as string
    pub txid: String,
    /// Fee paid in zatoshis
    pub fee: u64,
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

/// Transaction record for display
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    /// Transaction ID as hex string
    pub txid: String,
    /// Amount in zatoshis (negative for sent)
    pub amount: i64,
    /// Unix timestamp (approximate from block height)
    pub timestamp: u64,
    /// Whether this is a sent transaction
    pub is_sent: bool,
    /// Memo text if available
    pub memo: Option<String>,
}

type IkkiWalletDb =
    WalletDb<rusqlite::Connection, zcash_protocol::consensus::TestNetwork, SystemClock, OsRng>;

/// Helper to build and sign transaction with proper type annotations
fn build_and_sign_transaction(
    db: &mut IkkiWalletDb,
    params: &zcash_protocol::consensus::TestNetwork,
    prover: &LocalTxProver,
    spending_keys: &SpendingKeys,
    proposal: &zcash_client_backend::proposal::Proposal<
        StandardFeeRule,
        zcash_client_sqlite::ReceivedNoteId,
    >,
) -> anyhow::Result<::nonempty::NonEmpty<zcash_protocol::TxId>> {
    create_proposed_transactions::<
        IkkiWalletDb,
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

/// Zcash wallet for Ikki
pub struct IkkiWallet {
    config: ZcashConfig,
    db: IkkiWalletDb,
    client: CompactTxStreamerClient<Channel>,
    seed: [u8; 64],
}

impl IkkiWallet {
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
            "ikki-wallet",
            &ufvk,
            &birthday,
            AccountPurpose::Spending { derivation: None },
            None,
        )?;

        Ok(())
    }

    /// Sync wallet with the blockchain
    pub async fn sync(&mut self) -> anyhow::Result<()> {
        let db_cache = MemBlockCache::new();
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

    /// Generate a new diversified receiving address
    pub fn get_new_address(&mut self) -> anyhow::Result<String> {
        use zcash_keys::keys::{ReceiverRequirement, UnifiedAddressRequest};
        use zcash_protocol::consensus::NetworkType;

        let accounts = self.db.get_account_ids()?;
        let account_id = accounts
            .first()
            .ok_or_else(|| anyhow::anyhow!("No account found - run init_account first"))?;

        let request = UnifiedAddressRequest::unsafe_custom(
            ReceiverRequirement::Require,
            ReceiverRequirement::Require,
            ReceiverRequirement::Omit,
        );
        let (address, _diversifier_index) = self
            .db
            .get_next_available_address(*account_id, request)?
            .ok_or_else(|| anyhow::anyhow!("Failed to generate new address"))?;

        Ok(address.to_zcash_address(NetworkType::Test).to_string())
    }

    /// Get all addresses for the wallet
    pub fn get_all_addresses(&self) -> anyhow::Result<Vec<String>> {
        let accounts = self.db.get_account_ids()?;
        let account_id = accounts
            .first()
            .ok_or_else(|| anyhow::anyhow!("No account found - run init_account first"))?;

        let addresses = self.db.list_addresses(*account_id)?;

        Ok(addresses
            .iter()
            .map(|addr| addr.address().to_zcash_address(&TEST_NETWORK).to_string())
            .collect())
    }

    /// Send ZEC to an address
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
        let (orchard_balance, sapling_balance) = match &summary {
            Some(s) => {
                let mut orchard = 0u64;
                let mut sapling = 0u64;
                for balance in s.account_balances().values() {
                    orchard += u64::from(balance.orchard_balance().spendable_value());
                    sapling += u64::from(balance.sapling_balance().spendable_value());
                }
                (orchard, sapling)
            }
            None => (0, 0),
        };

        let total_shielded = orchard_balance + sapling_balance;
        let min_required = amount_zatoshi + 20000;
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

        let fee = 10000u64;

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

    /// Get recent transactions from the wallet
    pub fn get_recent_transactions(&self, limit: usize) -> anyhow::Result<Vec<TransactionRecord>> {
        use rusqlite::Connection;

        let db_path = self.config.wallet_db_path();

        let conn =
            Connection::open_with_flags(&db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        let mut stmt = conn.prepare(
            "SELECT
                txid,
                mined_height,
                account_balance_delta,
                block_time,
                sent_note_count,
                is_shielding
            FROM v_transactions
            WHERE mined_height IS NOT NULL
            ORDER BY mined_height DESC
            LIMIT ?",
        )?;

        let rows = stmt.query_map([limit as i64], |row| {
            let txid_bytes: Vec<u8> = row.get(0)?;
            let _mined_height: Option<u32> = row.get(1)?;
            let balance_delta: i64 = row.get(2)?;
            let block_time: Option<u32> = row.get(3)?;
            let sent_note_count: i64 = row.get(4)?;
            let is_shielding: bool = row.get(5)?;
            Ok((
                txid_bytes,
                balance_delta,
                block_time,
                sent_note_count,
                is_shielding,
            ))
        })?;

        let mut transactions = Vec::new();
        for (txid_bytes, balance_delta, block_time, sent_note_count, is_shielding) in rows.flatten()
        {
            let mut txid_arr = [0u8; 32];
            if txid_bytes.len() == 32 {
                txid_arr.copy_from_slice(&txid_bytes);
                txid_arr.reverse();
            }
            let txid: String = txid_arr.iter().map(|b| format!("{b:02x}")).collect();

            let timestamp = block_time.map(|t| t as u64).unwrap_or(0);
            let is_sent = sent_note_count > 0 && !is_shielding;

            transactions.push(TransactionRecord {
                txid,
                amount: balance_delta,
                timestamp,
                is_sent,
                memo: None,
            });
        }

        Ok(transactions)
    }
}
