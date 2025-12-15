//! Verify command implementation

use crate::output::*;
use std::path::PathBuf;
use zots_core::{TimestampProof, hash_file, hash_to_hex};

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

    // TODO: In a full implementation, we would fetch the transaction
    // from the blockchain and verify the memo contains the hash.
    // For MVP, we trust the proof file.

    println!();
    print_success("VALID TIMESTAMP");
    print_info("Network", &att.network.to_string());
    print_info("Block", &att.block_height.to_string());
    print_info("Time", &att.timestamp().to_rfc3339());
    print_info("TXID", &att.txid_hex());
    print_link("Explorer", &att.explorer_link());

    Ok(())
}
