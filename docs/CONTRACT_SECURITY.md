# SPRX Protocol: Smart Contract Security & Vulnerability Matrix
**Document Version:** 1.0.0  
**Target:** Smart Contract Auditing, Attack Vector Mitigations, Invariant Testing

---

## 1. Security Architecture & Threat Matrix

| Threat Category | Attack Vector | SPRX Mitigation Mechanism |
| :--- | :--- | :--- |
| **Reentrancy** | Recursive calling before state update | Native Runtime Reentrancy Call Stack Lock (`WasmContractEngine::enter_call`) + Checks-Effects-Interactions (CEI) model |
| **Integer Overflow / Underflow** | Arithmetic overflow in balances | Rust 2021 checked arithmetic (`checked_add`, `checked_sub`) + `Uint128` / `Amount` type system |
| **Unauthorized Access** | Non-admin executing admin actions | Strict caller validation (`info.sender == state.admin`) returning `ContractError::Unauthorized` |
| **Denial of Service (DoS)** | Infinite loops or excessive storage writes | Metered step gas accounting (`GasMeter::consume`) returning `ContractError::OutOfGas` |
| **Storage Collision** | Overwriting global keys | Contract-scoped prefix isolation (`state.entry(contract_addr)`) |
| **Front-Running / MEV** | Mempool transaction reordering | CometBFT deterministic round-robin proposer ordering + fast 1-block finality |

---

## 2. Best Practices Checklist for Contract Authors

1. **Checks-Effects-Interactions (CEI)**:
   - Perform all balance, allowance, and permission checks **first**.
   - Mutate local contract state **second**.
   - Emit sub-messages or events **last**.
2. **Deterministic Time**:
   - Never rely on local system time. Use `env.block_height` or `env.block_timestamp_unix` passed deterministically by consensus.
3. **Safe Fund Handling**:
   - Ensure contract accounts cannot retain trapped funds by implementing audited release/refund timeouts.
4. **Mandatory Auditing**:
   - Unaudited contracts **must not** be deployed to production mainnet environments.
