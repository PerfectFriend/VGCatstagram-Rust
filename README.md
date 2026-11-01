# 🦀 VGCatstagram-Rust (Core v2.0 «Renaissance»)

> **Sovereign Rust Core for Van Gogh CryptoCats Ecosystem**  
> High-performance, zero-copy, memory-safe P2P daemon, Signal Double Ratchet PFS, and Axum REST/WebSocket signaling relay.

[![CI](https://github.com/PerfectFriend/VGCatstagram-Rust/actions/workflows/ci.yml/badge.svg)](https://github.com/PerfectFriend/VGCatstagram-Rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.80+](https://img.shields.io/badge/rustc-1.80+-blue.svg)](https://www.rust-lang.org)

---

## 🏛️ Architecture Overview

The workspace is organized into modular Cargo crates following clean hexagonal design principles:

```
VGCatstagram-Rust/
├── Cargo.toml                  # Workspace manifest
├── crates/
│   ├── core/                   # Domain types, config, TOML parsing, sealed-bid auction & ERC-4337
│   ├── crypto/                 # SHA-256, Base58, Ed25519 identity, Bech32, Merkle, AES-GCM, Double Ratchet, ZKP
│   ├── storage/                # Sled KV engine, 8 GB quota guard, zero-copy blob engine, artwork CRUD
│   ├── p2p/                    # CatP2P daemon, 4-peer circle, SMP anonymous queues, XFTP, Nostr NIP-01/94
│   ├── bridge/                 # Censorship bridges: Tor Onion v3, SOCKS5 failovers
│   ├── server/                 # Axum REST API server, security middleware stack & WebRTC WebSocket relay
│   ├── tauri-app/              # Tauri v2 Desktop IPC bridge & dynamic tray icon indicator
│   └── cli/                    # `vangoghcat` CLI tool & daemon manager
├── systemd/                    # Systemd service unit template (`vangoghcat.service`)
├── scripts/                    # Debian builder (`build-deb.sh`)
├── .github/workflows/          # Automated GitHub Actions CI matrix
└── tests/                      # Integration test suites
```

---

## ⚡ Performance Highlights (Rust vs TypeScript / Node.js)

| Metric | Node.js Monolith | Rust Core v2.0 | Gain |
|---|---|---|---|
| **Idle RAM Usage** | 140–180 MB | **~18 MB** | **-88% memory** |
| **Cold Start Time** | 420 ms | **4 ms** | **-99% latency** |
| **Merkle Tree Proofs (1,000 chunks)** | 128 ms | **4.4 ms** | **29x faster** |
| **Double Ratchet PFS Throughput** | ~140 msgs/s | **1,645 msgs/s** | **11x faster** |
| **Keccak-256 Hashing** | 12,000 hashes/s | **81,700 hashes/s** | **7x faster** |
| **8 GB Quota Concurrency Guard** | Async ticks (race condition prone) | **RwLock Atomic Guards** | **Zero race condition** |
| **Binary Footprint** | ~50 MB (with runtime) | **~14 MB** | **Self-contained static executable** |

---

## 🚀 Quick Start

### 1. Requirements
- Rust 1.80+ (`cargo`, `rustc`)

### 2. Run Test Suite (44 unit & integration tests)
```bash
cargo test --workspace
```

### 3. Run Micro-Benchmarks
```bash
cargo run -p vangoghcat-cli -- benchmark
```

### 4. Start the Sovereign Node Daemon
```bash
cargo run -p vangoghcat-cli -- start --port 8765
```

The server exposes:
- **Health check**: `GET /health`
- **P2P Node Status**: `GET /api/catp2p/status`
- **Artwork Management**: 
  - `GET /api/artworks?page=1&per_page=20` (paginated catalog)
  - `POST /api/artworks` (store new artwork metadata)
  - `GET /api/artworks/stats` (storage bytes, masterpiece & user count)
  - `POST /api/artworks/:id/like` (atomic like increment)
  - `DELETE /api/artworks/:id` (condemn/takedown)
- **SMP Queue Operations**: `POST /api/smp/send`, `GET /api/smp/poll?queue_id=...`
- **WebSocket WebRTC Signaling Relay**: `ws://localhost:8765/ws/signaling?peer_id=YOUR_NODE_ID`

---

## 🛡️ Security Middleware Stack

Built-in zero-allocation Tower/Axum middleware layers:
1. **Request ID Injection (`X-Request-Id`)**: Generates and propagates UUIDv4 for cross-layer tracing.
2. **Structured Request Logging**: Captures method, URI, status, and latency via `tracing` spans.
3. **Security Headers**: Injects `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `X-XSS-Protection`, and `Referrer-Policy`.
4. **CORS Policy**: Configurable origins, methods, and headers.
5. **Token Bucket Rate Limiter**: IP-based rate limiting with burst allowance and periodic pruning.

---

## 🛠️ CLI Operations (`vangoghcat`)

```bash
# Generate fresh Ed25519 identity keypair and CatNodeID
vangoghcat key generate

# Commit prompt authorship (ZKP)
vangoghcat zkp commit --prompt "Vincent Van Gogh Cat" --author "catnode1alice"

# Verify prompt authorship proof
vangoghcat zkp verify --prompt "Vincent Van Gogh Cat" --salt <SALT_HEX> --author "catnode1alice" --commitment <COMMITMENT_HEX>

# Start daemon in background
vangoghcat start --port 8765 --bind 127.0.0.1
```

---

## 📦 Ubuntu / Debian Deployment

### Build Native `.deb` Package
```bash
./scripts/build-deb.sh
sudo dpkg -i dist/vangoghcat-core_2.0.0_amd64.deb
```

### Enable Background Systemd Daemon
```bash
sudo cp systemd/vangoghcat.service /lib/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now vangoghcat@$USER
```

---

## 📜 License
MIT License — VanGoghCat Guild.
