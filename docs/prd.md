# zOpenTimestamps - Product Requirements Document

## Executive Summary

zOpenTimestamps is a decentralized timestamping CLI tool built on Zcash blockchain, providing cryptographic proof that a document, file, or data existed at a specific point in time. Built in Rust with a cypherpunk-aesthetic TUI, it leverages Zcash's privacy features and the robust librustzcash ecosystem.

**Vision Statement:** A command-line tool for cypherpunks to create unforgeable, privacy-preserving proofs of existence using Zcash's blockchain.

---

## Problem Statement

### Current Challenges

1. **Centralized Trust:** Traditional timestamping services require trusting a third party.
2. **Privacy Leakage:** Bitcoin-based OpenTimestamps exposes transaction patterns.
3. **No Zcash Alternative:** The Zcash ecosystem lacks a native timestamping solution.
4. **Developer Tooling Gap:** No CLI tool for automated timestamping in CI/CD pipelines.

### Target Users

- **Developers:** Timestamping git commits, releases, and artifacts via CLI/scripts
- **Cypherpunks:** Privacy-focused proof of existence
- **Researchers:** Timestamping sensitive documents
- **Automation:** CI/CD integration for release verification

---

## Product Goals

### Primary Goals (MVP)

1. **Rust CLI:** Fast, reliable command-line tool with subcommands
2. **TUI Mode:** Interactive terminal UI with cypherpunk aesthetic
3. **Zcash Integration:** Native testnet support using librustzcash
4. **OpenTimestamps Compatibility:** Leverage rust-opentimestamps where applicable

### Secondary Goals (Post-MVP)

1. Mainnet support
2. Shielded transactions for enhanced privacy
3. Git hooks integration
4. Calendar server support
5. Batch timestamping

---

## Technical Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     zOpenTimestamps CLI                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    CLI Interface                         │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────┐    │   │
│  │  │   stamp     │ │   verify    │ │      info       │    │   │
│  │  │  <file>     │ │  <proof>    │ │   <proof>       │    │   │
│  │  │  <hash>     │ │  [--file]   │ │                 │    │   │
│  │  └─────────────┘ └─────────────┘ └─────────────────┘    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   TUI Mode (ratatui)                     │   │
│  │  ┌─────────────────────────────────────────────────┐    │   │
│  │  │  ╔═══════════════════════════════════════════╗  │    │   │
│  │  │  ║  z O P E N T I M E S T A M P S            ║  │    │   │
│  │  │  ╠═══════════════════════════════════════════╣  │    │   │
│  │  │  ║  [S]tamp  [V]erify  [W]allet  [Q]uit      ║  │    │   │
│  │  │  ╠═══════════════════════════════════════════╣  │    │   │
│  │  │  ║  Status: Synced | Block: 3,721,456        ║  │    │   │
│  │  │  ║  Balance: 0.001 TAZ                       ║  │    │   │
│  │  │  ╚═══════════════════════════════════════════╝  │    │   │
│  │  └─────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Core Library                          │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │   │
│  │  │  Hasher  │ │ Timestamp│ │ Verifier │ │  Proof   │    │   │
│  │  │ (SHA256) │ │ Creator  │ │          │ │ (.zots)  │    │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  Zcash Module                            │   │
│  │  ┌──────────────────┐  ┌────────────────────────────┐   │   │
│  │  │  Wallet Manager  │  │   Transaction Builder      │   │   │
│  │  │  (zcash_client_  │  │   (memo field encoding)    │   │   │
│  │  │   sqlite)        │  │                            │   │   │
│  │  └──────────────────┘  └────────────────────────────┘   │   │
│  │  ┌──────────────────┐  ┌────────────────────────────┐   │   │
│  │  │  Lightwalletd    │  │   Proof Generation         │   │   │
│  │  │  Client (gRPC)   │  │   (LocalTxProver)          │   │   │
│  │  └──────────────────┘  └────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
└──────────────────────────────┼──────────────────────────────────┘
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Zcash Testnet                                │
│  lightwalletd: testnet.zec.rocks:443                           │
│  explorer: testnet.zcashexplorer.app                           │
└─────────────────────────────────────────────────────────────────┘
```

### Timestamp Format (.zots)

Binary format for proof files:

```
ZOTS_MAGIC_HEADER (8 bytes): 0x00 0x7A 0x4F 0x54 0x53 0x00 0x00 0x01
                              (\x00 z  O  T  S \x00 \x00 version)

