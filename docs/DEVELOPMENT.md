# SPRX Protocol: Developer Setup & Multi-Node Testnet Guide
**Document Version:** 1.2.0  
**Target:** Local Development, Testing, Multi-Node Operations & Docker Deployment

---

## 1. Prerequisites & Toolchain Setup

The SPRX Protocol Cargo workspace requires a standard **Rust Stable** toolchain ($\ge 1.80.0$).

### 1.1 Install Rust Toolchain
```bash
# Install rustup (Linux / macOS)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# For Windows: Download and run rustup-init.exe from https://rustup.rs
```

### 1.2 Verify Toolchain Components
```bash
rustup default stable
rustup component add rustfmt clippy
rustc --version
cargo --version
```

---

## 2. Workspace Organization

```
sprax-chain/
├── docker/
│   ├── Dockerfile                 # Multi-stage container build definition
│   └── entrypoint.sh              # Container initialization & startup script
├── docker-compose.yml             # 3-Node local testnet orchestration
├── scripts/
│   ├── run_local_testnet.sh       # Bash multi-node launcher
│   └── run_local_testnet.ps1      # PowerShell multi-node launcher
├── .github/workflows/ci.yml       # GitHub Actions CI pipeline
├── Cargo.toml                     # Root Cargo workspace manifest
├── rust-toolchain.toml            # Pinned Rust toolchain specification
├── .rustfmt.toml                  # Workspace code formatting rules
├── crates/
│   ├── sprax-types/               # Domain types, Address, Hash, Amount, Tx, Block
│   ├── sprax-crypto/              # Ed25519, Secp256k1, Blake3, SHA-256, HD Wallet (BIP-39/44)
│   ├── sprax-storage/             # Key-Value traits, in-memory store, StateCommitment
│   ├── sprax-network/             # P2pService, TCP transport, SwarmHub, GossipSub topics
│   ├── sprax-consensus/           # BFT validator set, DWRR proposer election, Evidence
│   ├── sprax-core/                # Genesis loader, TxExecutor, ChainLedger, Gas metering
│   ├── sprax-node/                # Node service lifecycle, Keyring manager, metrics, telemetry
│   └── sprax-cli/                 # Command-line binary (`sprax`)
├── docs/                          # Architectural specifications & documentation
└── crates/sprax-network/tests/    # Multi-node network & P2P integration test suite
```

---

## 3. Build & Test Commands

### 3.1 Run All Unit, Integration, & Network Tests
```bash
# Run the entire workspace test suite (39 tests)
cargo test --workspace --all-targets

# Run tests with live output
cargo test --workspace -- --nocapture
```

### 3.2 Formatting & Linter Enforcement
```bash
# Verify code formatting
cargo fmt --all -- --check

# Run Clippy with zero-warning threshold
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

## 4. Multi-Node Local Testnet (3 Nodes)

You can run an internal 3-node testnet either natively or via Docker Compose.

### 4.1 Native Local Multi-Node Execution
```bash
# Node 1 (Bootstrap Node)
cargo run -p sprax-cli -- init --home .testnet/node1
cargo run -p sprax-cli -- start --home .testnet/node1 --p2p-port 26656 --rpc-port 26657

# Node 2 (Connects to Node 1)
cargo run -p sprax-cli -- init --home .testnet/node2
cp .testnet/node1/genesis.json .testnet/node2/genesis.json
cargo run -p sprax-cli -- start --home .testnet/node2 --p2p-port 26666 --rpc-port 26667 --peers 127.0.0.1:26656

# Node 3 (Connects to Node 1 & Node 2)
cargo run -p sprax-cli -- init --home .testnet/node3
cp .testnet/node1/genesis.json .testnet/node3/genesis.json
cargo run -p sprax-cli -- start --home .testnet/node3 --p2p-port 26676 --rpc-port 26677 --peers 127.0.0.1:26656,127.0.0.1:26666
```

Or run the automated helper script:
```bash
# Linux / macOS
./scripts/run_local_testnet.sh

# Windows PowerShell
.\scripts\run_local_testnet.ps1
```

### 4.2 Docker Compose Multi-Node Deployment
```bash
# Launch the 3-node testnet in detached mode
docker compose up -d --build

# View real-time cluster logs
docker compose logs -f

# Check container status
docker compose ps

# Shutdown the testnet
docker compose down -v
```

---

## 5. Development Metrics & Node Queries

### 5.1 Query Real-Time Node Metrics
```bash
cargo run -p sprax-cli -- query metrics --home .testnet/node1
```
Output:
```
============================================================
  SPRX NODE DEVELOPMENT METRICS
============================================================
  Chain ID           : sprax-devnet-1
  Node Status        : online
  Current Height     : #1
  Latest Block Hash  : 0x3af21689...
  State Root         : 0x7c7bd5e7...
  Connected Peers    : 2
  Total Transactions : 1
  Mempool Pending    : 0
  Syncing Status     : SYNCHRONIZED
============================================================
```

### 5.2 Query Connected Peers
```bash
cargo run -p sprax-cli -- query peers --home .testnet/node1
```

### 5.3 Submit Cross-Node Transactions
```bash
# Submit transaction on Node 1 -> gossips to Node 2 and Node 3
cargo run -p sprax-cli -- tx send --from alice --to bob --amount 500 --home .testnet/node1

# Verify synchronized state on Node 2
cargo run -p sprax-cli -- query balance bob --home .testnet/node2
```

---

## 6. Security Guidelines

- **Internal Development Only**: Do not expose P2P or RPC ports publicly without TLS and authentication.
- **Zero Secrets**: Never commit real private keys or genesis files with private credentials.
- **Unsafe Code Forbidden**: `#![deny(unsafe_code)]` enforced workspace-wide.
