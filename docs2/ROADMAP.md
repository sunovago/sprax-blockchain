# SPRX Protocol: Multi-Phase Engineering Roadmap
**Document Version:** 1.0.0-PROPOSAL  
**Status:** ARCHITECTURE_ONLY (Phase 01)  
**Protocol Lifecycle:** Concept $\rightarrow$ Architecture $\rightarrow$ Core Engine $\rightarrow$ Testnet $\rightarrow$ Audit $\rightarrow$ Mainnet

---

## 1. Roadmap Overview & Timeline

The SPRX Protocol development lifecycle is structured into six disciplined, sequential phases. Each phase establishes deterministic verification criteria before subsequent phases unlock.

```
+---------------------------------------------------------------------------------------------------+
|                                  SPRX MULTI-PHASE ROADMAP                                          |
+---------------------------------------------------------------------------------------------------+
| [Phase 01: Architecture Specification]  <--- COMPLETED (CURRENT PHASE)                           |
|  - Full technical architecture package, threat models, mathematical specs, decision matrix       |
|                                                                                                   |
| [Phase 02: Core Protocol Engine & Local Devnet]                                                   |
|  - Rust core workspace, BFT-PoS consensus, JMT storage, libp2p GossipSub, 4-node local devnet     |
|                                                                                                   |
| [Phase 03: VM Execution, Staking & Governance]                                                    |
|  - Dual Wasm/EVM sandbox runtime, gas metering, staking & slashing modules, on-chain governance   |
|                                                                                                   |
| [Phase 04: Public Incentivized Testnet & Ecosystem]                                               |
|  - Multi-node public testnet, ClickHouse indexer, Web3 block explorer, multi-currency wallet      |
|                                                                                                   |
| [Phase 05: Security Audits & Formal Verification]                                                 |
|  - Tier-1 cryptographic audits, consensus fuzzing, chaos engineering, public bug bounty program   |
|                                                                                                   |
| [Phase 06: Mainnet Genesis & Real-World X]                                                        |
|  - Genesis ceremony, decentralized validator launch, token distribution, RWA enterprise SDKs      |
+---------------------------------------------------------------------------------------------------+
```

---

## 2. Phase-by-Phase Deliverables & Milestones

### Phase 01: Technical Architecture & System Specification
- **Milestone 1.1**: Master Architecture Specification (`docs/ARCHITECTURE.md`).
- **Milestone 1.2**: BFT-PoS Consensus & Mathematical Model (`docs/CONSENSUS.md`).
- **Milestone 1.3**: P2P Networking & Sentry Topology (`docs/NETWORK.md`).
- **Milestone 1.4**: Dual-Tier Storage & JMT State Commitment (`docs/STORAGE.md`).
- **Milestone 1.5**: 32-Point Security Threat Matrix & Cryptographic Rules (`docs/SECURITY.md`).
- **Milestone 1.6**: Floating Native Tokenomics & Global Currency Display (`docs/TOKENOMICS.md`).
- **Milestone 1.7**: On-Chain Governance & Security Council (`docs/GOVERNANCE.md`).
- **Milestone 1.8**: In-Band Upgradeability & State Migration (`docs/UPGRADEABILITY.md`).
- **Milestone 1.9**: Technology Framework Decision Matrix (`docs/TECH_STACK.md`).
- **Milestone 1.10**: Master Engineering Roadmap (`docs/ROADMAP.md`).
- **Status**: **COMPLETE (Awaiting approval to begin Phase 02)**.

---

### Phase 02: Core Protocol Engine & Local Devnet
- **Scope**:
  - Rust Cargo workspace scaffolding (`sprx-core`, `sprx-consensus`, `sprx-crypto`, `sprx-storage`, `sprx-network`, `sprx-mempool`, `sprx-node`).
  - Cryptographic primitives integration (`ed25519-dalek`, `k256`, `blake3`, `sha2`).
  - Jellyfish Merkle Tree (JMT) implementation with RocksDB/Pebble backend.
  - Append-only segmented flat-file block and receipt storage.
  - libp2p P2P transport, Noise encryption, Discv5, and GossipSub v1.1 messaging.
  - BFT-PoS deterministic consensus round state machine (Propose, Prevote, Precommit, Commit).
  - Mempool transaction ordering and gas fee density sorting.
  - Automated 4-node local devnet test harness running in Docker/Localhost.
- **Verification Gate**: 4-node local cluster continuously producing and finalizing blocks with $< 1.5s$ latency, recovering cleanly from simulated node crash-restarts.

---

### Phase 03: VM Execution, Staking & Governance
- **Scope**:
  - WebAssembly (Wasmtime) sandbox runtime with deterministic instruction-level gas metering.
  - `revm` EVM execution engine integration for Solidity smart contracts.
  - Staking module: validator registration, self-bonding, delegation, and unbonding queues.
  - Slashing module: automated equivocation detection and liveness downtime tracker.
  - On-chain governance module: proposal creation, deposit escrow, tallying, and timelocks.
  - JSON-RPC 2.0 (`sprx_` and standard `eth_` namespaces), gRPC, and WebSocket streaming servers.
  - Change Data Capture (CDC) event stream exporter.
- **Verification Gate**: Successful deployment and execution of complex Wasm and Solidity smart contracts, automated slashing tests, and parameter updates enacted via on-chain governance.

---

### Phase 04: Public Incentivized Testnet & Ecosystem Tooling
- **Scope**:
  - Genesis launch of Public Testnet (SPRX Testnet-1 "Andromeda").
  - ClickHouse + PostgreSQL analytical indexer pipeline consuming CDC streams.
  - Production Block Explorer Web UI (Next.js 15, Tailwind, real-time charts).
  - Multi-Currency Wallet Core (Desktop, Browser Extension, Mobile) supporting INR, USD, EUR, GBP, JPY display toggles via decentralized price oracles.
  - Public testnet faucet and developer documentation portal.
- **Verification Gate**: 50+ independent external validator nodes joined, $> 1,000,000$ testnet transactions processed with zero consensus forks.

---

### Phase 05: Security Audits, Formal Verification & Chaos Testing
- **Scope**:
  - Independent third-party security audits by top-tier blockchain auditing firms.
  - Formal verification of core BFT consensus invariants and state transition math.
  - Chaos engineering: Jepsen testing, network partition simulations, Sybil/Eclipse stress tests.
  - Public Bug Bounty program ($500,000 USD prize pool).
- **Verification Gate**: Zero Critical or High severity vulnerabilities remaining open; formal verification proof signed off.

---

### Phase 06: Mainnet Genesis Launch & Real-World Adoption
- **Scope**:
  - Genesis Ceremony: Production genesis file generation with decentralized initial validator set.
  - Mainnet Network Spin-up and Token Generation Event (TGE).
  - Staking and governance activation.
  - Enterprise Real-World Asset (RWA) tokenization SDKs and fiat on/off-ramp bridge integrations.
- **Verification Gate**: Mainnet operational with 100+ decentralized validators securing the network globally.
