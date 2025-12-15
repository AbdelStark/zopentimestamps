# zOpenTimestamps - AI Agent Implementation Prompt (Rust CLI)

## 🎯 Mission

You are an autonomous coding agent tasked with building **zOpenTimestamps**, a Rust CLI tool for Zcash blockchain timestamping with a cypherpunk-aesthetic TUI. You will implement the complete MVP from scratch, working in phases, committing progressively, and tracking your progress.

---

## 📋 Context Files

Before starting, read these files thoroughly:

- **PRD:** `docs/prd.md` - Product requirements, user stories, technical architecture
- **Implementation Plan:** `docs/plan.md` - Detailed implementation phases, code structure, milestones
- **Progress Tracker:** `docs/status.md` - Update this file after each phase completion

---

## 🏗️ Project Structure

Create this Rust workspace structure:

```
zopentimestamps/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── zots-core/               # Core library (hashing, proof format)
│   ├── zots-zcash/              # Zcash integration (wallet, tx)
│   └── zots-cli/                # CLI application with TUI
├── docs/
│   ├── prd.md
│   ├── plan.md
│   └── status.md
├── .env.example
├── .gitignore
└── README.md
```

---

## ⚙️ Technical Requirements

### Core Library (`zots-core`)

Must provide:
- `hash_file(path) -> Result<[u8; 32]>` - SHA-256 hash a file
- `hash_bytes(data) -> [u8; 32]` - SHA-256 hash raw bytes
- `hash_from_hex(hex) -> Result<[u8; 32]>` - Parse hex string (git commit or full hash)
- `TimestampProof` struct with serialize/deserialize for .zots format
- `ZcashAttestation` struct for blockchain attestations

### Zcash Module (`zots-zcash`)

Must provide:
- `ZcashConfig::from_env()` - Load config from environment
- `ZotsWallet::new(config)` - Initialize wallet from seed
- `wallet.sync()` - Sync with lightwalletd
- `wallet.get_balance()` - Query balance
- `wallet.get_address()` - Get receiving address
- `wallet.create_timestamp_tx(hash)` - Create and broadcast timestamp TX
- `create_timestamp_memo(hash)` - Encode hash in memo field

### CLI Application (`zots-cli`)

Commands:
```
zots stamp <file>              # Timestamp a file
zots stamp --hash <hex>        # Timestamp a hash
zots verify <proof.zots>       # Verify a proof
zots verify <proof> --file <f> # Verify with original file
zots info <proof.zots>         # Display proof details
zots wallet sync               # Sync wallet
zots wallet balance            # Show balance
zots wallet address            # Show address
zots tui                       # Launch interactive TUI
```

### TUI Features

- ASCII art header (cypherpunk aesthetic)
- Menu: [S]tamp, [V]erify, [W]allet, [Q]uit
- Status bar: sync status, block height, balance
- Keyboard navigation
- Input fields for file paths and hashes

---

## 📦 Dependencies

### Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = ["crates/zots-core", "crates/zots-zcash", "crates/zots-cli"]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85.0"

[workspace.dependencies]
# Core
sha2 = "0.10"
hex = "0.4"
thiserror = "2.0"
anyhow = "1.0"
tokio = { version = "1", features = ["full"] }

# Zcash (pinned to working revision)
zcash_client_backend = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6", features = ["lightwalletd-tonic-transport", "transparent-inputs"] }
zcash_client_sqlite = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6", features = ["transparent-inputs", "unstable"] }
zcash_protocol = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6" }
zcash_address = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6" }
zcash_proofs = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6", features = ["bundled-prover"] }
zcash_keys = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6", features = ["orchard", "sapling", "transparent-inputs"] }
zcash_transparent = { git = "https://github.com/zcash/librustzcash.git", rev = "9f47de6" }

# gRPC
tonic = { version = "0.14", features = ["tls-native-roots"] }
prost = "0.14"
rustls = { version = "0.23", features = ["aws-lc-rs"] }

# CLI/TUI
clap = { version = "4.5", features = ["derive", "env"] }
ratatui = "0.29"
crossterm = "0.28"
colored = "2.1"
indicatif = "0.17"

# Crypto
bip0039 = "0.12"
rand = "0.8"
rand_core = "0.6"

