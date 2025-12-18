<h1 align="center">
  <br>
  <img src="public/ikki-logo.png" alt="Ikki" width="120">
  <br>
  Ikki
  <br>
</h1>

<h4 align="center">A beautiful, privacy-first Zcash wallet for everyone.</h4>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#development">Development</a> •
  <a href="#architecture">Architecture</a> •
  <a href="#security">Security</a> •
  <a href="#license">License</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Zcash-Testnet-yellow?style=flat-square" alt="Zcash Testnet">
  <img src="https://img.shields.io/badge/Tauri-2.0-blue?style=flat-square" alt="Tauri 2.0">
  <img src="https://img.shields.io/badge/Svelte-5-orange?style=flat-square" alt="Svelte 5">
  <img src="https://img.shields.io/badge/Rust-1.81+-brown?style=flat-square" alt="Rust 1.81+">
</p>

---

## Overview

Ikki is a modern, cross-platform Zcash wallet built with privacy and user experience at its core. It combines a beautiful Svelte-based UI with a powerful Rust backend powered by the official Zcash libraries.

**Current Status:** Testnet only. Do not use with real funds.

## Features

### Core Wallet

- **HD Wallet** - BIP-39 compatible 24-word seed phrases
- **Unified Addresses** - Full support for Orchard and Sapling shielded pools
- **Diversified Addresses** - Generate unlimited unlinkable addresses
- **Fast Sync** - Compact block scanning via lightwalletd

### User Experience

- **Premium Dark Theme** - Carefully crafted monochrome design
- **QR Code Support** - Scan and share addresses easily
- **Transaction History** - Grouped by date with detailed views
- **Real-time Sync** - Background blockchain synchronization

### Privacy

- **Shielded by Default** - All transactions use Orchard/Sapling pools
- **Encrypted Memos** - 512-byte encrypted memo field support
- **No Tracking** - Zero analytics or telemetry

## Installation

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.81+
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/AbdelStark/ikki.git
cd ikki

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Building for Production

```bash
# Build the application
npm run tauri build
```

The built application will be in `src-tauri/target/release/`.

## Development

### Project Structure

```
ikki/
├── src/                    # Svelte frontend
│   ├── lib/
│   │   ├── components/     # Reusable UI components
│   │   ├── stores/         # Svelte stores (state management)
│   │   └── utils/          # Utilities and Tauri bridge
│   └── routes/             # Page components
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri IPC commands
│       ├── wallet/         # Core wallet implementation
│       └── state.rs        # Application state
├── public/                 # Static assets
└── package.json
```

### Frontend (Svelte 5)

The frontend is built with:
- **Svelte 5** - Reactive UI framework
- **TypeScript** - Type-safe JavaScript
- **Vite** - Fast build tool
- **Lucide** - Beautiful icons

Key components:
- `Home.svelte` - Main dashboard with balance and recent activity
- `Send.svelte` - Send ZEC with amount and address validation
- `Receive.svelte` - QR code display and address management
- `History.svelte` - Transaction history with date grouping
- `TransactionDetail.svelte` - Detailed transaction view

### Backend (Rust + Tauri)

The backend is powered by:
- **Tauri 2** - Native app framework
- **zcash_client_backend** - Official Zcash wallet library
- **zcash_client_sqlite** - SQLite-based wallet storage
- **zcash_proofs** - Zero-knowledge proof generation

Key modules:
- `wallet/core.rs` - Wallet initialization, sync, and transactions
- `wallet/config.rs` - Configuration management
- `commands/wallet.rs` - Tauri commands for wallet operations
- `commands/transactions.rs` - Transaction management

### Available Commands

| Command | Description |
|---------|-------------|
| `npm run dev` | Start Vite dev server |
| `npm run build` | Build frontend for production |
| `npm run tauri dev` | Run Tauri in development mode |
| `npm run tauri build` | Build production app |

## Architecture

### State Management

```
┌─────────────────────────────────────────────────────────────┐
│                     Svelte Frontend                          │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────────┐ │
│  │ wallet  │  │   ui    │  │  send   │  │  transaction    │ │
│  │  store  │  │  store  │  │  store  │  │     store       │ │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────────┬────────┘ │
└───────┼────────────┼────────────┼────────────────┼──────────┘
        │            │            │                │
        └────────────┴────────────┴────────────────┘
                              │
                    Tauri IPC Bridge
                              │
┌─────────────────────────────┴───────────────────────────────┐
│                      Rust Backend                            │
│  ┌──────────────────┐     ┌─────────────────────────────┐   │
│  │    AppState      │────▶│        IkkiWallet           │   │
│  │  (Tauri State)   │     │  ┌─────────────────────┐    │   │
│  └──────────────────┘     │  │   zcash_client_*    │    │   │
│                           │  └──────────┬──────────┘    │   │
│                           └─────────────┼───────────────┘   │
└─────────────────────────────────────────┼───────────────────┘
                                          │
                               lightwalletd (gRPC)
                                          │
                               ┌──────────┴──────────┐
                               │   Zcash Network     │
                               │     (Testnet)       │
                               └─────────────────────┘
```

### Data Flow

1. **User Action** → Svelte component triggers store update
2. **Store** → Calls Tauri command via `invoke()`
3. **Tauri IPC** → Deserializes and routes to Rust handler
4. **Command** → Acquires wallet lock, performs operation
5. **Wallet** → Uses zcash_client_* for blockchain operations
6. **Response** → Serialized and sent back to frontend
7. **Store** → Updates state, triggers reactive UI update

### Wallet Storage

```
~/.ikki/
├── wallet.db              # SQLite database (zcash_client_sqlite)
├── wallet_config.json     # Encrypted seed storage
└── wallet.db-wal          # WAL journal (auto-managed)
```

## Security

### Seed Phrase Storage

- Stored locally in `~/.ikki/wallet_config.json`
- Unix file permissions set to `0600` (owner read/write only)
- **Warning:** This is a development setup. Production should use OS keychain.

### Network Security

- All lightwalletd connections use TLS
- Certificate validation via system root CAs

### Best Practices

1. **Backup your seed phrase** - Write it down and store securely
2. **Use testnet only** - Do not use with real funds
3. **Verify addresses** - Always double-check before sending
4. **Keep software updated** - Security patches are important

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Logging level (trace, debug, info, warn, error) |

### Lightwalletd Server

Currently connects to `testnet.zec.rocks:443`. To use a different server, modify `wallet/config.rs`.

## Troubleshooting

### Common Issues

**Sync takes too long**
- Initial sync may take several minutes
- Set an appropriate birthday height when importing

**Transaction pending for too long**
- Zcash blocks are mined ~every 75 seconds
- Wait for at least 10 confirmations

**"Insufficient shielded funds" error**
- Ensure funds are in Orchard/Sapling pools
- Transparent funds need to be shielded first

### Reset Wallet

To completely reset the wallet:

```bash
rm -rf ~/.ikki/
```

Then restart the application and create/import a new wallet.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Zcash Foundation](https://www.zfnd.org/) - For the zcash-client libraries
- [Electric Coin Company](https://electriccoin.co/) - For Zcash development
- [Tauri](https://tauri.app/) - For the amazing app framework
- [Svelte](https://svelte.dev/) - For the reactive UI framework

---

<p align="center">
  Made with ❤️ for privacy
</p>
