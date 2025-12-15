//! .zots proof format
//!
//! Binary format for timestamp proofs with Zcash attestations.

use crate::{Error, Hash256, Result};
use chrono::{DateTime, Utc};

/// Magic header: \x00zOTS\x00\x00\x01
pub const ZOTS_MAGIC: [u8; 8] = [0x00, 0x7A, 0x4F, 0x54, 0x53, 0x00, 0x00, 0x01];

/// Current proof format version
pub const PROOF_VERSION: u8 = 1;

/// Hash type identifier for SHA-256
pub const HASH_TYPE_SHA256: u8 = 0x00;

/// Attestation type identifier for Zcash transactions
pub const ATTESTATION_TYPE_ZCASH: u8 = 0x01;

/// Network type (mainnet or testnet)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    /// Convert network to byte representation
    pub fn to_byte(&self) -> u8 {
        match self {
            Network::Mainnet => 0x00,
            Network::Testnet => 0x01,
        }
    }

    /// Parse network from byte
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            0x00 => Ok(Network::Mainnet),
            0x01 => Ok(Network::Testnet),
            _ => Err(Error::InvalidProof(format!("Unknown network byte: {}", b))),
        }
    }

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
#[derive(Debug, Clone)]
pub struct ZcashAttestation {
    /// Network where the transaction was broadcast
    pub network: Network,
    /// Transaction ID (32 bytes, internal byte order)
    pub txid: [u8; 32],
    /// Block height where transaction was confirmed
    pub block_height: u32,
    /// Block timestamp (Unix timestamp)
    pub block_time: u32,
    /// Offset in memo field where hash is stored
    pub memo_offset: u16,
}

impl ZcashAttestation {
    /// Get the transaction ID as a hex string (display byte order - reversed)
    pub fn txid_hex(&self) -> String {
        // Zcash txids are displayed in reversed byte order
        let mut reversed = self.txid;
        reversed.reverse();
        hex::encode(reversed)
    }

    /// Get the full explorer link for this transaction
    pub fn explorer_link(&self) -> String {
        format!("{}/tx/{}", self.network.explorer_url(), self.txid_hex())
    }

    /// Get the block timestamp as a DateTime
    pub fn timestamp(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.block_time as i64, 0).unwrap_or_default()
    }
}

/// A timestamp proof containing hash and attestations
#[derive(Debug, Clone)]
pub struct TimestampProof {
    /// Proof format version
    pub version: u8,
    /// SHA-256 hash of the timestamped data
    pub hash: Hash256,
    /// List of blockchain attestations
    pub attestations: Vec<ZcashAttestation>,
}

impl TimestampProof {
    /// Create a new proof for a hash (no attestations yet)
    pub fn new(hash: Hash256) -> Self {
        Self {
            version: PROOF_VERSION,
            hash,
            attestations: Vec::new(),
        }
    }

    /// Add an attestation to the proof
    pub fn add_attestation(&mut self, att: ZcashAttestation) {
        self.attestations.push(att);
    }

    /// Check if the proof has any confirmed attestations
    pub fn is_confirmed(&self) -> bool {
        !self.attestations.is_empty()
    }

    /// Serialize the proof to binary .zots format
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Magic header (8 bytes)
        buf.extend_from_slice(&ZOTS_MAGIC);

        // Version (1 byte)
        buf.push(self.version);

        // Hash type (1 byte)
        buf.push(HASH_TYPE_SHA256);

        // Hash (32 bytes)
        buf.extend_from_slice(&self.hash);

        // Number of attestations (1 byte)
        buf.push(self.attestations.len() as u8);

        // Attestations
        for att in &self.attestations {
            // Type (1 byte)
            buf.push(ATTESTATION_TYPE_ZCASH);
            // Network (1 byte)
            buf.push(att.network.to_byte());
            // TXID (32 bytes)
            buf.extend_from_slice(&att.txid);
            // Block height (4 bytes, little-endian)
            buf.extend_from_slice(&att.block_height.to_le_bytes());
            // Block time (4 bytes, little-endian)
            buf.extend_from_slice(&att.block_time.to_le_bytes());
            // Memo offset (2 bytes, little-endian)
            buf.extend_from_slice(&att.memo_offset.to_le_bytes());
        }

