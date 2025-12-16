//! Encode command implementation.
//!
//! Converts a proof to compact CBOR+Base64 format for embedding in files,
//! metadata, git commits, or QR codes.
//!
//! Input can be:
//! - A .zots file path
//! - A JSON string
//! - An existing compact string (for validation)

use crate::output::*;
use std::path::PathBuf;
use zots_core::TimestampProof;

pub fn run(input: String) -> anyhow::Result<()> {
    print_header("Encoding Proof");

    // Check if input is a file path or already compact format
    let path = PathBuf::from(&input);
    let proof = if path.exists() {
        print_info("Input", &input);
        TimestampProof::load(&path)?
    } else if TimestampProof::is_compact_format(&input) {
        // Already compact, just validate and re-encode
        print_info("Input", "compact string");
        TimestampProof::from_compact(&input)?
    } else {
        // Try to parse as JSON
        print_info("Input", "JSON string");
        TimestampProof::deserialize(&input)?
    };

    // Encode to compact format
    let compact = proof.to_compact()?;

    println!();
    print_header("Compact Format");
    println!("{}", compact);
    println!();
    print_info("Length", &format!("{} chars", compact.len()));

    // Show what's embedded
    println!();
    print_info("Hash", &proof.hash);
    print_info("Attestations", &proof.attestations.len().to_string());
    if let Some(att) = proof.attestations.first() {
        print_info("Network", &att.network.to_string());
        print_info("Block", &att.block_height.to_string());
    }

    Ok(())
}
