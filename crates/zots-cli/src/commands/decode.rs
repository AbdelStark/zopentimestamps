//! Decode command implementation

use crate::output::*;
use std::path::PathBuf;
use zots_core::TimestampProof;

pub fn run(compact: String, output: Option<PathBuf>) -> anyhow::Result<()> {
    print_header("Decoding Proof");

    // Decode from compact format
    let proof = TimestampProof::from_compact(&compact)?;

    // Serialize to JSON
    let json = proof.serialize()?;

    if let Some(output_path) = output {
        // Save to file
        proof.save(&output_path)?;
        print_success(&format!("Proof saved: {}", output_path.display()));
    } else {
        // Print to stdout
        println!();
        println!("{}", json);
    }

    println!();
    print_info("Hash", &proof.hash);
    print_info("Attestations", &proof.attestations.len().to_string());

    if !proof.attestations.is_empty() {
        let att = &proof.attestations[0];
        print_info("Network", &att.network.to_string());
        print_info("Block", &att.block_height.to_string());
        print_info("Time", &att.timestamp().to_rfc3339());
        print_info("TXID", att.txid_hex());
    } else {
        print_warning("Pending proof (no attestations)");
    }

    Ok(())
}
