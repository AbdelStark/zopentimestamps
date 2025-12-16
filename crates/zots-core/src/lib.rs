//! # zots-core
//!
//! Core library for zOpenTimestamps providing hashing, proof format
//! serialization, and verification logic for Zcash blockchain timestamping.
//!
//! ## Overview
//!
//! This crate provides the fundamental types and operations for creating
//! and verifying timestamp proofs:
//!
//! - **Hashing**: SHA-256 (default) and optional BLAKE3 hashing of files and data
//! - **Proof Format**: JSON and compact CBOR+Base64 serialization
//! - **Attestations**: Blockchain attestation records
//!
//! ## Example
//!
//! ```rust,no_run
//! use zots_core::{TimestampProof, ZcashAttestation, Network, hash_file};
//!
//! // Hash a file
//! let hash = hash_file("document.pdf").unwrap();
//!
//! // Create a proof
//! let mut proof = TimestampProof::new(hash);
//!
//! // Add an attestation (after blockchain confirmation)
//! proof.add_attestation(ZcashAttestation::new(
//!     Network::Testnet,
//!     [0u8; 32], // txid bytes
//!     3739654,   // block height
//!     1734293400, // block time
//!     0,         // memo offset
//! ));
//!
//! // Serialize to JSON
//! let json = proof.serialize().unwrap();
//!
//! // Or to compact format for embedding
//! let compact = proof.to_compact().unwrap();
//! ```
//!
//! ## Security Warning
//!
//! This is experimental software. Do not use on mainnet with real funds.
//! The code has not been audited.

pub mod error;
pub mod hash;
pub mod proof;

pub use error::{Error, Result};
pub use hash::*;
pub use proof::*;
