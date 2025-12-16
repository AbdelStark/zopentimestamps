//! Verify command implementation.
//!
//! Verifies a timestamp proof by:
//! 1. Loading the proof file
//! 2. Optionally verifying the hash matches an original file
//! 3. Fetching the transaction from the blockchain
//! 4. Decrypting the memo and verifying it contains the expected hash
//!
//! This provides cryptographic proof that the data existed at the block time.

use crate::output::*;
use std::path::PathBuf;
use zots_core::{TimestampProof, hash_file_with, hash_to_hex};
use zots_zcash::{ZcashConfig, ZotsWallet};

pub async fn run(proof_path: PathBuf, file: Option<PathBuf>) -> anyhow::Result<()> {
    print_header("Verifying Timestamp");

    // Load proof
    let proof = TimestampProof::load(&proof_path)?;
    print_info("Proof", &proof_path.display().to_string());
    print_hash(&proof.hash, proof.hash_algorithm().name());

    // Get hash bytes for comparison
    let proof_hash_bytes = proof.hash_bytes()?;
    let algorithm = proof.hash_algorithm();

    // Verify against original file if provided
    if let Some(file_path) = file {
        print_status("Verifying hash against original file...");
        let file_hash = hash_file_with(&file_path, algorithm)?;

        if file_hash == proof_hash_bytes {
            print_success("Hash matches original file");
        } else {
            print_error("Hash does NOT match original file!");
            print_info("Expected", &proof.hash);
            print_info("Got", &hash_to_hex(&file_hash));
            print_info("Algorithm", algorithm.name());
            return Ok(());
        }
    }

    // Check attestations
    if proof.attestations.is_empty() {
        print_warning("No attestations found - proof is pending confirmation");
        return Ok(());
    }

    let att = &proof.attestations[0];

    // Verify against the blockchain by fetching the transaction
    // and checking the memo contains the expected hash
    print_status("Verifying against blockchain...");

    let config = ZcashConfig::from_env()?;
    let mut wallet = ZotsWallet::new(config).await?;
    wallet.init_account().await?;

    // Convert txid from hex string to bytes
    let txid_bytes = att.txid_bytes()?;

    let result = wallet
        .verify_timestamp_tx(&txid_bytes, &proof_hash_bytes, Some(att.block_height))
        .await?;

    if result.valid {
        println!();
        print_success("VALID TIMESTAMP (verified on-chain)");
        print_info("Network", &att.network.to_string());
        print_info("Block", &att.block_height.to_string());
        print_info("Time", &att.timestamp().to_rfc3339());
        print_info("TXID", att.txid_hex());
        print_link("Explorer", &att.explorer_link());
    } else {
        println!();
        print_error("VERIFICATION FAILED");
        if let Some(error) = result.error {
            print_info("Reason", &error);
        }
        print_info("TXID", att.txid_hex());
    }

    Ok(())
}
