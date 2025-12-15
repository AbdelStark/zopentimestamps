//! Memo field encoding for timestamp data

use zots_core::proof::ZOTS_MAGIC;

/// Create a memo field containing timestamp data
///
/// Format: ZOTS_MAGIC (8 bytes) + hash (32 bytes) = 40 bytes
/// Padded to 512 bytes for Zcash memo field
pub fn create_timestamp_memo(hash: &[u8; 32]) -> Vec<u8> {
    let mut data = Vec::with_capacity(512);
    data.extend_from_slice(&ZOTS_MAGIC);
    data.extend_from_slice(hash);

    // Pad to 512 bytes (Zcash memo field size)
    data.resize(512, 0);

    data
}

/// Parse hash from a memo field
///
/// Returns None if memo doesn't have valid ZOTS magic header
pub fn parse_timestamp_memo(memo: &[u8]) -> Option<[u8; 32]> {
    if memo.len() < 40 {
        return None;
    }

    // Check magic header
    if memo[0..8] != ZOTS_MAGIC {
        return None;
    }

    // Extract hash
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&memo[8..40]);
    Some(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_memo() {
        let hash = [0xAB; 32];
        let memo = create_timestamp_memo(&hash);

        assert_eq!(memo.len(), 512);
        assert_eq!(&memo[0..8], &ZOTS_MAGIC);
        assert_eq!(&memo[8..40], &hash);
    }

    #[test]
    fn test_parse_memo_roundtrip() {
        let hash = [0xCD; 32];
        let memo = create_timestamp_memo(&hash);
        let parsed = parse_timestamp_memo(&memo);

        assert_eq!(parsed, Some(hash));
    }

    #[test]
    fn test_parse_memo_invalid_magic() {
        let mut memo = vec![0u8; 512];
        memo[0..8].copy_from_slice(&[0x00; 8]); // Invalid magic

        assert_eq!(parse_timestamp_memo(&memo), None);
    }

    #[test]
    fn test_parse_memo_too_short() {
        let memo = vec![0u8; 20];
        assert_eq!(parse_timestamp_memo(&memo), None);
    }
}
