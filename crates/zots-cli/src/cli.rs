//! CLI argument parsing using clap

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// zOpenTimestamps - Zcash blockchain timestamping CLI
#[derive(Parser)]
#[command(name = "zots")]
#[command(
    author,
    version,
    about = "Zcash blockchain timestamping with cypherpunk TUI"
)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Timestamp a file or hash on the Zcash blockchain
    Stamp {
        /// File to timestamp
        #[arg(conflicts_with = "hash")]
        file: Option<PathBuf>,

        /// Hash to timestamp (hex string, 40 or 64 chars)
        #[arg(long, conflicts_with = "file")]
        hash: Option<String>,

        /// Output proof file path (default: <file>.zots or <hash>.zots)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Don't wait for confirmation (create pending proof)
        #[arg(long)]
        no_wait: bool,
    },

    /// Verify a timestamp proof
    Verify {
        /// Proof file (.zots)
        proof: PathBuf,

        /// Original file to verify against (optional)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Display proof information
    Info {
        /// Proof file (.zots)
        proof: PathBuf,
    },

    /// Encode a .zots proof to compact format (CBOR+Base64)
    Encode {
        /// Proof file (.zots) or compact string to encode
        input: String,
    },

    /// Decode a compact proof string to JSON
    Decode {
        /// Compact proof string (zots1...) to decode
        compact: String,

        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Wallet management commands
    Wallet {
        #[command(subcommand)]
        command: WalletCommands,
    },

    /// Launch interactive TUI mode
    Tui,
}

#[derive(Subcommand)]
pub enum WalletCommands {
    /// Sync wallet with the blockchain
    Sync,

    /// Show wallet balance
    Balance,

    /// Show receiving address
    Address,

    /// Show wallet info (height, balance, address)
    Info,
}
