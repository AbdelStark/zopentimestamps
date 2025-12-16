//! Verify command implementation

use crate::output::*;
use std::path::PathBuf;
use zots_core::{TimestampProof, hash_file, hash_to_hex};
use zots_zcash::{ZcashConfig, ZotsWallet};

pub async fn run(proof_path: PathBuf, file: Option<PathBuf>) -> anyhow::Result<()> {
    print_header("Verifying Timestamp");

    // Load proof
    let proof = TimestampProof::load(&proof_path)?;
    print_info("Proof", &proof_path.display().to_string());
    print_hash(&hash_to_hex(&proof.hash));

    // Verify against original file if provided
    if let Some(file_path) = file {
        print_status("Verifying hash against original file...");
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

    let result = wallet
        .verify_timestamp_tx(&att.txid, &proof.hash, Some(att.block_height))
        .await?;

    if result.valid {
        println!();
        print_success("VALID TIMESTAMP (verified on-chain)");
        print_info("Network", &att.network.to_string());
        print_info("Block", &att.block_height.to_string());
        print_info("Time", &att.timestamp().to_rfc3339());
        print_info("TXID", &att.txid_hex());
        print_link("Explorer", &att.explorer_link());
    } else {
        println!();
        print_error("VERIFICATION FAILED");
        if let Some(error) = result.error {
            print_info("Reason", &error);
        }
        print_info("TXID", &att.txid_hex());
    }

    Ok(())
}