# Storage & other
rusqlite = { version = "0.37", features = ["bundled"] }
dotenvy = "0.15"
chrono = "0.4"
time = "0.3"
dirs = "5.0"
```

---

## 🔧 Environment Configuration

**.env.example:**
```bash
# Required
ZOTS_SEED="your 24 word seed phrase here"

# Optional (defaults shown)
ZOTS_BIRTHDAY_HEIGHT=3717528
ZOTS_LIGHTWALLETD="https://testnet.zec.rocks:443"
ZOTS_NETWORK="testnet"
ZOTS_DATA_DIR="~/.zopentimestamps"
```

---

## 🔄 Implementation Phases

Execute these phases in order. After each phase:
1. Ensure `cargo build` succeeds
2. Run `cargo test` for relevant crates
3. Commit with descriptive message
4. Push to remote
5. Update `docs/status.md`

### Phase 1: Project Setup & Core Types
**Goal:** Initialize Cargo workspace, create crate structure, define error types

Tasks:
- [ ] Create root Cargo.toml with workspace
- [ ] Create crates/zots-core/Cargo.toml
- [ ] Implement src/lib.rs, src/error.rs
- [ ] Define Result and Error types
- [ ] Commit: "feat: initialize rust workspace and core types"

### Phase 2: SHA-256 Hashing Module
**Goal:** Implement hashing for files, bytes, and hex strings

Tasks:
- [ ] Implement crates/zots-core/src/hash.rs
- [ ] hash_bytes, hash_file (streaming), hash_from_hex
- [ ] hash_to_hex helper
- [ ] Unit tests
- [ ] Commit: "feat: implement SHA-256 hashing module"

### Phase 3: Proof Format (.zots)
**Goal:** Binary serialization for timestamp proofs

Tasks:
- [ ] Implement crates/zots-core/src/proof.rs
- [ ] Define ZOTS_MAGIC, Network enum
- [ ] TimestampProof and ZcashAttestation structs
- [ ] serialize() and deserialize() methods
- [ ] save() and load() file helpers
- [ ] Round-trip tests
- [ ] Commit: "feat: implement .zots proof format"

### Phase 4: Zcash Wallet Module
**Goal:** Wallet initialization, sync, and transaction creation

Tasks:
- [ ] Create crates/zots-zcash/Cargo.toml
- [ ] Implement src/config.rs (from_env)
- [ ] Implement src/wallet.rs (ZotsWallet struct)
- [ ] init_account, sync, get_balance, get_address
- [ ] Implement src/memo.rs (create_timestamp_memo)
- [ ] create_timestamp_tx method
- [ ] Commit: "feat: implement zcash wallet module"

### Phase 5: CLI Application Setup
**Goal:** Clap argument parsing and main structure

Tasks:
- [ ] Create crates/zots-cli/Cargo.toml
- [ ] Implement src/cli.rs with clap derive
- [ ] Define Commands enum (Stamp, Verify, Info, Wallet, Tui)
- [ ] Implement src/main.rs entry point
- [ ] Implement src/output.rs (colored helpers)
- [ ] Commit: "feat: implement CLI application structure"

### Phase 6: CLI Commands Implementation
**Goal:** All CLI commands working

Tasks:
- [ ] Implement src/commands/stamp.rs
- [ ] Implement src/commands/verify.rs
- [ ] Implement src/commands/info.rs
- [ ] Implement src/commands/wallet.rs (sync, balance, address, info)
- [ ] Progress indicators with indicatif
- [ ] Error handling and user-friendly output
- [ ] Commit: "feat: implement CLI commands"

### Phase 7: TUI Implementation
**Goal:** Interactive terminal UI with ratatui

Tasks:
- [ ] Implement src/tui/mod.rs (main loop)
- [ ] Implement src/tui/app.rs (App state)
- [ ] Implement src/tui/ui.rs (rendering)
- [ ] ASCII art header
- [ ] Menu screen with keyboard nav
- [ ] Stamp, Verify, Wallet screens
- [ ] Status bar (block height, balance)
- [ ] Commit: "feat: implement cypherpunk TUI"

### Phase 8: Integration & Polish
**Goal:** Complete MVP, documentation, final testing

Tasks:
- [ ] Create .env.example
- [ ] Create .gitignore
- [ ] Create README.md with usage
- [ ] End-to-end testing on testnet
- [ ] Error handling improvements
- [ ] Code cleanup
- [ ] Commit: "feat: complete MVP with documentation"

---

## 📝 Status Tracking

Update `docs/status.md` after each phase with this format:

```markdown
# zOpenTimestamps - Implementation Status

