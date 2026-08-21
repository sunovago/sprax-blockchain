# SPRX Protocol: Indexer Ingestion & Data Pipeline
**Document Version:** 1.0.0  
**Target:** Event Processing, Storage Optimization, State Reconciliation

---

## 1. Indexing Workflow

```mermaid
sequenceDiagram
    autonumber
    participant Node as Node Service / Ledger
    participant Pipeline as Indexer Ingestion Task
    participant Store as Index Store (Relational)
    participant Auditor as Consistency Auditor

    Node->>Pipeline: 1. Finalized Block #H + Tx Receipts
    Pipeline->>Pipeline: 2. Calculate Blake3 Block Hash & Tx Hashes
    Pipeline->>Store: 3. Insert IndexedBlock(H, Hash, Proposer, Gas)
    loop For Each Transaction in Block
        Pipeline->>Pipeline: 4. Extract Sender, Recipient, Amount, Memo
        Pipeline->>Store: 5. Insert IndexedTx(Hash, H, From, To, Amount, Success)
        Pipeline->>Store: 6. Update Account Activity & Tx History
    end
    Pipeline->>Auditor: 7. Trigger State Reconciliation Audit
    Auditor->>Node: 8. Query Ledger Latest Header & Account State
    Auditor->>Store: 9. Compare StateRoot & Balances
    Auditor-->>Pipeline: 10. Audit PASSED (100% Fidelity)
```

---

## 2. Ingestion Stages & Transforms

### Stage 1: Header Ingestion & Hash Validation
- Header metadata (`height`, `parent_hash`, `timestamp_unix_secs`, `proposer`, `state_root`, `txs_root`) is validated against cryptographic commitments.
- Total gas consumed is aggregated across all execution receipts.

### Stage 2: Transaction Normalization & Event Splitting
- Each `Transaction` in `BlockBody` is unpacked:
  - `TxMessage::Transfer { to, amount }` extracts transfer value and recipient.
  - Senders and recipients are indexed into bidirectional lookup arrays for instant address timeline generation.

### Stage 3: Validator & Staking Tracking
- Synchronizes with `StakingKeeper`:
  - Computes relative voting power share: $\text{Share}_i = \frac{w_i}{W_{\text{total}}} \times 100\%$.
  - Tracks uptime percentages and slashing history.

### Stage 4: State Consistency Audits
- Periodically verifies that:
  $$\text{Indexer}.\text{height} = \text{Ledger}.\text{height}$$
  $$\text{Indexer}.\text{state\_root} = \text{Ledger}.\text{state\_root}$$
  $$\sum_{\text{Indexed Accounts}} \text{balance}_i = \text{Ledger Total Supply}$$
