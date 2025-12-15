//! zots-core - Core library for zOpenTimestamps
//!
//! Provides hashing, proof format serialization, and verification logic
//! for Zcash blockchain timestamping.

pub mod error;
pub mod hash;
pub mod proof;

pub use error::{Error, Result};
pub use hash::*;
pub use proof::*;
