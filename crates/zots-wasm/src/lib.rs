//! WebAssembly bindings for zOpenTimestamps.
//!
//! This crate provides JavaScript-callable functions for:
//! - Hashing files and data (SHA-256, BLAKE3)
//! - Creating and manipulating timestamp proofs
//! - Encoding/decoding proofs to compact format
//!
//! ## Usage from JavaScript
//!
//! ```javascript
//! import init, { hash_bytes, hash_hex, create_proof, encode_proof, decode_proof } from 'zots-wasm';
//!
//! await init();
//!
//! // Hash some data
//! const data = new Uint8Array([1, 2, 3, 4]);
//! const hash = hash_bytes(data, "sha256");
//!
//! // Create a proof
//! const proof = create_proof(hash, "sha256");
//!
//! // Encode to compact format
//! const compact = encode_proof(proof);
//! ```

use wasm_bindgen::prelude::*;
use zots_core::{
    HashAlgorithm, Network, TimestampProof, ZcashAttestation, hash_bytes_with, hash_from_hex_with,
    hash_to_hex,
};

/// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Hash algorithm enum for JavaScript
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum WasmHashAlgorithm {
    Sha256,
    Blake3,
}

impl From<WasmHashAlgorithm> for HashAlgorithm {
    fn from(algo: WasmHashAlgorithm) -> Self {
        match algo {
            WasmHashAlgorithm::Sha256 => HashAlgorithm::Sha256,
            WasmHashAlgorithm::Blake3 => HashAlgorithm::Blake3,
        }
    }
}

/// Hash raw bytes and return hex string
#[wasm_bindgen]
pub fn hash_bytes(data: &[u8], algorithm: WasmHashAlgorithm) -> String {
    let hash = hash_bytes_with(data, algorithm.into());
    hash_to_hex(&hash)
}

/// Hash a hex string (40 or 64 chars) and return hex string
#[wasm_bindgen]
pub fn hash_hex(hex_str: &str, algorithm: WasmHashAlgorithm) -> Result<String, JsError> {
    let hash =
        hash_from_hex_with(hex_str, algorithm.into()).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(hash_to_hex(&hash))
}

/// Create a new timestamp proof from a hash hex string
#[wasm_bindgen]
pub fn create_proof(hash_hex: &str, algorithm: WasmHashAlgorithm) -> Result<JsValue, JsError> {
    let hash_bytes =
        hex::decode(hash_hex).map_err(|e| JsError::new(&format!("Invalid hex: {e}")))?;

    if hash_bytes.len() != 32 {
        return Err(JsError::new("Hash must be 32 bytes (64 hex chars)"));
    }

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hash_bytes);

    let algo: HashAlgorithm = algorithm.into();
    let proof = TimestampProof::new_with_algorithm(hash, algo);

    serde_wasm_bindgen::to_value(&proof).map_err(|e| JsError::new(&e.to_string()))
}

/// Parse a proof from JSON string
#[wasm_bindgen]
pub fn parse_proof(json: &str) -> Result<JsValue, JsError> {
    let proof = TimestampProof::deserialize(json).map_err(|e| JsError::new(&e.to_string()))?;

    serde_wasm_bindgen::to_value(&proof).map_err(|e| JsError::new(&e.to_string()))
}

/// Serialize a proof to JSON string
#[wasm_bindgen]
pub fn serialize_proof(proof: JsValue) -> Result<String, JsError> {
    let proof: TimestampProof =
        serde_wasm_bindgen::from_value(proof).map_err(|e| JsError::new(&e.to_string()))?;

    proof.serialize().map_err(|e| JsError::new(&e.to_string()))
}

/// Encode a proof to compact format (zots1...)
#[wasm_bindgen]
pub fn encode_proof(proof: JsValue) -> Result<String, JsError> {
    let proof: TimestampProof =
        serde_wasm_bindgen::from_value(proof).map_err(|e| JsError::new(&e.to_string()))?;

    proof.to_compact().map_err(|e| JsError::new(&e.to_string()))
}

/// Decode a compact proof string to JSON
#[wasm_bindgen]
pub fn decode_proof(compact: &str) -> Result<JsValue, JsError> {
    let proof = TimestampProof::from_compact(compact).map_err(|e| JsError::new(&e.to_string()))?;

    serde_wasm_bindgen::to_value(&proof).map_err(|e| JsError::new(&e.to_string()))
}

/// Check if a string is a valid compact proof format
#[wasm_bindgen]
pub fn is_compact_format(s: &str) -> bool {
    TimestampProof::is_compact_format(s)
}

