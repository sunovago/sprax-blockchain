# SPRX Protocol: Explorer & Indexer Architecture
**Document Version:** 1.0.0  
**Target:** Blockchain Indexing, Relational Query Engine, Responsive Web UI

---

## 1. System Overview & End-to-End Pipeline

The **SPRX Explorer & Indexer Infrastructure** ingests finalized CometBFT blockchain blocks, extracts transactions and emitted state events, normalizes account balances, and provides low-latency paginated REST/GraphQL APIs and a responsive Web UI.

```
+---------------------+
| SPRX Chain Ledger   |  Block Commitment (1-block finality)
+----------+----------+
           |
           v
+---------------------+
| SPRX JSON-RPC (:2657|  GET /blocks/{height}, /txs/{hash}
+----------+----------+
           |
           v
+---------------------+
| Indexer Engine      |  Event Extraction, State Auditing & Integrity Checks
+----------+----------+
           |
           v
+---------------------+
| Relational Store    |  Indexed Blocks, Tx Chronology, Account Balances
+----------+----------+
           |
           v
+---------------------+
| Explorer REST API   |  Rate Limiting, Pagination, Omni-Search Resolver
+----------+----------+
           |
           v
+---------------------+
| Explorer Web UI     |  React + TypeScript (Desktop & Mobile Responsive)
+---------------------+
```

---

## 2. Component Subsystems

### 2.1 Indexer Engine (`crates/sprax-indexer/src/engine.rs`)
- **Block Ingestion**: Parses header hashes, parent hash linkages, proposer address, gas used, and transaction payloads.
- **Transaction Normalization**: Categorizes message types (`Transfer`, `Delegate`, `Undelegate`, `ContractCall`), sender/recipient balances, fees paid, and execution receipts.
- **Validator Set Synchronization**: Tracks validator rank, commission rate, bonded tokens, and consensus uptime.
- **Continuous Consistency Auditor**: Reconciles indexer heights, block hashes, and account balances against the underlying `ChainLedger` state store, rejecting any desynchronization.

### 2.2 Relational Storage Layer (`crates/sprax-indexer/src/storage.rs`)
- **Multi-Index Keys**:
  - Blocks: Primary index by integer `height` (BTreeMap), secondary lookup by 32-byte `hash`.
  - Transactions: Primary index by `tx_hash`, secondary index by `block_height`, secondary index by `Address` (sender / recipient).
  - Accounts: Primary index by `Address`.
  - Validators: Primary index by `operator_address`, secondary lookup by `moniker`.

### 2.3 Explorer Web Application (`apps/explorer-ui/`)
- **Responsive Layout**: Designed for mobile smartphones, tablets, and wide-screen desktop displays.
- **Omni-Search**: Instant search bar resolving block heights, 32-byte hashes, Bech32/Hex addresses, and validator monikers.
- **Multi-Currency Global Display**: Renders reference fiat estimates in USD ($), INR (₹), EUR (€), and JPY (¥).
