//! Wallet command implementations

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

    print_info("Balance", &format!("{:.8} ZEC ({} zatoshis)", zec, balance));

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
            "{:.8} ZEC ({} zatoshis)",
            balance as f64 / 100_000_000.0,
            balance
        ),
    );
    print_info("Address", &address);

    Ok(())
}

pub async fn debug() -> anyhow::Result<()> {
    print_header("Wallet Debug - Address Derivation");

    let config = ZcashConfig::from_env()?;
    let wallet = ZotsWallet::new(config).await?;

    wallet.debug_show_all_accounts()?;

    println!();
    print_status("If none of these match Zingo's addresses, check:");
    println!("  1. Is the seed phrase identical (including word order)?");
    println!("  2. Is Zingo using a BIP-39 passphrase?");
    println!("  3. Is Zingo in testnet mode?");

    Ok(())
}
