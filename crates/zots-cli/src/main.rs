//! # zots - zOpenTimestamps CLI
//!
//! A Zcash blockchain timestamping tool with cypherpunk TUI.
//!
//! ## Commands
//!
//! - `stamp` - Timestamp a file or hash on the Zcash blockchain
//! - `verify` - Verify a timestamp proof against the blockchain
//! - `info` - Display proof information
//! - `encode` - Convert proof to compact embeddable format
//! - `decode` - Convert compact format back to JSON
//! - `wallet` - Wallet management (sync, balance, address)
//! - `tui` - Launch interactive terminal UI
//!
//! ## Usage
//!
//! ```bash
//! # Timestamp a file
//! zots stamp document.pdf
//!
//! # Verify a proof
//! zots verify document.pdf.zots
//!
//! # Launch TUI
//! zots tui
//! ```
//!
//! ## Security Warning
//!
//! This is experimental software. Do not use on mainnet with real funds.

mod cli;
mod commands;
mod output;
mod tui;

use clap::Parser;
use cli::{Cli, Commands, LogLevelArg, WalletCommands};
use tracing_subscriber::filter::LevelFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();
    init_logging(cli.log_level);

    match cli.command {
        Commands::Stamp {
            file,
            hash,
            output,
            hash_algorithm,
            qr,
            no_wait,
        } => commands::stamp::run(file, hash, output, hash_algorithm.into(), qr, no_wait).await,
        Commands::Verify { proof, file } => commands::verify::run(proof, file).await,
        Commands::Info { proof } => commands::info::run(proof),
        Commands::Encode { input, qr } => commands::encode::run(input, qr),
        Commands::Decode { compact, output } => commands::decode::run(compact, output),
        Commands::Wallet { command } => match command {
            WalletCommands::Sync => commands::wallet::sync().await,
            WalletCommands::Balance => commands::wallet::balance().await,
            WalletCommands::Address => commands::wallet::address().await,
            WalletCommands::Info => commands::wallet::info().await,
        },
        Commands::Tui => tui::run().await,
    }
}

/// Initialize global logging with the desired level.
fn init_logging(level: LogLevelArg) {
    let level_filter = match level {
        LogLevelArg::Error => LevelFilter::ERROR,
        LogLevelArg::Warn => LevelFilter::WARN,
        LogLevelArg::Info => LevelFilter::INFO,
        LogLevelArg::Debug => LevelFilter::DEBUG,
        LogLevelArg::Trace => LevelFilter::TRACE,
    };

    // Ignore errors if already initialized (e.g., in tests)
    let _ = tracing_subscriber::fmt()
        .with_max_level(level_filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .try_init();
}
