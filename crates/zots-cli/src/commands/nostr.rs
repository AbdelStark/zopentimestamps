//! Nostr protocol integration for publishing and retrieving timestamp proofs.
//!
//! This module enables sharing zOpenTimestamps proofs via the Nostr protocol,
//! allowing decentralized distribution and discovery of timestamp attestations.
//!
//! ## Configuration
//!
//! Requires the following environment variables:
//! - `ZOTS_NOSTR_NSEC`: Your Nostr secret key (nsec1... or hex format)
//! - `ZOTS_NOSTR_RELAYS`: Comma-separated list of relay URLs
//!
//! ## Commands
//!
//! - `publish`: Publish a proof to Nostr relays
//! - `fetch`: Retrieve a proof from a Nostr event ID

use crate::output::*;
use std::path::PathBuf;
use std::time::Duration;
use zots_core::TimestampProof;

use nostr_sdk::prelude::*;

/// Nostr configuration loaded from environment variables.
pub struct NostrConfig {
    /// Secret key for signing events
    pub keys: Keys,
    /// List of relay URLs
    pub relays: Vec<String>,
}

impl NostrConfig {
    /// Load Nostr configuration from environment variables.
    ///
    /// Required variables:
    /// - `ZOTS_NOSTR_NSEC`: Secret key (nsec1... bech32 or hex format)
    /// - `ZOTS_NOSTR_RELAYS`: Comma-separated relay URLs
    pub fn from_env() -> anyhow::Result<Self> {
        let nsec = std::env::var("ZOTS_NOSTR_NSEC").map_err(|_| {
            anyhow::anyhow!(
                "ZOTS_NOSTR_NSEC environment variable not set.\n\
                 Set your Nostr secret key in .env file:\n\
                 ZOTS_NOSTR_NSEC=\"nsec1...\""
            )
        })?;

        let keys = Keys::parse(&nsec).map_err(|e| {
            anyhow::anyhow!(
                "Invalid ZOTS_NOSTR_NSEC: {e}\n\
                 Provide a valid nsec1... bech32 key or hex secret key"
            )
        })?;

        let relays_str = std::env::var("ZOTS_NOSTR_RELAYS").map_err(|_| {
            anyhow::anyhow!(
                "ZOTS_NOSTR_RELAYS environment variable not set.\n\
                 Set relay URLs in .env file:\n\
                 ZOTS_NOSTR_RELAYS=\"wss://relay.damus.io,wss://nos.lol\""
            )
        })?;

        let relays: Vec<String> = relays_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if relays.is_empty() {
            anyhow::bail!("ZOTS_NOSTR_RELAYS must contain at least one relay URL");
        }

        Ok(Self { keys, relays })
    }
}

/// Custom event kind for zOpenTimestamps proofs (NIP-XX style)
/// Using kind 30078 (Parameterized Replaceable Event) for timestamp proofs
const ZOTS_EVENT_KIND: u16 = 30078;

/// Tag identifier for the compact proof data
const ZOTS_PROOF_TAG: &str = "zots-proof";

/// Tag identifier for the hash being timestamped
const ZOTS_HASH_TAG: &str = "zots-hash";

/// Generate a human-readable description of a timestamp proof.
fn proof_description(proof: &TimestampProof) -> String {
    let mut desc = String::new();

    desc.push_str("⏰ zOpenTimestamps Proof\n\n");

    desc.push_str(&format!(
        "📄 Hash ({}):\n{}\n\n",
        proof.hash_algorithm().name(),
        proof.hash
    ));

    if proof.attestations.is_empty() {
        desc.push_str("⏳ Status: Pending confirmation\n");
    } else {
        desc.push_str(&format!(
            "✅ Attestations: {}\n\n",
            proof.attestations.len()
        ));

        for (i, att) in proof.attestations.iter().enumerate() {
            let timestamp = att.timestamp();
            desc.push_str(&format!("🔗 Attestation #{}\n", i + 1));
            desc.push_str(&format!("   Network: {} (Zcash)\n", att.network));
            desc.push_str(&format!("   Block: {}\n", att.block_height));
            desc.push_str(&format!(
                "   Time: {}\n",
                timestamp.format("%Y-%m-%d %H:%M:%S UTC")
            ));
            desc.push_str(&format!("   TX: {}\n", att.txid));
            desc.push_str(&format!("   Explorer: {}\n\n", att.explorer_link()));
        }
    }

    desc.push_str("🔧 Verify with: zots verify <proof.zots>\n");
    desc.push_str("📚 https://github.com/AbdelStark/zopentimestamps");

    desc
}

