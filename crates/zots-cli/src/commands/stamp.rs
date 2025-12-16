//! Stamp command implementation

use crate::output::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use zots_core::{TimestampProof, ZcashAttestation, hash_file, hash_from_hex, hash_to_hex};
use zots_zcash::{ZcashConfig, ZotsWallet};

pub async fn run(
    file: Option<PathBuf>,
    hash: Option<String>,
    output: Option<PathBuf>,
    no_wait: bool,
) -> anyhow::Result<()> {
    // Determine hash to timestamp
    let (hash_bytes, output_path) = if let Some(file_path) = file {
        print_header("Timestamping File");

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Hashing file...");

        let hash = hash_file(&file_path)?;
        pb.finish_with_message("Hashing complete");

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
        print_hash(&hash_to_hex(&hash));

        (hash, output)
    } else if let Some(hex) = hash {
        print_header("Timestamping Hash");

        let hash = hash_from_hex(&hex)?;
        let output = output.unwrap_or_else(|| PathBuf::from(format!("{}.zots", &hex[..16])));

        print_hash(&hash_to_hex(&hash));

        (hash, output)
    } else {
        return Err(anyhow::anyhow!(
            "Either a file path or --hash must be provided"
        ));
    };

    // Initialize wallet
    let config = ZcashConfig::from_env()?;
    let mut wallet = ZotsWallet::new(config.clone()).await?;
    wallet.init_account().await?;

    // Sync wallet
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Syncing wallet...");
    wallet.sync().await?;
    pb.finish_with_message("Wallet synced");

    // Create and broadcast transaction
    let pb = ProgressBar::new_spinner();
    pb.set_message("Creating transaction...");
    let tx_result = wallet.create_timestamp_tx(&hash_bytes).await?;
    pb.finish_with_message("Transaction broadcast");

    print_info("TXID", &tx_result.txid);

    // Create proof
    let mut proof = TimestampProof::new(hash_bytes);

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
