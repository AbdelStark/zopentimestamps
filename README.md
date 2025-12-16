# zOpenTimestamps

A Zcash blockchain timestamping tool inspired by [OpenTimestamps](https://opentimestamps.org/).

Timestamp any file or hash on the Zcash blockchain to create cryptographic proof that data existed at a specific point in time. Features privacy-preserving shielded transactions and embeddable proof format.

> **Warning**
> This is experimental software. **Do not use on mainnet with real funds.**
> The code has not been audited. Use only on testnet for development and testing.

## Table of Contents

- [Features](#features)
- [Security Notice](#security-notice)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
- [Proof Formats](#proof-formats)
- [How It Works](#how-it-works)
- [Architecture](#architecture)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

## Features

- **Timestamp files or hashes** - Create blockchain-anchored timestamps for any data
- **On-chain verification** - Verify timestamps directly against the Zcash blockchain
- **Shielded transactions** - Privacy-preserving using Orchard/Sapling protocols
- **Embeddable proofs** - Compact CBOR+Base64 format for photos, screenshots, git commits
- **Human-readable format** - JSON proof files for transparency and interoperability
- **Cypherpunk TUI** - ASCII art terminal interface
- **Testnet support** - Safe development and testing environment

## Security Notice

> **This software is provided for educational and experimental purposes only.**

- **NOT AUDITED**: This code has not undergone security audits
- **TESTNET ONLY**: Do not use on mainnet with real ZEC
- **EXPERIMENTAL**: APIs and formats may change without notice
- **NO WARRANTY**: See [LICENSE](LICENSE) for full terms

If you discover security vulnerabilities, please report them responsibly by opening a GitHub issue or contacting the maintainers directly.

## Installation

### Prerequisites

- Rust 1.85.0+ (2024 edition)
- Git

### Build from Source

```bash
# Clone the repository
git clone https://github.com/AbdelStark/zopentimestamps
cd zopentimestamps

# Build release binary
cargo build --release

# Binary location: target/release/zots
```

### Verify Installation

```bash
./target/release/zots --version
./target/release/zots --help
```

## Quick Start

### 1. Configure Environment

Create a `.env` file or set environment variables:

```bash
# Required: 24-word BIP-39 seed phrase
# Generate one at: https://iancoleman.io/bip39/
export ZOTS_SEED="your twenty four word seed phrase here ..."

# Optional (defaults shown)
export ZOTS_BIRTHDAY_HEIGHT=3717528        # Wallet birthday for faster sync
export ZOTS_LIGHTWALLETD="https://testnet.zec.rocks:443"
export ZOTS_NETWORK=testnet                 # IMPORTANT: Keep as testnet!
export ZOTS_DATA_DIR=~/.zopentimestamps
```

### 2. Get Testnet Funds

```bash
# Show your receiving address
zots wallet address

# Get testnet ZEC from faucet:
# https://faucet.zecpages.com/
```

### 3. Sync and Check Balance

```bash
zots wallet sync
zots wallet balance
```

### 4. Create Your First Timestamp

```bash
# Timestamp a file
zots stamp document.pdf

# View the proof
zots info document.pdf.zots
```

## Usage

### Timestamp a File

```bash
# Basic usage (creates <filename>.zots)
zots stamp document.pdf

# Custom output path
zots stamp document.pdf -o my-proof.zots

# Don't wait for confirmation (creates pending proof)
zots stamp document.pdf --no-wait
```

### Timestamp a Hash

```bash
# SHA-256 hash (64 hex characters)
zots stamp --hash e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

# Git commit SHA-1 (40 hex characters, zero-padded to 64)
zots stamp --hash abc123def456789...
```

### Verify a Timestamp

```bash
# Verify proof against blockchain
zots verify document.pdf.zots

# Also verify file hash matches
zots verify document.pdf.zots -f document.pdf
```

### View Proof Information

```bash
zots info document.pdf.zots
```

### Convert Proof Formats

```bash
# Encode to compact format (for embedding)
zots encode document.pdf.zots

# Decode compact format to JSON
zots decode "zots1o2d2ZXJzaW9u..."

# Save decoded proof to file
zots decode "zots1o2d2ZXJzaW9u..." -o proof.zots
```

### Wallet Commands

```bash
zots wallet sync      # Sync with blockchain
zots wallet balance   # Show balance breakdown
zots wallet address   # Show receiving address
zots wallet info      # Show all wallet information
```

### Interactive TUI

```bash
zots tui
```

**Controls:**
- `S` - Stamp screen
- `V` - Verify screen
- `W` - Wallet screen
- `Q` / `Esc` - Quit/Back

## Proof Formats

### JSON Format (.zots files)

Human-readable JSON for transparency and easy inspection:

```json
{
  "version": 1,
  "hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "attestations": [
    {
      "network": "testnet",
      "txid": "abc123...",
      "block_height": 3739654,
      "block_time": 1734293400,
      "memo_offset": 0
    }
  ]
}
```

**Fields:**
| Field | Description |
|-------|-------------|
| `version` | Proof format version (currently 1) |
| `hash` | SHA-256 hash of timestamped data (hex) |
| `attestations` | List of blockchain attestations |
| `attestations[].network` | "mainnet" or "testnet" |
| `attestations[].txid` | Transaction ID (hex, display order) |
| `attestations[].block_height` | Confirmation block number |
| `attestations[].block_time` | Block Unix timestamp |
| `attestations[].memo_offset` | Memo field offset (usually 0) |

### Compact Format (Embeddable)

For embedding timestamps in files, metadata, or QR codes:

```
zots1o2d2ZXJzaW9uAWRoYXNoeEBhYmNkZWYxMjM0NTY3ODkw...
```

**Structure:**
- Prefix: `zots1` (version identifier)
- Payload: CBOR-encoded proof, Base64url-encoded (no padding)

**Use Cases:**
- **Screenshots**: Embed in EXIF/XMP metadata
- **Photos**: Store in image metadata fields
- **Git commits**: Include as commit message trailer
- **Documents**: Embed in PDF/Office metadata
- **QR codes**: Compact enough for QR encoding

**Example git commit:**
```
feat: implement new feature

Timestamp: zots1o2d2ZXJzaW9uAWRoYXNoeEBhYmNkZWYxMjM0NTY3ODkw...
```

## How It Works

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   1. Hash   │───▶│   2. Memo   │───▶│ 3. Broadcast│───▶│  4. Confirm │
│             │    │             │    │             │    │             │
│ SHA-256 of  │    │ Encode hash │    │ Self-send   │    │ Block time  │
│ your file   │    │ in tx memo  │    │ shielded tx │    │ = timestamp │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                                                                │
                                                                ▼
                                                         ┌─────────────┐
                                                         │  5. Proof   │
                                                         │             │
                                                         │ .zots file  │
                                                         │ with txid   │
                                                         └─────────────┘
```

1. **Hash**: Your file is hashed using SHA-256
2. **Memo**: The hash is encoded in a Zcash shielded transaction memo field
3. **Broadcast**: A self-send transaction preserves your privacy while anchoring the hash
4. **Confirm**: Once mined, the block timestamp provides cryptographic proof of time
5. **Proof**: A `.zots` proof file stores the attestation for later verification

### Verification

During verification, zots:
1. Loads the proof file
2. Fetches the transaction from the blockchain
3. Decrypts the memo field using viewing keys
4. Verifies the memo contains the expected hash
5. Returns the block timestamp as proof of existence

## Architecture

```
zopentimestamps/
├── Cargo.toml              # Workspace configuration
├── LICENSE                 # MIT License
├── README.md               # This file
├── crates/
│   ├── zots-core/          # Core library
│   │   ├── src/
│   │   │   ├── lib.rs      # Public API
│   │   │   ├── hash.rs     # SHA-256 hashing utilities
│   │   │   ├── proof.rs    # Proof types and serialization
│   │   │   └── error.rs    # Error types
│   │   └── Cargo.toml
│   │
│   ├── zots-zcash/         # Zcash integration
│   │   ├── src/
│   │   │   ├── lib.rs      # Public API
│   │   │   ├── wallet.rs   # Wallet operations
│   │   │   ├── config.rs   # Configuration
│   │   │   └── memo.rs     # Memo encoding
│   │   └── Cargo.toml
│   │
│   └── zots-cli/           # CLI application
│       ├── src/
│       │   ├── main.rs     # Entry point
│       │   ├── cli.rs      # Argument parsing
│       │   ├── commands/   # Command implementations
│       │   ├── output.rs   # Terminal output helpers
│       │   └── tui/        # Terminal UI
│       └── Cargo.toml
│
└── docs/                   # Additional documentation
```

### Crate Responsibilities

| Crate | Purpose |
|-------|---------|
| `zots-core` | Hash functions, proof format, serialization |
| `zots-zcash` | Wallet, transactions, lightwalletd integration |
| `zots-cli` | CLI commands, TUI, user interaction |

## Development

### Prerequisites

- Rust 1.85.0+ with 2024 edition support
- A Zcash testnet lightwalletd server (default: testnet.zec.rocks)

### Build

```bash
cargo build          # Debug build
cargo build --release # Release build
```

### Test

```bash
cargo test           # Run all tests
cargo test -p zots-core  # Test specific crate
```

### Lint

```bash
cargo clippy         # Run linter
cargo fmt            # Format code
cargo fmt -- --check # Check formatting
```

### Documentation

```bash
cargo doc --open     # Generate and open docs
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [OpenTimestamps](https://opentimestamps.org/) - Original Bitcoin timestamping protocol
- [Zcash](https://z.cash/) - Privacy-focused cryptocurrency
- [librustzcash](https://github.com/zcash/librustzcash) - Zcash Rust libraries
- [Zooko](https://x.com/zooko/status/1998440166244102664?s=46) - For the vision of embedded timestamp proofs (see this [X post](https://x.com/zooko/status/1998440166244102664?s=46))

---

**Disclaimer**: This software is experimental and provided "as is" without warranty. Do not use with real funds. Always verify proofs independently before relying on them for any purpose.
