//! .zots proof format
//!
//! Human-readable JSON format for timestamp proofs with Zcash attestations.

use crate::{Error, Hash256, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Magic header for ZOTS timestamp memo: \x00zOTS\x00\x00\x01
/// Used in blockchain memo fields to identify timestamp data
pub const ZOTS_MAGIC: [u8; 8] = [0x00, 0x7A, 0x4F, 0x54, 0x53, 0x00, 0x00, 0x01];

/// Current proof format version
pub const PROOF_VERSION: u8 = 1;

/// Network type (mainnet or testnet)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    /// Get the block explorer URL for this network
    pub fn explorer_url(&self) -> &'static str {
        match self {
            Network::Mainnet => "https://explorer.zec.rocks",
            Network::Testnet => "https://testnet.zcashexplorer.app",
        }
    }

    /// Get network name as string
    pub fn name(&self) -> &'static str {
        match self {
            Network::Mainnet => "mainnet",
            Network::Testnet => "testnet",
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A Zcash blockchain attestation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZcashAttestation {
    /// Network where the transaction was broadcast
    pub network: Network,
    /// Transaction ID (hex string, display byte order)
    pub txid: String,
    /// Block height where transaction was confirmed
    pub block_height: u32,
    /// Block timestamp (Unix timestamp)
    pub block_time: u32,
    /// Offset in memo field where hash is stored
    pub memo_offset: u16,
}

impl ZcashAttestation {
    /// Create a new attestation from raw txid bytes (internal byte order)
    pub fn new(
        network: Network,
        txid_bytes: [u8; 32],
        block_height: u32,
        block_time: u32,
        memo_offset: u16,
    ) -> Self {
        // Convert to display byte order (reversed) and hex string
        let mut reversed = txid_bytes;
        reversed.reverse();
        let txid = hex::encode(reversed);

        Self {
            network,
            txid,
            block_height,
            block_time,
            memo_offset,
        }
    }

    /// Get the transaction ID as a hex string (display byte order)
    pub fn txid_hex(&self) -> &str {
        &self.txid
    }

    /// Get the txid as raw bytes (internal byte order)
    pub fn txid_bytes(&self) -> Result<[u8; 32]> {
        let bytes = hex::decode(&self.txid)
            .map_err(|e| Error::InvalidProof(format!("Invalid txid hex: {}", e)))?;
        if bytes.len() != 32 {
            return Err(Error::InvalidProof("TXID must be 32 bytes".into()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        // Reverse back to internal byte order
        arr.reverse();
        Ok(arr)
    }

    /// Get the full explorer link for this transaction
    pub fn explorer_link(&self) -> String {
        format!("{}/tx/{}", self.network.explorer_url(), self.txid)
    }

    /// Get the block timestamp as a DateTime
    pub fn timestamp(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.block_time as i64, 0).unwrap_or_default()
    }
}

/// A timestamp proof containing hash and attestations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampProof {
    /// Proof format version
    pub version: u8,
    /// SHA-256 hash of the timestamped data (hex string)
    pub hash: String,
    /// List of blockchain attestations
    pub attestations: Vec<ZcashAttestation>,
}

impl TimestampProof {
    /// Create a new proof for a hash (no attestations yet)
    pub fn new(hash: Hash256) -> Self {
        Self {
            version: PROOF_VERSION,
            hash: hex::encode(hash),
            attestations: Vec::new(),
        }
    }

    /// Get the hash as raw bytes
    pub fn hash_bytes(&self) -> Result<Hash256> {
        let bytes = hex::decode(&self.hash)
            .map_err(|e| Error::InvalidProof(format!("Invalid hash hex: {}", e)))?;
        if bytes.len() != 32 {
            return Err(Error::InvalidProof("Hash must be 32 bytes".into()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }

    /// Add an attestation to the proof
    pub fn add_attestation(&mut self, att: ZcashAttestation) {
        self.attestations.push(att);
    }

    /// Check if the proof has any confirmed attestations
    pub fn is_confirmed(&self) -> bool {
        !self.attestations.is_empty()
    }

    /// Serialize the proof to JSON
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| Error::InvalidProof(format!("JSON serialization failed: {}", e)))
    }

    /// Deserialize a proof from JSON
    pub fn deserialize(data: &str) -> Result<Self> {
        let proof: Self = serde_json::from_str(data)
            .map_err(|e| Error::InvalidProof(format!("JSON parse error: {}", e)))?;

        if proof.version != PROOF_VERSION {
            return Err(Error::InvalidProof(format!(
                "Unsupported version: {}",
                proof.version
            )));
        }

        // Validate hash is valid hex
        let _ = proof.hash_bytes()?;

        // Validate all txids are valid hex
        for att in &proof.attestations {
            let _ = att.txid_bytes()?;
        }

        Ok(proof)
    }

    /// Save the proof to a file
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let json = self.serialize()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load a proof from a file
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        Self::deserialize(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_serialization() {
        assert_eq!(
            serde_json::to_string(&Network::Mainnet).unwrap(),
            "\"mainnet\""
        );
        assert_eq!(
            serde_json::to_string(&Network::Testnet).unwrap(),
            "\"testnet\""
        );
    }

    #[test]
    fn test_network_deserialization() {
        let mainnet: Network = serde_json::from_str("\"mainnet\"").unwrap();
        let testnet: Network = serde_json::from_str("\"testnet\"").unwrap();
        assert_eq!(mainnet, Network::Mainnet);
        assert_eq!(testnet, Network::Testnet);
    }

    #[test]
    fn test_attestation_txid() {
        let txid_bytes = [0xAB; 32];
        let att = ZcashAttestation::new(Network::Testnet, txid_bytes, 100, 1700000000, 0);

        // Display order is reversed
        let hex = att.txid_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c == 'a' || c == 'b'));

        // Round-trip back to bytes
        let recovered = att.txid_bytes().unwrap();
        assert_eq!(recovered, txid_bytes);
    }

    #[test]
    fn test_proof_new() {
        let hash = [0x42u8; 32];
        let proof = TimestampProof::new(hash);
        assert_eq!(proof.version, PROOF_VERSION);
        assert_eq!(proof.hash, hex::encode(hash));
        assert!(proof.attestations.is_empty());
        assert!(!proof.is_confirmed());
    }

    #[test]
    fn test_proof_hash_bytes() {
        let hash = [0x42u8; 32];
        let proof = TimestampProof::new(hash);
        assert_eq!(proof.hash_bytes().unwrap(), hash);
    }

    #[test]
    fn test_proof_roundtrip() {
        let hash = [0xABu8; 32];
        let mut proof = TimestampProof::new(hash);

        proof.add_attestation(ZcashAttestation::new(
            Network::Testnet,
            [0xCDu8; 32],
            3721456,
            1734567890,
            8,
        ));

        assert!(proof.is_confirmed());

        let json = proof.serialize().unwrap();
        let deserialized = TimestampProof::deserialize(&json).unwrap();

        assert_eq!(deserialized.version, proof.version);
        assert_eq!(deserialized.hash, proof.hash);
        assert_eq!(deserialized.attestations.len(), 1);
        assert_eq!(deserialized.attestations[0].network, Network::Testnet);
        assert_eq!(deserialized.attestations[0].block_height, 3721456);
        assert_eq!(deserialized.attestations[0].block_time, 1734567890);
        assert_eq!(deserialized.attestations[0].memo_offset, 8);
    }

    #[test]
    fn test_proof_json_is_readable() {
        let hash = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
        ];
        let mut proof = TimestampProof::new(hash);
        proof.add_attestation(ZcashAttestation::new(
            Network::Testnet,
            [0x11; 32],
            12345,
            1700000000,
            0,
        ));

        let json = proof.serialize().unwrap();

        // Verify it's human-readable JSON
        assert!(json.contains("\"version\": 1"));
        assert!(json.contains("\"hash\":"));
        assert!(json.contains("\"attestations\":"));
        assert!(json.contains("\"network\": \"testnet\""));
        assert!(json.contains("\"block_height\": 12345"));
    }

    #[test]
    fn test_proof_multiple_attestations() {
        let hash = [0x00u8; 32];
        let mut proof = TimestampProof::new(hash);

        proof.add_attestation(ZcashAttestation::new(
            Network::Testnet,
            [0x01u8; 32],
            100,
            1000,
            0,
        ));

        proof.add_attestation(ZcashAttestation::new(
            Network::Mainnet,
            [0x02u8; 32],
            200,
            2000,
            0,
        ));

        let json = proof.serialize().unwrap();
        let deserialized = TimestampProof::deserialize(&json).unwrap();

        assert_eq!(deserialized.attestations.len(), 2);
        assert_eq!(deserialized.attestations[0].network, Network::Testnet);
        assert_eq!(deserialized.attestations[1].network, Network::Mainnet);
    }

    #[test]
    fn test_proof_invalid_json() {
        assert!(TimestampProof::deserialize("not json").is_err());
    }

    #[test]
    fn test_proof_invalid_hash() {
        let json = r#"{"version": 1, "hash": "not_hex", "attestations": []}"#;
        assert!(TimestampProof::deserialize(json).is_err());
    }

    #[test]
    fn test_proof_file_roundtrip() {
        let hash = [0x55u8; 32];
        let mut proof = TimestampProof::new(hash);
        proof.add_attestation(ZcashAttestation::new(
            Network::Testnet,
            [0x66u8; 32],
            12345,
            1700000000,
            0,
        ));

        let temp_path = std::env::temp_dir().join("test_proof.zots");
        proof.save(&temp_path).unwrap();

        let loaded = TimestampProof::load(&temp_path).unwrap();
        assert_eq!(loaded.hash, proof.hash);
        assert_eq!(loaded.attestations.len(), 1);

        // Cleanup
        std::fs::remove_file(&temp_path).ok();
    }
}