        buf
    }

    /// Deserialize a proof from binary .zots format
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(Error::InvalidProof("File too small".into()));
        }

        // Check magic header
        if data[0..8] != ZOTS_MAGIC {
            return Err(Error::InvalidProof("Invalid magic header".into()));
        }

        let mut offset = 8;

        // Version
        if data.len() < offset + 1 {
            return Err(Error::InvalidProof("Truncated at version".into()));
        }
        let version = data[offset];
        if version != PROOF_VERSION {
            return Err(Error::InvalidProof(format!(
                "Unsupported version: {}",
                version
            )));
        }
        offset += 1;

        // Hash type
        if data.len() < offset + 1 {
            return Err(Error::InvalidProof("Truncated at hash type".into()));
        }
        let hash_type = data[offset];
        if hash_type != HASH_TYPE_SHA256 {
            return Err(Error::InvalidProof(format!(
                "Unsupported hash type: {}",
                hash_type
            )));
        }
        offset += 1;

        // Hash (32 bytes)
        if data.len() < offset + 32 {
            return Err(Error::InvalidProof("Truncated at hash".into()));
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&data[offset..offset + 32]);
        offset += 32;

        // Number of attestations
        if data.len() < offset + 1 {
            return Err(Error::InvalidProof("Truncated at attestation count".into()));
        }
        let num_attestations = data[offset] as usize;
        offset += 1;

        // Parse attestations
        let mut attestations = Vec::with_capacity(num_attestations);
        for i in 0..num_attestations {
            // Each attestation: type(1) + network(1) + txid(32) + height(4) + time(4) + offset(2) = 44 bytes
            if data.len() < offset + 44 {
                return Err(Error::InvalidProof(format!(
                    "Truncated at attestation {}",
                    i
                )));
            }

            // Type
            let att_type = data[offset];
            if att_type != ATTESTATION_TYPE_ZCASH {
                return Err(Error::InvalidProof(format!(
                    "Unknown attestation type: {}",
                    att_type
                )));
            }
            offset += 1;

            // Network
            let network = Network::from_byte(data[offset])?;
            offset += 1;

            // TXID
            let mut txid = [0u8; 32];
            txid.copy_from_slice(&data[offset..offset + 32]);
            offset += 32;

            // Block height
            let block_height = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            // Block time
            let block_time = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            // Memo offset
            let memo_offset = u16::from_le_bytes([data[offset], data[offset + 1]]);
            offset += 2;

            attestations.push(ZcashAttestation {
                network,
                txid,
                block_height,
                block_time,
                memo_offset,
            });
        }

        Ok(Self {
            version,
            hash,
            attestations,
        })
    }

    /// Save the proof to a file
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let data = self.serialize();
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Load a proof from a file
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let data = std::fs::read(path)?;
        Self::deserialize(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_roundtrip() {
        assert_eq!(
            Network::from_byte(Network::Mainnet.to_byte()).unwrap(),
            Network::Mainnet
        );
        assert_eq!(
            Network::from_byte(Network::Testnet.to_byte()).unwrap(),
            Network::Testnet
        );
    }

    #[test]
    fn test_network_invalid() {
        assert!(Network::from_byte(0xFF).is_err());
    }

    #[test]
    fn test_attestation_txid_hex() {
        let att = ZcashAttestation {
            network: Network::Testnet,
            txid: [0xAB; 32],
            block_height: 100,
            block_time: 1700000000,
            memo_offset: 0,
        };
        let hex = att.txid_hex();
        // All 0xAB bytes reversed is still all 0xAB
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c == 'a' || c == 'b'));
    }

    #[test]
    fn test_proof_new() {
        let hash = [0x42u8; 32];
        let proof = TimestampProof::new(hash);
        assert_eq!(proof.version, PROOF_VERSION);
        assert_eq!(proof.hash, hash);
        assert!(proof.attestations.is_empty());
        assert!(!proof.is_confirmed());
    }

    #[test]
    fn test_proof_roundtrip() {
        let hash = [0xABu8; 32];
        let mut proof = TimestampProof::new(hash);

        proof.add_attestation(ZcashAttestation {
            network: Network::Testnet,
            txid: [0xCDu8; 32],
            block_height: 3721456,
            block_time: 1734567890,
            memo_offset: 8,
        });

        assert!(proof.is_confirmed());

        let serialized = proof.serialize();
        let deserialized = TimestampProof::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.version, proof.version);
        assert_eq!(deserialized.hash, hash);
        assert_eq!(deserialized.attestations.len(), 1);
        assert_eq!(deserialized.attestations[0].network, Network::Testnet);
        assert_eq!(deserialized.attestations[0].block_height, 3721456);
        assert_eq!(deserialized.attestations[0].block_time, 1734567890);
        assert_eq!(deserialized.attestations[0].memo_offset, 8);
    }

    #[test]
    fn test_proof_multiple_attestations() {
        let hash = [0x00u8; 32];
        let mut proof = TimestampProof::new(hash);

        proof.add_attestation(ZcashAttestation {
            network: Network::Testnet,
            txid: [0x01u8; 32],
            block_height: 100,
            block_time: 1000,
            memo_offset: 0,
        });

        proof.add_attestation(ZcashAttestation {
            network: Network::Mainnet,
            txid: [0x02u8; 32],
            block_height: 200,
            block_time: 2000,
            memo_offset: 0,
        });

        let serialized = proof.serialize();
        let deserialized = TimestampProof::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.attestations.len(), 2);
        assert_eq!(deserialized.attestations[0].network, Network::Testnet);
        assert_eq!(deserialized.attestations[1].network, Network::Mainnet);
    }

    #[test]
    fn test_proof_invalid_magic() {
        let data = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(TimestampProof::deserialize(&data).is_err());
    }

    #[test]
    fn test_proof_too_small() {
        let data = vec![0x00, 0x7A, 0x4F];
        assert!(TimestampProof::deserialize(&data).is_err());
    }

    #[test]
    fn test_proof_file_roundtrip() {
        let hash = [0x55u8; 32];
        let mut proof = TimestampProof::new(hash);
        proof.add_attestation(ZcashAttestation {
            network: Network::Testnet,
            txid: [0x66u8; 32],
            block_height: 12345,
            block_time: 1700000000,
            memo_offset: 0,
        });

        let temp_path = std::env::temp_dir().join("test_proof.zots");
        proof.save(&temp_path).unwrap();

        let loaded = TimestampProof::load(&temp_path).unwrap();
        assert_eq!(loaded.hash, hash);
        assert_eq!(loaded.attestations.len(), 1);

        // Cleanup
        std::fs::remove_file(&temp_path).ok();
    }
}