## Current Phase: [Phase Number]

## Completed Phases

### Phase 1: Project Setup & Core Types
- Status: ✅ Complete
- Commit: abc123
- Notes: [Any relevant notes]

## In Progress

### Phase 2: SHA-256 Hashing
- Status: 🔄 In Progress
- Current Task: Implementing hash_file
- Blockers: None

## Pending Phases
- Phase 3: Proof Format
- Phase 4: Zcash Wallet
...

## Known Issues
- [List any discovered issues]

## Build Status
- cargo build: ✅ / ❌
- cargo test: ✅ / ❌
- cargo clippy: ✅ / ❌
```

---

## 🚨 Important Guidelines

### Code Quality
- Use Rust 2024 edition idioms
- Run `cargo clippy` and fix warnings
- Run `cargo fmt` before commits
- Meaningful variable and function names
- Document public APIs with `///` comments
- Handle all Results properly (no unwrap in library code)

### Git Workflow
- One commit per phase (can have WIP commits, squash before push)
- Conventional commits: `feat:`, `fix:`, `docs:`, `test:`
- Push after each phase completion

### Error Handling
- Use `thiserror` for error types
- Use `anyhow` in CLI for error propagation
- User-friendly error messages
- Never panic in library code

### Testing
- Unit tests for pure functions in zots-core
- Integration tests for wallet operations (can mock network)
- Manual E2E testing with testnet

### Security
- Seed phrase only from environment (never hardcoded)
- Don't log sensitive data
- Check file permissions on wallet.db

---

## 🔗 Reference Code

The implementation should reference this working Zcash wallet code pattern:

```rust
// Example from zcash_batch_pay - wallet initialization
use bip0039::{English, Mnemonic};
use zcash_client_backend::data_api::{AccountBirthday, AccountPurpose, WalletWrite};
use zcash_client_backend::keys::UnifiedSpendingKey;
use zcash_client_sqlite::WalletDb;
use zcash_protocol::consensus::TEST_NETWORK;
use zip32::AccountId;

let mnemonic = Mnemonic::<English>::from_phrase(seed_phrase)?;
let seed = mnemonic.to_seed("");
let usk = UnifiedSpendingKey::from_seed(&TEST_NETWORK, &seed, AccountId::ZERO)?;
let ufvk = usk.to_unified_full_viewing_key();

// Import account
db.import_account_ufvk("default", &ufvk, &birthday, AccountPurpose::Spending { derivation: None }, None)?;
```

---

## 🎬 Starting the Implementation

1. Read `docs/prd.md` thoroughly
2. Read `docs/plan.md` thoroughly  
3. Create `docs/status.md` with initial state
4. Begin Phase 1
5. Work autonomously through all phases
6. Update status after each phase
7. Commit and push progressively

**Goal:** Complete MVP where a user can:
1. Run `zots stamp myfile.txt` and get a .zots proof
2. Run `zots verify myfile.txt.zots` and see blockchain confirmation
3. Run `zots tui` and use the interactive interface
4. Manage wallet with `zots wallet` commands

---

## 🔗 Reference Links

- rust-opentimestamps: https://github.com/opentimestamps/rust-opentimestamps
- librustzcash: https://github.com/zcash/librustzcash
- ratatui: https://ratatui.rs/
- Zcash Testnet Faucet: https://testnet.zecfaucet.com/
- Zcash Testnet Explorer: https://testnet.zcashexplorer.app/

---

## ✅ Success Criteria

MVP is complete when:
- [ ] `cargo build --release` succeeds
- [ ] `cargo test` passes
- [ ] `zots stamp <file>` creates timestamp on testnet
- [ ] `zots verify <proof>` validates proof
- [ ] `zots tui` launches interactive interface
- [ ] `zots wallet balance` shows correct balance
- [ ] All code committed and pushed
- [ ] `docs/status.md` shows all phases complete

---

*Now begin implementation. Work autonomously. Ask questions only if blocked by ambiguous requirements.*
