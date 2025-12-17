//! Wallet command implementations.
//!
//! Provides wallet management operations:
//! - `sync` - Sync wallet state with the blockchain
//! - `balance` - Show balance breakdown by pool (Orchard, Sapling, Transparent)
//! - `address` - Show unified receiving address
//! - `info` - Show comprehensive wallet information

use crate::output::*;
use indicatif::{ProgressBar, ProgressStyle};
use zots_zcash::{ZcashConfig, ZotsWallet};

pub async fn sync() -> anyhow::Result<()> {
    print_header("Syncing Wallet");

    let config = ZcashConfig::from_env()?;
    let mut wallet = ZotsWallet::new(config).await?;
    wallet.init_account().await?;

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Syncing with blockchain...");

    wallet.sync().await?;

    pb.finish_with_message("Sync complete");
    print_success("Wallet synchronized");

    Ok(())
}

pub async fn balance() -> anyhow::Result<()> {
    print_header("Wallet Balance");

    let config = ZcashConfig::from_env()?;
    let mut wallet = ZotsWallet::new(config).await?;
    wallet.init_account().await?;

    print_status("Syncing wallet...");
    wallet.sync().await?;

    let balance = wallet.get_balance()?;
    let zec = balance as f64 / 100_000_000.0;

    print_info("Balance", &format!("{zec:.8} ZEC ({balance} zatoshis)"));

    Ok(())
}

pub async fn address() -> anyhow::Result<()> {
    print_header("Wallet Address");

    let config = ZcashConfig::from_env()?;
    let mut wallet = ZotsWallet::new(config).await?;
    wallet.init_account().await?;

    let address = wallet.get_address()?;
    print_info("Address", &address);

    println!();
    print_status("Fund this address with testnet ZEC from:");
    print_link("Faucet", "https://testnet.zecfaucet.com/");

    Ok(())
}

pub async fn info() -> anyhow::Result<()> {
    print_header("Wallet Info");

    let config = ZcashConfig::from_env()?;
    let mut wallet = ZotsWallet::new(config.clone()).await?;
    wallet.init_account().await?;

    print_status("Syncing wallet...");
    wallet.sync().await?;

    let height = wallet.get_block_height().await?;
    let balance = wallet.get_balance()?;
    let address = wallet.get_address()?;

    print_info("Network", &config.network.to_string());
    print_info("Lightwalletd", &config.lightwalletd_url);
    print_info("Data Dir", &config.data_dir.display().to_string());
    print_info("Block Height", &height.to_string());
    print_info(
        "Balance",
        &format!(
            "{:.8} ZEC ({balance} zatoshis)",
            balance as f64 / 100_000_000.0
        ),
    );
    print_info("Address", &address);

    Ok(())
}
