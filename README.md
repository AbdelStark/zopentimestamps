# zOpenTimestamps

A Zcash blockchain timestamping tool with a cypherpunk TUI.

Timestamp any file or hash on the Zcash blockchain to create cryptographic proof that the data existed at a specific point in time.

## Features

- **Timestamp files or hashes** on the Zcash blockchain
- **Verify timestamps** with .zots proof files
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

Timestamp proofs are stored in a binary format with:
- Magic header: `ZOTS`
- Version: 1
- SHA-256 hash (32 bytes)
- Attestations (network, txid, block height, block time)

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
