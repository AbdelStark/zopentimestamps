//! CLI argument parsing using clap.
//!
//! Defines the command-line interface structure using clap's derive macros.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use zots_core::HashAlgorithm;

/// zOpenTimestamps - Zcash blockchain timestamping CLI
///
/// Timestamp files and hashes on the Zcash blockchain with privacy-preserving
/// shielded transactions. Proofs can be verified against the blockchain.
///
/// WARNING: This is experimental software. Do not use on mainnet with real funds.
#[derive(Parser)]
#[command(name = "zots")]
#[command(
    author,
    version,
    about = "Zcash blockchain timestamping with cypherpunk TUI",
    long_about = "Timestamp files and hashes on the Zcash blockchain.\n\n\
                  WARNING: This is experimental software. Do not use on mainnet with real funds.\n\
                  The code has not been audited. Use only on testnet for development and testing."
)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Log level for diagnostic output
    #[arg(long, value_enum, default_value_t = LogLevelArg::Info, global = true, value_name = "LEVEL")]
    pub log_level: LogLevelArg,

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

        /// Hash algorithm to use (sha256 or blake3)
        #[arg(long, value_enum, default_value_t = HashAlgorithmArg::Sha256, value_name = "ALGO")]
        hash_algorithm: HashAlgorithmArg,

        /// Display QR code for the compact proof output
        #[arg(long)]
        qr: bool,

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

        /// Display QR code for the compact proof output
        #[arg(long)]
        qr: bool,
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

/// Hash algorithm option for CLI arguments
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum HashAlgorithmArg {
    Sha256,
    Blake3,
}

impl From<HashAlgorithmArg> for HashAlgorithm {
    fn from(value: HashAlgorithmArg) -> Self {
        match value {
            HashAlgorithmArg::Sha256 => HashAlgorithm::Sha256,
            HashAlgorithmArg::Blake3 => HashAlgorithm::Blake3,
        }
    }
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

/// Log level option for CLI
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum LogLevelArg {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
