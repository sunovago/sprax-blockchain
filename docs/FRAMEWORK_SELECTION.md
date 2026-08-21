# SPRX Protocol — Blockchain Framework Selection & ADR-001

## Executive Summary
This document specifies the architectural decision record (ADR-001) for the SPRX Protocol core architecture. After evaluating Cosmos SDK, Substrate (Polkadot), Solana SVM, and Custom Modular Rust architectures, SPRX selected a **Modular Pure-Rust L1 Chain Architecture** combining a custom CometBFT-inspired consensus engine with sandboxed CosmWasm-compatible smart contracts and embedded persistent key-value state commitments.

---

## Comparative Decision Matrix

| Dimension | Cosmos SDK (Go) | Substrate (Rust) | Custom Modular Rust (Selected) | EVM Rollup |
|:---|:---|:---|:---|:---|
| **Language & Toolchain** | Go | Rust / Wasm | Pure Rust (2021 edition) | Solidity / Go / Rust |
| **Deterministic Consensus** | CometBFT (BFT-PoS) | GRANDPA + BABE | CometBFT 2-Step BFT-PoS | Sequencer + Fraud/ZK |
| **Sub-second Finality** | Yes (1-2s) | Probabilistic (BABE) | Deterministic Single-Slot | Dependent on L1 settlement |
| **Contract Execution** | CosmWasm | ink! / EVM | Sandboxed CosmWasm Engine | EVM Bytecode |
| **Storage Architecture** | IAVL / SMT | TrieDB | Embedded LSM / redb KV | Merkle Patricia Trie |
| **Cryptography** | Ed25519, Secp256k1 | Sr25519, Ed25519 | Ed25519 + Secp256k1 Native | ECDSA Secp256k1 |
| **Zero-dependency Build** | Requires Go toolchain | Heavy Wasm runtime | Standard `cargo` build | Complex stack |

---

## Architectural Decision Record (ADR-001)

### Status
**APPROVED & IMPLEMENTED**

### Context
SPRX requires:
1. High throughput (10,000+ TPS target) with sub-second deterministic finality.
2. Complete memory safety and fearless concurrency in Rust.
3. Decoupled microservices architecture: Rust chain core, high-performance JSON-RPC 2.0 API, Python FastAPI backend, and multi-platform TypeScript/Dart wallets.
4. Native support for both Ed25519 (high-throughput low-latency) and Secp256k1 (hardware wallet & cross-chain compatibility).

### Consequences
- **Positive**: Clean modular Rust codebase split into clear domain crates (`sprax-types`, `sprax-crypto`, `sprax-storage`, `sprax-consensus`, `sprax-core`, `sprax-node`, `sprax-wasm`, `sprax-faucet`).
- **Positive**: Zero native C/C++ build dependencies required for devnet execution.
- **Maintenance**: In-house maintenance of BFT state machine and JSON-RPC servers.
