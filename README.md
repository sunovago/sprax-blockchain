# SPRX Protocol (Scalable Protocol for Real-world X)

> **Phase 01: System Architecture Specification** — *COMPLETE*  
> **Phase 02: Blockchain Framework Selection** — *PROPOSED (Awaiting Approval)*

---

## Technical Documentation Suite

The complete technical architecture for the SPRX Protocol is documented in the following specifications:

1. [**Blockchain Framework Selection & ADR-001**](docs/FRAMEWORK_SELECTION.md) — 16-dimension comparative analysis and official framework selection (Cosmos SDK + CometBFT + CosmWasm Rust + EVM).
2. [**Master System Architecture**](docs/ARCHITECTURE.md) — Comprehensive overview of the 20 architectural pillars, node topologies, multi-currency abstraction, and 8 end-to-end Mermaid diagrams.
3. [**Consensus & Mathematical Specification**](docs/CONSENSUS.md) — BFT-PoS consensus mechanics, voting power formulas, deterministic weighted round-robin proposer election, BFT-time, and slashing penalties.
4. [**P2P Networking Specification**](docs/NETWORK.md) — libp2p, QUIC/TCP, Noise protocol encryption, Discv5 discovery, GossipSub v1.1 topics, compact block relay, and validator sentry architecture.
5. [**Storage & State Commitments**](docs/STORAGE.md) — Dual-tier storage architecture, Jellyfish Merkle Tree (JMT), LSM-Tree state storage (RocksDB/Pebble), flat-file block logs, WAL crash consistency, and state pruning.
6. [**Security Architecture & Threat Matrix**](docs/SECURITY.md) — 32-point categorized security threat matrix (Critical, High, Medium, Low) with attack vectors and mitigations, cryptographic standards, and validator HSM key hygiene.
7. [**Native Tokenomics & Global Display**](docs/TOKENOMICS.md) — Native floating cryptocurrency economics (SPRX), 18 decimal precision, dynamic inflation curves, EIP-1559 base fee burn, staking yield, and decoupled global fiat display (INR, USD, EUR, GBP, JPY).
8. [**On-Chain Governance Specification**](docs/GOVERNANCE.md) — Proposal lifecycles, voting mathematics, quorum/majority/veto thresholds, timelock controllers, and emergency security council powers.
9. [**Upgradeability & State Migration**](docs/UPGRADEABILITY.md) — In-band deterministic coordinate upgrades, state migration hooks, and smart contract proxy architectures.
10. [**Technology Decision Matrix**](docs/TECH_STACK.md) — 10-dimensional comparative evaluation of Cosmos SDK, Substrate, Custom Modular Rust L1, EVM Rollups, and Solana SVM.
11. [**Multi-Phase Roadmap**](docs/ROADMAP.md) — Sequential 6-phase engineering lifecycle from Phase 01 Architecture through to Mainnet Genesis.

---

## Native Asset Definition

- **Asset Symbol**: `SPRX`
- **Asset Name**: Scalable Protocol for Real-world X
- **Nature**: Freely floating Layer-1 cryptocurrency (NOT a fixed-price or INR stablecoin).
- **Sub-Unit Precision**: 18 Decimals ($1 \text{ SPRX} = 10^{18} \text{ atto-SPRX}$).
- **Global Currency Display**: Real-time fiat valuation (₹ INR, $ USD, € EUR, £ GBP, ¥ JPY) provided via decentralized oracle price aggregation at the wallet and explorer presentation layers.