/// Get proof information as a formatted string
#[wasm_bindgen]
pub fn get_proof_info(proof: JsValue) -> Result<String, JsError> {
    let proof: TimestampProof =
        serde_wasm_bindgen::from_value(proof).map_err(|e| JsError::new(&e.to_string()))?;

    let mut info = String::new();
    info.push_str(&format!("Version: {}\n", proof.version));
    info.push_str(&format!(
        "Hash Algorithm: {}\n",
        proof.hash_algorithm.name()
    ));
    info.push_str(&format!("Hash: {}\n", proof.hash));
    info.push_str(&format!("Attestations: {}\n", proof.attestations.len()));

    for (i, att) in proof.attestations.iter().enumerate() {
        info.push_str(&format!("\nAttestation #{}:\n", i + 1));
        info.push_str(&format!("  Network: {}\n", att.network));
        info.push_str(&format!("  Block Height: {}\n", att.block_height));
        info.push_str(&format!(
            "  Block Time: {}\n",
            att.timestamp().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        info.push_str(&format!("  TX ID: {}\n", att.txid));
        info.push_str(&format!("  Explorer: {}\n", att.explorer_link()));
    }

    Ok(info)
}

/// Add an attestation to a proof (returns new proof)
#[wasm_bindgen]
pub fn add_attestation(
    proof: JsValue,
    network: &str,
    txid: &str,
    block_height: u32,
    block_time: u32,
    memo_offset: u16,
) -> Result<JsValue, JsError> {
    let mut proof: TimestampProof =
        serde_wasm_bindgen::from_value(proof).map_err(|e| JsError::new(&e.to_string()))?;

    let network = match network.to_lowercase().as_str() {
        "mainnet" | "main" => Network::Mainnet,
        "testnet" | "test" => Network::Testnet,
        _ => return Err(JsError::new("Invalid network: use 'mainnet' or 'testnet'")),
    };

    // Parse txid hex to bytes
    let txid_bytes =
        hex::decode(txid).map_err(|e| JsError::new(&format!("Invalid txid hex: {e}")))?;

    if txid_bytes.len() != 32 {
        return Err(JsError::new("TXID must be 32 bytes (64 hex chars)"));
    }

    // Reverse from display order to internal byte order
    let mut txid_arr = [0u8; 32];
    txid_arr.copy_from_slice(&txid_bytes);
    txid_arr.reverse();

    proof.add_attestation(ZcashAttestation::new(
        network,
        txid_arr,
        block_height,
        block_time,
        memo_offset,
    ));

    serde_wasm_bindgen::to_value(&proof).map_err(|e| JsError::new(&e.to_string()))
}

/// Get the hash from a proof
#[wasm_bindgen]
pub fn get_proof_hash(proof: JsValue) -> Result<String, JsError> {
    let proof: TimestampProof =
        serde_wasm_bindgen::from_value(proof).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(proof.hash)
}

/// Get the algorithm from a proof
#[wasm_bindgen]
pub fn get_proof_algorithm(proof: JsValue) -> Result<String, JsError> {
    let proof: TimestampProof =
        serde_wasm_bindgen::from_value(proof).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(proof.hash_algorithm.name().to_string())
}

/// Get attestation count from a proof
#[wasm_bindgen]
pub fn get_attestation_count(proof: JsValue) -> Result<usize, JsError> {
    let proof: TimestampProof =
        serde_wasm_bindgen::from_value(proof).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(proof.attestations.len())
}

/// Check if proof has attestations (is confirmed)
#[wasm_bindgen]
pub fn is_proof_confirmed(proof: JsValue) -> Result<bool, JsError> {
    let proof: TimestampProof =
        serde_wasm_bindgen::from_value(proof).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(proof.is_confirmed())
}

/// Get explorer URL for a network
#[wasm_bindgen]
pub fn get_explorer_url(network: &str) -> Result<String, JsError> {
    let net = match network.to_lowercase().as_str() {
        "mainnet" | "main" => Network::Mainnet,
        "testnet" | "test" => Network::Testnet,
        _ => return Err(JsError::new("Invalid network: use 'mainnet' or 'testnet'")),
    };
    Ok(net.default_explorer_url().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_hash_bytes_sha256() {
        let data = b"hello world";
        let hash = hash_bytes(data, WasmHashAlgorithm::Sha256);
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[wasm_bindgen_test]
    fn test_hash_bytes_blake3() {
        let data = b"hello world";
        let hash = hash_bytes(data, WasmHashAlgorithm::Blake3);
        assert!(!hash.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_is_compact_format() {
        assert!(is_compact_format("zots1abc123"));
        assert!(!is_compact_format("invalid"));
    }
}
