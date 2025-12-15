//! zots - zOpenTimestamps CLI
//!
//! A Zcash blockchain timestamping tool with cypherpunk TUI.

mod cli;
mod commands;
mod output;
mod tui;

use clap::Parser;
use cli::{Cli, Commands, WalletCommands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Stamp {
            file,
            hash,
            output,
            no_wait,
        } => commands::stamp::run(file, hash, output, no_wait).await,
        Commands::Verify { proof, file } => commands::verify::run(proof, file).await,
        Commands::Info { proof } => commands::info::run(proof),
        Commands::Wallet { command } => match command {
            WalletCommands::Sync => commands::wallet::sync().await,
            WalletCommands::Balance => commands::wallet::balance().await,
            WalletCommands::Address => commands::wallet::address().await,
            WalletCommands::Info => commands::wallet::info().await,
        },
        Commands::Tui => tui::run().await,
    }
}
