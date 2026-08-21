# SPRX Protocol — Storage & State Commitments

## Architecture Overview
SPRX Protocol utilizes a dual-table persistent key-value store backed by pure-Rust `redb` (`crates/sprax-storage`).

---

## Storage Tables

1. **`state` Table**:
   - Stores account balances, nonces, smart contract bytecodes, and contract storage slots.
   - Key: 20-byte account address or scoped contract storage key.
   - Value: Serialized `AccountState` or contract raw bytes.

2. **`chain_meta` Table**:
   - Stores canonical blockchain headers, full block bodies, hash-to-height index, and transaction execution receipts.
   - Key prefixes:
     - `meta:height`: Current committed blockchain height.
     - `blk:<height_u64_be>`: Canonical serialized `Block`.
     - `blkhash:<hash_32>`: Hash-to-height index lookup.
     - `tx:<tx_hash_32>`: Transaction execution index + receipt.

---

## State Pruning Strategies

- **`Archive`**: Retains all historical block headers, bodies, and intermediate state commitments since genesis.
- **`KeepRecent { keep_blocks }`**: Retains full state for the latest $N$ blocks (default 100,000 blocks), pruning older non-canonical block data via `RedbStore::prune_history`.
- **`Aggressive { keep_blocks }`**: Minimal storage footprint for lightweight validator nodes retaining only the active unbonding/consensus window.
