//! Info command implementation

use crate::output::*;
use std::path::PathBuf;
use zots_core::{TimestampProof, hash_to_hex};

pub fn run(proof_path: PathBuf) -> anyhow::Result<()> {
    print_header("Proof Information");

    let proof = TimestampProof::load(&proof_path)?;

    print_info("File", &proof_path.display().to_string());
    print_info("Version", &proof.version.to_string());
    print_hash(&hash_to_hex(&proof.hash));
    print_info("Attestations", &proof.attestations.len().to_string());
    print_info(
        "Status",
        if proof.is_confirmed() {
            "Confirmed"
        } else {
            "Pending"
        },
    );

    if !proof.attestations.is_empty() {
        for (i, att) in proof.attestations.iter().enumerate() {
            println!();
            println!("  {} Attestation #{}", "─".repeat(3), i + 1);
            print_info("  Network", &att.network.to_string());
            print_info("  TXID", &att.txid_hex());
            print_info("  Block", &att.block_height.to_string());
            print_info("  Time", &att.timestamp().to_rfc3339());
            print_link("  Explorer", &att.explorer_link());
        }
    }

    Ok(())
}
