//! Error types for zots-core

use thiserror::Error;

/// Core errors for zOpenTimestamps operations
#[derive(Error, Debug)]
pub enum Error {
    /// IO error during file operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid proof format or corrupted proof file
    #[error("Invalid proof format: {0}")]
    InvalidProof(String),

    /// Hash mismatch during verification
    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    /// Invalid hash format (wrong length or non-hex characters)
    #[error("Invalid hash format: {0}")]
    InvalidHash(String),

    /// Proof not yet confirmed on blockchain
    #[error("Proof not yet confirmed")]
    NotConfirmed,

    /// Transaction not found on blockchain
    #[error("Transaction not found: {0}")]
    TxNotFound(String),

    /// Network communication error
    #[error("Network error: {0}")]
    Network(String),
}

/// Result type alias for zots-core operations
pub type Result<T> = std::result::Result<T, Error>;
