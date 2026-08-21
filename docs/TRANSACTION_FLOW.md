# SPRX Protocol: End-to-End Transaction Flow
**Document Version:** 1.0.0  
**Target:** Client-to-Chain Transaction Pipeline, Signature Verification, Finality

---

## 1. Transaction Lifecycle Diagram

```mermaid
sequenceDiagram
    autonumber
    actor User as User / DApp
    participant Wallet as SPRX Wallet (Client)
    participant RPC as SPRX RPC Node (:26657)
    participant Mempool as Node Mempool
    participant Consensus as CometBFT Consensus
    participant Ledger as State Machine (ChainLedger)

    User->>Wallet: 1. Input: Recipient, Amount (e.g. 500 SPRX), Memo
    Wallet->>RPC: 2. Query Account Nonce & Latest Balance
    RPC-->>Wallet: 3. Return Nonce (e.g. 4) & Balance
    Note over Wallet: 4. Client validates balance >= amount + fee
    Note over Wallet: 5. Construct Canonical TxBody JSON
    Note over Wallet: 6. Sign TxBody with Private Key (Offline)
    Wallet->>RPC: 7. POST /txs/broadcast (Signed Transaction)
    RPC->>Mempool: 8. Validate Signature, Nonce & Balance
    Mempool-->>RPC: 9. Admitted to Mempool
    RPC-->>Wallet: 10. Return TxHash (0x3a1f...)
    Note over Wallet: 11. Display "Pending Confirmation" Badge
    Consensus->>Mempool: 12. Proposer reaps tx into block proposal
    Consensus->>Consensus: 13. +2/3 Prevotes & Precommits reached
    Consensus->>Ledger: 14. Commit Block #Height & Execute Tx
    Note over Ledger: 15. Balances atomically updated & StateRoot computed
    Wallet->>RPC: 16. Poll /txs/{txHash} (or WebSocket Subscription)
    RPC-->>Wallet: 17. Return TxReceipt (Success, GasUsed, Height #42)
    Note over Wallet: 18. Update UI to "Confirmed" & Refresh Balance
```

---

## 2. Step-by-Step Execution Breakdown

### Step 1: Client Transaction Construction
- The user enters a recipient address (`sprax1...` or `0x...`) and amount in SPRX.
- `TransactionBuilder.sprxToAtto` converts decimal SPRX to atomic `atto-SPRX` ($10^{18}$).
- Default network fee ($0.0005\text{ SPRX}$, gas limit $200,000$) is attached.

### Step 2: Nonce Fetching & Offline Signing
- The wallet fetches the sender's current nonce $N$ via JSON-RPC.
- The wallet constructs canonical JSON `TxBody` without whitespace variations.
- The client generates an Ed25519 signature $\sigma = \text{Sign}_{sk}(\text{sign\_bytes})$ entirely in local memory.

### Step 3: Broadcast & Mempool Admission
- The wallet submits `{ body, keyType: "Ed25519", publicKey, signature }` to the node via `POST /txs/broadcast`.
- The node executes stateless checks:
  1. Validates signature $\sigma$ against `publicKey`.
  2. Asserts `sender` equals $\text{Address}(\text{publicKey})$.
  3. Checks that account balance $\ge \text{amount} + \text{fee}$.
  4. Checks that nonce equals current account nonce $N$.
- The transaction enters the node mempool and is gossiped to connected P2P peers.

### Step 4: Block Inclusion & State Execution
- The elected block proposer bundles the transaction into block $H$.
- `TxExecutor` executes the state transition:
  - Decrements sender balance by $\text{amount} + \text{fee}$.
  - Increments recipient balance by $\text{amount}$.
  - Increments sender nonce $N \leftarrow N + 1$.
  - Generates `TxReceipt`.
- CometBFT reaches +2/3 precommit consensus quorum, committing the block with 1-block deterministic finality.

### Step 5: Client Confirmation & Receipt Display
- The wallet receives the execution receipt containing block height, gas used, and final status (`SUCCESS`).
- The wallet updates the local transaction feed and emits a confirmation notification.
