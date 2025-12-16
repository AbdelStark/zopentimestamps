# zOpenTimestamps

A Zcash blockchain timestamping tool with a cypherpunk TUI.

Timestamp any file or hash on the Zcash blockchain to create cryptographic proof that the data existed at a specific point in time.

## Features

- **Timestamp files or hashes** on the Zcash blockchain
- **Verify timestamps** with .zots proof files
- **Embeddable proofs** - compact CBOR+Base64 format for photos, screenshots, git commits
- **Cypherpunk TUI** with ASCII art interface
- **Testnet support** for development and testing
- **Shielded transactions** using Orchard/Sapling protocols

## Installation

```bash
# Clone the repository
git clone https://github.com/example/zopentimestamps
cd zopentimestamps

# Build
cargo build --release

# The binary will be at target/release/zots
```

## Configuration

Set environment variables or create a `.env` file:

```bash
# Required: 24-word BIP-39 seed phrase
export ZOTS_SEED="your twenty four word seed phrase here ..."

# Optional (defaults shown)
export ZOTS_BIRTHDAY_HEIGHT=3717528
export ZOTS_LIGHTWALLETD="https://testnet.zec.rocks:443"
export ZOTS_NETWORK=testnet
export ZOTS_DATA_DIR=~/.zopentimestamps
```

## Getting Started

### 1. Get Testnet Funds

First, get your wallet address and fund it with testnet ZEC:

```bash
# Show your receiving address
zots wallet address

# Get testnet funds from the faucet
# https://testnet.zecfaucet.com/
```

### 2. Sync Your Wallet

```bash
zots wallet sync
```

### 3. Check Balance

```bash
zots wallet balance
```

## Usage

### Timestamp a File

```bash
# Timestamp a file (creates <filename>.zots)
zots stamp document.pdf

# Specify output path
zots stamp document.pdf -o proof.zots

# Don't wait for confirmation (creates pending proof)
zots stamp document.pdf --no-wait
```

### Timestamp a Hash

```bash
# Timestamp a SHA-256 hash
zots stamp --hash e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

# Timestamp a git commit (40-char SHA-1, padded to SHA-256)
zots stamp --hash abc123def456...
```

### Verify a Timestamp

```bash
# Verify a proof file
zots verify document.pdf.zots

# Verify against the original file
zots verify document.pdf.zots -f document.pdf
```

### Show Proof Information

```bash
zots info document.pdf.zots
```

### Encode/Decode Compact Format

Convert between JSON (.zots) and compact (embeddable) format:

```bash
# Encode a .zots file to compact format
zots encode document.pdf.zots

# Decode a compact string to JSON
zots decode "zots1o2d2ZXJzaW9u..."

# Save decoded proof to file
zots decode "zots1o2d2ZXJzaW9u..." -o proof.zots
```

### Wallet Commands

```bash
# Sync wallet with blockchain
zots wallet sync

# Show balance
zots wallet balance

# Show receiving address
zots wallet address

# Show full wallet info
zots wallet info
```

### Interactive TUI

Launch the cypherpunk TUI interface:

```bash
zots tui
```

Controls:
- `S` - Stamp screen
- `V` - Verify screen
- `W` - Wallet screen
- `Q` / `Esc` - Quit/Back

## Proof Format (.zots)

Timestamp proofs are stored in human-readable JSON format:

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

Fields:
- `version`: Proof format version (currently 1)
- `hash`: SHA-256 hash of the timestamped data (hex string)
- `attestations`: List of blockchain attestations confirming the timestamp
  - `network`: "mainnet" or "testnet"
  - `txid`: Transaction ID (hex string, display byte order)
  - `block_height`: Block number where the transaction was confirmed
  - `block_time`: Unix timestamp of the block
  - `memo_offset`: Offset in memo field (usually 0)

## Compact Format (Embeddable)

For embedding timestamps in files (EXIF metadata, git commits, QR codes), proofs can be encoded to a compact CBOR+Base64 format:

```
zots1o2d2ZXJzaW9uAWRoYXNoeEBhYmNkZWYxMjM0NTY3ODkw...
```

The format:
- Prefix: `zots1` (version identifier)
- Payload: CBOR-encoded proof, Base64url-encoded (no padding)

Use cases:
- **Screenshots**: Embed in EXIF/XMP metadata
- **Photos**: Store in image metadata
- **Git commits**: Include in commit message trailer
- **Documents**: Embed in PDF metadata

Example git commit with embedded timestamp:
```
Fix authentication bug

Timestamp: zots1o2d2ZXJzaW9uAWRoYXNoeEBhYmNkZWYxMjM0NTY3ODkw...
```

## How It Works

1. **Hash**: Your file is hashed using SHA-256
2. **Memo**: The hash is encoded in a Zcash transaction memo field
3. **Broadcast**: A self-send transaction is broadcast to the network
4. **Confirm**: Once confirmed, the block provides the timestamp
5. **Proof**: A .zots proof file stores the attestation

## Architecture

```
zopentimestamps/
├── crates/
│   ├── zots-core/     # Hash and proof types
│   ├── zots-zcash/    # Wallet and transaction handling
│   └── zots-cli/      # CLI application and TUI
```

## Dependencies

- Rust 1.85.0+ (2024 edition)
- librustzcash (rev 9f47de6)
- Zcash testnet lightwalletd server

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy

# Format code
cargo fmt
```

## License

MIT

## Related

- [OpenTimestamps](https://opentimestamps.org/) - Bitcoin timestamping
- [Zcash](https://z.cash/) - Privacy-focused cryptocurrency
- [librustzcash](https://github.com/zcash/librustzcash) - Zcash Rust libraries
