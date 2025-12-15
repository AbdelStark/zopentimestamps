//! SHA-256 hashing utilities
//!
//! Provides functions to hash files, bytes, and hex strings.

use crate::{Error, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// 32-byte SHA-256 hash
pub type Hash256 = [u8; 32];

/// Hash raw bytes using SHA-256
pub fn hash_bytes(data: &[u8]) -> Hash256 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash a file by path using streaming for memory efficiency
pub fn hash_file(path: impl AsRef<Path>) -> Result<Hash256> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();

    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().into())
}

/// Parse a hex string and return a Hash256
///
/// Accepts:
/// - 40 hex chars (git commit hash) - will be hashed to get 32 bytes
/// - 64 hex chars (full SHA-256 hash) - used directly
pub fn hash_from_hex(hex_str: &str) -> Result<Hash256> {
    let cleaned = hex_str.trim().trim_start_matches("0x");

    if cleaned.len() == 40 {
        // Git commit hash (20 bytes) - hash it to get 32 bytes
        let bytes =
            hex::decode(cleaned).map_err(|e| Error::InvalidHash(format!("Invalid hex: {}", e)))?;
        Ok(hash_bytes(&bytes))
    } else if cleaned.len() == 64 {
        // Already a SHA-256 hash
        let bytes =
            hex::decode(cleaned).map_err(|e| Error::InvalidHash(format!("Invalid hex: {}", e)))?;
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        Ok(hash)
    } else {
        Err(Error::InvalidHash(format!(
            "Expected 40 or 64 hex chars, got {}",
            cleaned.len()
        )))
    }
}

/// Convert a Hash256 to hex string
pub fn hash_to_hex(hash: &Hash256) -> String {
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes() {
        let data = b"hello world";
        let hash = hash_bytes(data);
        let hex = hash_to_hex(&hash);
        assert_eq!(
            hex,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_hash_bytes_empty() {
        let data = b"";
        let hash = hash_bytes(data);
        let hex = hash_to_hex(&hash);
        // SHA-256 of empty string
        assert_eq!(
            hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_hash_from_hex_git() {
        // 40 char git commit hash
        let git_hash = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        let result = hash_from_hex(git_hash);
        assert!(result.is_ok());
        // The result should be SHA-256 of the 20-byte git hash
        let hash = result.unwrap();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_hash_from_hex_sha256() {
        let sha256 = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let result = hash_from_hex(sha256);
        assert!(result.is_ok());
        let hash = result.unwrap();
        // Should be the same hash directly
        assert_eq!(hash_to_hex(&hash), sha256);
    }

    #[test]
    fn test_hash_from_hex_with_0x_prefix() {
        let sha256 = "0xb94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let result = hash_from_hex(sha256);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hash_from_hex_invalid_length() {
        let invalid = "abc123";
        let result = hash_from_hex(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_from_hex_invalid_chars() {
        let invalid = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
        let result = hash_from_hex(invalid);
        assert!(result.is_err());
    }
}
