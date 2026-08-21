# SPRX Protocol: Storage Architecture & State Commitment Specification
**Document Version:** 1.0.0-PROPOSAL  
**Status:** ARCHITECTURE_ONLY (Phase 01)  
**Storage Stack:** Dual-Tier Storage / Jellyfish Merkle Tree (JMT) / LSM-Tree (RocksDB / Pebble)

---

## 1. Storage Architecture Overview

The SPRX storage architecture is engineered to provide high write throughput, ultra-fast cryptographic state commitment calculation, deterministic crash recovery, and efficient state pruning.

```
+---------------------------------------------------------------------------------------------------+
|                                  SPRX DUAL-TIER STORAGE ENGINE                                    |
+---------------------------------------------------------------------------------------------------+
|                                                                                                   |
| [TIER 1: HOT STATE STORAGE (LSM-Tree)]                                                            |
|  - Engine: Embedded RocksDB / Pebble / MDBX                                                       |
|  - Layout: Versioned Jellyfish Merkle Tree (JMT) + Sparse Merkle Tree (SMT)                       |
|  - Content: Account Balances, Nonces, Smart Contract Storage Slots, Validator Set                 |
|  - Operations: Sub-millisecond point lookups, atomic batch state diff commits                     |
|                                                                                                   |
| [TIER 2: IMMUTABLE HISTORICAL LOG (Flat Append-Only Files)]                                       |
|  - Engine: Pre-allocated Segmented Append-Only Flat Files (`.blk` & `.idx`)                       |
|  - Content: Raw Block Headers, Full Transaction Payloads, Execution Receipts, Event Logs         |
|  - Operations: Sequential batch writes, indexed offset random reads, zero GC fragmentation        |
|                                                                                                   |
| [CRASH RECOVERY & ATOMICITY (WAL)]                                                                |
|  - Write-Ahead Log (WAL) guaranteeing ACID transactions across Tier 1 and Tier 2                  |
+---------------------------------------------------------------------------------------------------+
```

---

## 2. Tier 1: State Management & Jellyfish Merkle Tree (JMT)

SPRX manages global state using a **Jellyfish Merkle Tree (JMT)**, a versioned, copy-on-write sparse Merkle trie structure optimized for low disk write amplification and fast root hash calculation.

```
+---------------------------------------------------------------------------------------------------+
|                                  JELLYFISH MERKLE TREE (JMT) LAYOUT                               |
+---------------------------------------------------------------------------------------------------+
|                                                                                                   |
|                                       [Root Node (Version H)]                                     |
|                                            /           \                                          |
|                                           /             \                                         |
|                                    [Internal Node]     [Internal Node]                            |
|                                      /         \              |                                   |
|                                     /           \             |                                   |
|                             [Leaf: Account A]  [Leaf: Account B]  [Leaf: Account C]               |
|                             - Nonce: 42        - Nonce: 104       - Nonce: 1                      |
|                             - Balance: 500 SPRX- Balance: 12 SPRX - Balance: 9,000 SPRX           |
|                             - StorageRoot: 0x..- StorageRoot: 0x..- StorageRoot: 0x..             |
|                                                                                                   |
+---------------------------------------------------------------------------------------------------+
```

### 2.1 JMT Node Encoding & Types
1. **Internal Node**: Contains a 16-element child bitmap and child hash pointers (nibble-based branching, fan-out = 16).
2. **Leaf Node**: Contains `KeyHash = Blake3(AccountAddress)` and `ValueHash = Blake3(AccountState)`.
3. **Null Node**: Implicit empty subtree representation requiring 0 bytes on disk.

### 2.2 Account State Schema
```protobuf
syntax = "proto3";
package sprx.storage.v1;

message AccountState {
  uint64 nonce = 1;
  bytes balance = 2;          // 256-bit unsigned integer (in atto-SPRX)
  bytes code_hash = 3;        // 32-byte hash of contract bytecode (empty for EOA)
  bytes storage_root = 4;     // 32-byte root hash of contract storage trie
  uint64 last_modified_height = 5;
}
```

### 2.3 Contract Storage Slot Layout
Each smart contract maintains its own isolated JMT storage subtree:
- `SlotKey = Blake3(ContractAddress || StorageSlotIndex)`
- `SlotValue = StoredBytes`

---

## 3. Tier 2: Immutable Block & Receipt Flat-File Storage

Historical block data and receipts are strictly separated from hot state to prevent LSM-tree compaction overhead and write amplification.

```
/data/sprx/chain/
├── blocks/
│   ├── block_0000000000.blk       # 1 GB Segmented Block Chunk (Heights 0 - 50,000)
│   ├── block_0000000000.idx       # Binary Index: Height -> File Offset + Length
│   ├── block_0000050000.blk       # Next 1 GB Segmented Block Chunk
│   └── block_0000050000.idx
├── receipts/
│   ├── receipts_0000000000.rcp    # Segmented Transaction Execution Receipts
│   └── receipts_0000000000.idx
└── state/                         # RocksDB / Pebble LSM Database Directory
    ├── CURRENT
    ├── MANIFEST-000001
    └── *.sst
```