/// Publish a timestamp proof to Nostr relays.
///
/// The proof is published as a note containing:
/// - Human-readable description of the timestamp
/// - Compact proof (zots1...) in a tag for machine parsing
/// - Hash tag for discoverability
pub async fn publish(proof_path: PathBuf) -> anyhow::Result<()> {
    print_header("Publishing Proof to Nostr");

    // Load configuration
    let config = NostrConfig::from_env()?;
    print_info("Public Key", &config.keys.public_key().to_bech32()?);

    // Load proof
    print_info("Proof File", &proof_path.display().to_string());
    let proof = TimestampProof::load(&proof_path)?;
    print_hash(&proof.hash, proof.hash_algorithm().name());

    // Convert to compact format
    let compact = proof.to_compact()?;
    print_info("Compact Size", &format!("{} bytes", compact.len()));

    // Generate human-readable description
    let description = proof_description(&proof);

    // Create Nostr client
    let client = Client::new(config.keys);

    // Add relays
    print_status("Connecting to relays...");
    for relay in &config.relays {
        client.add_relay(relay).await?;
        print_info("Relay", relay);
    }

    // Connect to relays
    client.connect().await;

    // Wait a moment for connections to establish
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Build the event with custom tags
    let event = EventBuilder::new(Kind::Custom(ZOTS_EVENT_KIND), &description)
        .tag(Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed(ZOTS_PROOF_TAG)),
            vec![compact.clone()],
        ))
        .tag(Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed(ZOTS_HASH_TAG)),
            vec![proof.hash.clone()],
        ))
        .tag(Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed("d")),
            vec![proof.hash.clone()], // Use hash as identifier for parameterized replaceable
        ));

    // Send the event
    print_status("Publishing event...");
    let output = client.send_event_builder(event).await?;

    println!();
    print_success("Proof published to Nostr!");
    print_info("Event ID", &output.id().to_bech32()?);
    print_info("Event ID (hex)", &output.id().to_hex());

    // Show which relays received it
    let success_count = output.success.len();
    let failed_count = output.failed.len();

    if success_count > 0 {
        print_info("Relays (success)", &success_count.to_string());
    }
    if failed_count > 0 {
        print_warning(&format!("Relays (failed): {failed_count}"));
    }

    println!();
    print_info(
        "Fetch with",
        &format!("zots nostr fetch {}", output.id().to_bech32()?),
    );

    // Disconnect
    client.disconnect().await;

    Ok(())
}

/// Fetch a timestamp proof from a Nostr event.
///
/// Retrieves the event by ID and extracts the compact proof from the tags.
pub async fn fetch(event_id: String, output: Option<PathBuf>) -> anyhow::Result<()> {
    print_header("Fetching Proof from Nostr");

    // Load configuration (we need relays to fetch from)
    let config = NostrConfig::from_env()?;

    // Parse event ID (supports both bech32 note1... and hex format)
    let id = EventId::parse(&event_id).map_err(|e| {
        anyhow::anyhow!(
            "Invalid event ID: {e}\n\
             Provide a valid note1... bech32 ID or hex event ID"
        )
    })?;

    print_info("Event ID", &id.to_bech32()?);

    // Create client (no signing needed for fetching)
    let client = Client::new(config.keys);

    // Add relays
    print_status("Connecting to relays...");
    for relay in &config.relays {
        client.add_relay(relay).await?;
    }

    // Connect
    client.connect().await;
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Create filter for this specific event
    let filter = Filter::new().id(id);

    // Fetch the event
    print_status("Fetching event...");
    let events = client.fetch_events(filter, Duration::from_secs(10)).await?;

    if events.is_empty() {
        print_error("Event not found on any connected relay");
        client.disconnect().await;
        anyhow::bail!("Event not found: {event_id}");
    }

    let event = events.into_iter().next().unwrap();

    // Look for the zots-proof tag
    let mut compact_proof: Option<String> = None;

    for tag in event.tags.iter() {
        let tag_vec: Vec<String> = tag.clone().to_vec();
        if tag_vec.first().map(|s| s.as_str()) == Some(ZOTS_PROOF_TAG) {
            if let Some(proof_data) = tag_vec.get(1) {
                compact_proof = Some(proof_data.clone());
                break;
            }
        }
    }

    let compact = compact_proof.ok_or_else(|| {
        anyhow::anyhow!(
            "Event does not contain a zots-proof tag.\n\
             This event was not created by zots nostr publish."
        )
    })?;

    // Decode the proof
    let proof = TimestampProof::from_compact(&compact)?;

    println!();
    print_success("Proof retrieved from Nostr!");
    print_hash(&proof.hash, proof.hash_algorithm().name());
    print_info("Attestations", &proof.attestations.len().to_string());

    // Show attestation details
    for (i, att) in proof.attestations.iter().enumerate() {
        println!();
        print_info(
            &format!("Attestation #{}", i + 1),
            &format!("{} block {}", att.network, att.block_height),
        );
        print_info("  TX", &att.txid);
        print_info(
            "  Time",
            &att.timestamp().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        );
    }

    // Save or display
    if let Some(output_path) = output {
        proof.save(&output_path)?;
        println!();
        print_success(&format!("Proof saved to: {}", output_path.display()));
    } else {
        println!();
        print_header("Compact Proof");
        println!("{compact}");
        println!();
        print_info("Tip", "Use -o <file.zots> to save the proof to a file");
    }

    // Disconnect
    client.disconnect().await;

    Ok(())
}
