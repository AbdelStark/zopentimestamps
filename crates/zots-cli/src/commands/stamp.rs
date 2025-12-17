//! Stamp command implementation.
//!
//! Creates a timestamp by broadcasting a shielded transaction with the hash
//! encoded in the memo field. The resulting proof can be used to verify that
//! the data existed at the time the transaction was confirmed.
//!
//! ## Warning
//!
//! This command sends a real blockchain transaction. Only use on testnet.

use crate::output::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tracing::{debug, info};
use zots_core::{
    HashAlgorithm, TimestampProof, ZcashAttestation, hash_file_with, hash_from_hex_with,
    hash_to_hex,
};
use zots_zcash::{ZcashConfig, ZotsWallet};

pub async fn run(
    file: Option<PathBuf>,
    hash: Option<String>,
    output: Option<PathBuf>,
    hash_algorithm: HashAlgorithm,
    no_wait: bool,
) -> anyhow::Result<()> {
    info!("Starting stamp operation");
    debug!("Selected hash algorithm: {}", hash_algorithm.name());

    // Determine hash to timestamp
    let (hash_bytes, output_path) = if let Some(file_path) = file {
        print_header("Timestamping File");
        info!("Hashing file {}", file_path.display());

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Hashing file...");

        let hash = hash_file_with(&file_path, hash_algorithm)?;
        pb.finish_with_message("Hashing complete");
        debug!("Computed hash: {}", hash_to_hex(&hash));

        let output = output.unwrap_or_else(|| {
            let mut p = file_path.clone();
            let new_name = format!(
                "{}.zots",
                p.file_name().unwrap_or_default().to_string_lossy()
            );
            p.set_file_name(new_name);
            p
        });

        print_info("File", &file_path.display().to_string());
        print_hash(&hash_to_hex(&hash), hash_algorithm.name());

        (hash, output)
    } else if let Some(hex) = hash {
        print_header("Timestamping Hash");
        info!("Using provided hash input");

        let hash = hash_from_hex_with(&hex, hash_algorithm)?;
        let output = output.unwrap_or_else(|| PathBuf::from(format!("{}.zots", &hex[..16])));

        print_hash(&hash_to_hex(&hash), hash_algorithm.name());

        (hash, output)
    } else {
        return Err(anyhow::anyhow!(
            "Either a file path or --hash must be provided"
        ));
    };

    // Initialize wallet
    let config = ZcashConfig::from_env()?;
    let mut wallet = ZotsWallet::new(config.clone()).await?;
    info!("Initializing wallet");
    wallet.init_account().await?;

    // Sync wallet
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Syncing wallet...");
    debug!("Syncing wallet with lightwalletd");
    wallet.sync().await?;
    pb.finish_with_message("Wallet synced");

    // Create and broadcast transaction
    let pb = ProgressBar::new_spinner();
    pb.set_message("Creating transaction...");
    info!("Creating timestamp transaction");
    let tx_result = wallet.create_timestamp_tx(&hash_bytes).await?;
    pb.finish_with_message("Transaction broadcast");

    print_info("TXID", &tx_result.txid);

    // Create proof
    let mut proof = TimestampProof::new_with_algorithm(hash_bytes, hash_algorithm);

    if no_wait {
        print_warning("Not waiting for confirmation - proof will be pending");
        proof.save(&output_path)?;
        print_success(&format!("Pending proof saved: {}", output_path.display()));

        // Show compact format for embedding
        println!();
        print_info("Compact", &proof.to_compact()?);
        return Ok(());
    }

    // Wait for confirmation
    let pb = ProgressBar::new_spinner();
    pb.set_message("Waiting for confirmation...");
    let confirmation = wallet.wait_confirmation(&tx_result.txid, 10).await?;
    pb.finish_with_message("Transaction confirmed");

    // Add attestation
    proof.add_attestation(ZcashAttestation::new(
        config.network,
        tx_result.txid_bytes,
        confirmation.block_height,
        confirmation.block_time,
        0,
    ));

    // Save proof
    proof.save(&output_path)?;

    print_success(&format!("Confirmed in block {}", confirmation.block_height));
    print_success(&format!("Proof saved: {}", output_path.display()));

    // Show compact format for embedding
    println!();
    print_header("Embeddable Proof");
    let compact = proof.to_compact()?;
    println!("{}", compact);
    println!();
    print_info("Length", &format!("{} chars", compact.len()));

    Ok(())
}