### 3.1 Binary Index Format (`.idx`)
Each record in the index file is a fixed 16-byte structure:
- `Offset`: 8 bytes (`uint64`) — byte offset in corresponding `.blk` file.
- `Length`: 4 bytes (`uint32`) — payload length in bytes.
- `Checksum`: 4 bytes (`uint32`) — CRC32C checksum of data chunk.

---

## 4. Atomic Commit Pipeline & Crash Consistency

To guarantee absolute database integrity across system crashes, power cuts, or hardware failures, state transitions execute through a strict **Two-Phase Write-Ahead Logging (WAL)** pipeline:

```
+---------------------------------------------------------------------------------------------------+
|                                  STATE COMMIT PIPELINE (ATOMIC)                                   |
+---------------------------------------------------------------------------------------------------+
|                                                                                                   |
|  1. Execute Block Txs (VM Sandbox) -> Generate Dirty State Batch (In-Memory)                      |
|                                |                                                                  |
|                                v                                                                  |
|  2. Write Block & Receipts to Flat File -> Append to AppendLog (Pending)                         |
|                                |                                                                  |
|                                v                                                                  |
|  3. Append State Mutation Batch to WAL (Synchronous fsync)                                        |
|                                |                                                                  |
|                                v                                                                  |
|  4. Compute New JMT StateRoot Hash -> Validate with Block Header StateRoot                        |
|                                |                                                                  |
|                                v                                                                  |
|  5. Apply Batch to LSM State DB (RocksDB/Pebble) -> Mark WAL Checkpoint Committed                 |
|                                |                                                                  |
|                                v                                                                  |
|  6. Notify Ingress & Indexers (CDC Event Stream)                                                  |
|                                                                                                   |
+---------------------------------------------------------------------------------------------------+
```

### Crash Recovery Mechanism:
- If a node crashes during steps 1–3: The pending block is uncommitted; on restart, the node rolls back to the last committed checkpoint in the WAL.
- If a node crashes during steps 4–5: On restart, the node detects unapplied WAL entries, replays the batch into the LSM-tree, and cleanly completes the commit.

---

## 5. State Pruning & Archival Retention Modes

```
+---------------------------------------------------------------------------------------------------+
|                                  STORAGE RETENTION PROFILES                                       |
+---------------------------------------------------------------------------------------------------+
| Node Profile       | State Trie Retention     | Historical Block Retention | Disk Footprint (1 Yr) |
+--------------------+--------------------------+----------------------------+----------------------+
| Full Node (Default)| Current + Last 10k H     | Last 1,000,000 Blocks      | ~250 GB - 400 GB     |
| Archive Node       | All Versions (0 to H)    | All Blocks (Genesis to H)  | ~3 TB - 6 TB         |
| Validator Node     | Current + Last 1k H      | Last 500,000 Blocks        | ~150 GB - 250 GB     |
| Light Node         | None (Verifies Proofs)   | Headers Only               | < 1 GB               |
+---------------------------------------------------------------------------------------------------+
```

### 5.1 Generational State Pruning Algorithm (Full Nodes)
1. Full nodes maintain an active state version $V_{current}$.
2. Every $1,000$ blocks, an asynchronous background pruning worker sweeps JMT nodes whose `stale_since_version < V_{current} - 10,000`.
3. Stale nodes are removed in batch SST file compactions, bounding disk consumption permanently without blocking real-time block execution.

---

## 6. Fast State Sync & Snapshot Serialization

When a new node synchronizes with the network via State-Sync:
1. **Snapshot Creation**: Every 50,000 blocks, archival/full nodes export a state snapshot partitioned into deterministic **16 MB Chunks**.
2. **Chunk Commitment**:
   $$\text{ChunkHash}_k = \text{Blake3}(\text{ChunkData}_k)$$
   $$\text{SnapshotManifestRoot} = \text{MerkleRoot}(\{\text{ChunkHash}_1, \dots, \text{ChunkHash}_M\})$$
3. **Parallel Chunk Streaming**: The syncing node downloads chunks concurrently from multiple peers via P2P.
4. **Instant Verification**: Reconstructed trie is validated against the consensus-signed `StateRoot`, achieving full sync in $< 15\text{ minutes}$ regardless of chain age.

---

## 7. Storage Performance Targets

| Metric | Target Specification |
| :--- | :--- |
| **Account Balance Point-Lookup** | $< 0.8\text{ ms}$ ($p99 < 2.5\text{ ms}$) |
| **Contract Storage Slot Read** | $< 1.2\text{ ms}$ ($p99 < 3.5\text{ ms}$) |
| **Batch State Commit (10,000 Txs)** | $< 35\text{ ms}$ |
| **StateRoot Hash Calculation** | $< 20\text{ ms}$ via parallelized Blake3 JMT hashing |
| **Write Amplification Factor** | $< 3.8\times$ (LSM tuned with dynamic compaction) |