VERSION: 1 (1 byte)
HASH_TYPE: 0x00 = SHA256 (1 byte)
FILE_HASH: 32 bytes
NUM_ATTESTATIONS: 1 byte
ATTESTATIONS: [
  {
    type: 0x01 = ZCASH_TX (1 byte)
    network: 0x00 = mainnet, 0x01 = testnet (1 byte)
    txid: 32 bytes
    block_height: u32 (4 bytes, little-endian)
    block_time: u32 (4 bytes, unix timestamp, little-endian)
    memo_offset: u16 (2 bytes, little-endian)
  }
]
```

### Core Operations

#### 1. Stamping Flow

```
Input (file/hash) → SHA-256 → Memo Encoding → 
  → Zcash Transaction → Broadcast → Wait Confirmation → 
  → Generate .zots Proof File
```

#### 2. Verification Flow

```
.zots File → Parse Attestation → Fetch TX from Chain → 
  → Extract Memo → Verify Hash Match → 
  → Return Result with Block Info
```

---

## Functional Requirements

### FR1: CLI Commands

| Command | Description | Priority |
|---------|-------------|----------|
| `zots stamp <file>` | Timestamp a file | P0 |
| `zots stamp --hash <hex>` | Timestamp a raw hash | P0 |
| `zots verify <proof.zots>` | Verify a proof file | P0 |
| `zots verify <proof.zots> --file <original>` | Verify with original | P0 |
| `zots info <proof.zots>` | Display proof details | P0 |
| `zots wallet sync` | Sync wallet with chain | P0 |
| `zots wallet balance` | Show wallet balance | P0 |
| `zots wallet address` | Show receiving address | P0 |
| `zots tui` | Launch interactive TUI | P1 |
| `zots upgrade <old.zots>` | Upgrade pending proof | P2 |

### FR2: TUI Features

| Feature | Description | Priority |
|---------|-------------|----------|
| Main menu | Stamp/Verify/Wallet/Quit | P1 |
| File browser | Navigate and select files | P1 |
| Hash input | Manual hex input mode | P1 |
| Status bar | Sync status, block height, balance | P1 |
| Progress display | TX broadcast & confirmation | P1 |
| Result display | Verification results | P1 |
| ASCII art header | Cypherpunk aesthetic | P1 |

### FR3: Zcash Integration

| Requirement | Description | Priority |
|-------------|-------------|----------|
| Wallet init | Create wallet from seed phrase | P0 |
| Wallet sync | Sync with lightwalletd | P0 |
| TX building | Create memo-bearing transaction | P0 |
| TX broadcast | Send via lightwalletd | P0 |
| TX query | Fetch transaction by txid | P0 |
| Balance check | Query wallet balance | P0 |

### FR4: Configuration

Environment variables (`.env` file):

```bash
# Required
ZOTS_SEED="your 24 word seed phrase here"

