# SPRX Protocol: Smart Contract Architecture
**Document Version:** 1.0.0  
**Target:** WebAssembly (WASM) Virtual Machine, Standard Contracts, Gas Accounting

---

## 1. Virtual Machine Architecture

The SPRX smart-contract runtime is built on a sandboxed, deterministic **WebAssembly (WASM) execution engine (`sprax-wasm`)** adhering to the CosmWasm contract actor model:

```
+-------------------------------------------------------------+
|                      SPRX Node Runtime                      |
|                                                             |
|   +-----------------------------------------------------+   |
|   |            ChainLedger / Transaction Router         |   |
|   +--------------------------+--------------------------+   |
|                              |                              |
|                              v                              |
|   +-----------------------------------------------------+   |
|   |         WASM Contract Engine (sprax-wasm)           |   |
|   |  - Deterministic Bytecode Verification (Blake3)     |   |
|   |  - Sandboxed KV State Store with Atomic Isolation   |   |
|   |  - Exact Gas Metering (Storage + Compute + Memory)  |   |
|   |  - Non-Reentrancy Guard Call Stack Protection       |   |
|   +--------------------------+--------------------------+   |
|                              |                              |
|         +--------------------+--------------------+         |
|         |                    |                    |         |
|         v                    v                    v         |
|   +-----------+        +-----------+        +-----------+   |
|   | CW20 /    |        | Escrow &  |        | Governance|   |
|   | Token     |        | Payments  |        | Voting    |   |
|   +-----------+        +-----------+        +-----------+   |
+-------------------------------------------------------------+
```

---

## 2. Standard Contract Interfaces

### 2.1 CW20 Fungible Token (`crates/sprax-wasm/src/contracts/token.rs`)
- **Messages**:
  - `Instantiate { name, symbol, decimals, initial_balances, minter }`
  - `Transfer { recipient, amount }`
  - `TransferFrom { owner, recipient, amount }`
  - `Approve { spender, amount }`
  - `Mint { recipient, amount }` (strictly gated to `minter`)
  - `Burn { amount }`
- **Queries**:
  - `Balance { address } -> Amount`
  - `Allowance { owner, spender } -> Amount`
  - `TokenInfo {} -> TokenInfo`

### 2.2 Multi-Party Escrow & Conditional Payment (`crates/sprax-wasm/src/contracts/escrow.rs`)
- **Messages**:
  - `Instantiate { recipient, arbiter, timeout_height }`
  - `Deposit {}`
  - `Release {}` (authorized by arbiter or sender)
  - `Refund {}` (authorized by arbiter, or sender if `current_height >= timeout_height`)
- **Queries**:
  - `GetEscrow {} -> EscrowState`

### 2.3 On-Chain Governance & Proposal Voting (`crates/sprax-wasm/src/contracts/governance.rs`)
- **Messages**:
  - `Instantiate { voting_period_blocks, quorum_votes_required }`
  - `SubmitProposal { title, description }`
  - `CastVote { proposal_id, option, voting_weight }`
  - `TallyAndExecute { proposal_id }`
- **Queries**:
  - `GetProposal { proposal_id } -> Proposal`
  - `GetConfig {} -> GovConfig`

---

## 3. Gas & Fee Model

Execution is metered with strict step-wise gas accounting:

| Operation | Gas Cost |
| :--- | :--- |
| **Base Contract Invocation** | $2,000\text{ gas}$ |
| **Storage Read (Base)** | $100\text{ gas}$ |
| **Storage Read (Per Byte)** | $1\text{ gas / byte}$ |
| **Storage Write (Base)** | $500\text{ gas}$ |
| **Storage Write (Per Byte)** | $2\text{ gas / byte}$ |
