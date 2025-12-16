//! CLI command implementations.
//!
//! Each submodule implements one CLI command:
//!
//! - [`stamp`] - Timestamp files or hashes on the blockchain
//! - [`verify`] - Verify timestamp proofs against the blockchain
//! - [`info`] - Display proof file information
//! - [`encode`] - Convert proofs to compact format
//! - [`decode`] - Convert compact format to JSON
//! - [`wallet`] - Wallet management operations

pub mod decode;
pub mod encode;
pub mod info;
pub mod stamp;
pub mod verify;
pub mod wallet;