# Optional (defaults shown)
ZOTS_BIRTHDAY_HEIGHT=3717528
ZOTS_LIGHTWALLETD="https://testnet.zec.rocks:443"
ZOTS_NETWORK="testnet"  # or "mainnet" (future)
ZOTS_DATA_DIR="~/.zopentimestamps"
```

---

## Non-Functional Requirements

### NFR1: Performance

- File hashing: < 1 second for files up to 100MB
- Wallet sync: Progress indicator, resumable
- TX confirmation: Poll with exponential backoff

### NFR2: Reliability

- Graceful network error handling
- Transaction retry logic
- Wallet data persistence (SQLite)

### NFR3: Security

- Seed phrase in environment only (never logged)
- Wallet.db file permissions check
- No plaintext secret storage

### NFR4: UX

- Clear CLI help text
- Colored output (errors red, success green)
- Progress spinners for long operations
- TUI keyboard navigation

---

## MVP Scope Definition

### In Scope (MVP v0.1.0)

1. **CLI Commands:**
   - `stamp <file>` and `stamp --hash <hex>`
   - `verify <proof.zots>` with optional original file
   - `info <proof.zots>`
   - `wallet sync/balance/address`

2. **TUI Mode:**
   - Basic menu navigation
   - File selection for stamping
   - Verification display
   - Wallet status

3. **Zcash:**
   - Testnet only
   - Orchard pool (shielded)
   - SQLite wallet storage

4. **Proof Format:**
   - .zots binary format
   - Single attestation per proof

### Out of Scope (MVP)

- Mainnet support
- Calendar server aggregation
- Batch timestamping
- Git hooks
- Merkle tree aggregation
- Cross-chain verification

---

## User Stories

### US1: Timestamp a File (CLI)

**As a** developer  
**I want to** timestamp a file from the command line  
**So that** I can prove when my code release existed

**Acceptance Criteria:**
```bash
$ zots stamp ./release-v1.0.tar.gz
Hashing file... done
SHA-256: a1b2c3d4e5f6...
Building transaction...
Broadcasting... done
Waiting for confirmation... 
✓ Confirmed in block 3,721,456
Proof saved: release-v1.0.tar.gz.zots
```

### US2: Timestamp a Git Commit

**As a** developer  
**I want to** timestamp a git commit hash  
**So that** I have blockchain proof of my commit

**Acceptance Criteria:**
```bash
$ zots stamp --hash $(git rev-parse HEAD)
Hash: 7a8b9c0d...
Building transaction...
✓ Confirmed in block 3,721,460
Proof saved: 7a8b9c0d.zots
```

### US3: Verify a Timestamp

**As a** verifier  
**I want to** verify a .zots proof file  
**So that** I can confirm when something was timestamped

**Acceptance Criteria:**
```bash
$ zots verify release-v1.0.tar.gz.zots --file release-v1.0.tar.gz
Parsing proof...
Hash matches original file ✓
Fetching transaction...
✓ VALID TIMESTAMP
  Block: 3,721,456
  Time: 2025-12-15 14:32:05 UTC
  TX: https://testnet.zcashexplorer.app/tx/abc123...
```

### US4: Interactive TUI

**As a** user  
**I want to** use an interactive terminal interface  
**So that** I can timestamp files without memorizing commands

**Acceptance Criteria:**
- Launch with `zots tui`
- Navigate with keyboard
- See wallet status
- Stamp and verify files interactively

---

## Technical Dependencies

### Rust Crates

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `ratatui` | Terminal UI framework |
| `crossterm` | Terminal manipulation |
| `tokio` | Async runtime |
| `zcash_client_backend` | Zcash wallet backend |
| `zcash_client_sqlite` | SQLite wallet storage |
| `zcash_proofs` | Transaction proving |
| `zcash_keys` | Key derivation |
| `zcash_protocol` | Protocol constants |
| `tonic` | gRPC client |
| `sha2` | SHA-256 hashing |
| `bip0039` | Mnemonic handling |
| `dotenvy` | Environment loading |
| `opentimestamps` | OTS format reference |

### External Services

| Service | Endpoint |
|---------|----------|
| Lightwalletd (testnet) | `https://testnet.zec.rocks:443` |
| Block Explorer | `https://testnet.zcashexplorer.app` |
| Faucet | `https://testnet.zecfaucet.com` |

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Lightwalletd downtime | High | Retry logic, fallback endpoints |
| Wallet sync slow | Medium | Progress indicator, background sync |
| Proof format changes | Medium | Version field for compatibility |
| Testnet resets | Low | Document data volatility |

---

## Success Metrics

### MVP Success Criteria

1. ✅ Successfully stamp a file on Zcash testnet
2. ✅ Successfully verify a .zots proof
3. ✅ CLI runs on Linux/macOS/Windows
4. ✅ TUI launches and is navigable
5. ✅ Wallet syncs and shows balance
6. ✅ End-to-end flow completes in < 5 minutes

---

## Glossary

| Term | Definition |
|------|------------|
| **Attestation** | Proof that data was timestamped |
| **Memo Field** | 512-byte field in Zcash transactions |
| **Lightwalletd** | Light client server for Zcash |
| **TAZ** | Testnet ZEC |
| **Orchard** | Latest Zcash shielded pool |
| **.zots** | zOpenTimestamps proof file extension |
| **TUI** | Terminal User Interface |

---

## References

- [OpenTimestamps](https://opentimestamps.org)
- [rust-opentimestamps](https://github.com/opentimestamps/rust-opentimestamps)
- [librustzcash](https://github.com/zcash/librustzcash)
- [Zcash Protocol Spec](https://zips.z.cash)

---

*Document Version: 2.0 (Rust CLI)*  
*Last Updated: December 2025*
