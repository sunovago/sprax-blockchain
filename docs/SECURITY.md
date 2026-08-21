# SPRX Protocol — Security Architecture & Threat Matrix

## Cryptographic Standards
- **Signatures**: Ed25519 (`ed25519-dalek` / RFC 8032) and ECDSA Secp256k1 (`k256`).
- **Hashing**: BLAKE3 for high-throughput state hashing, transaction IDs, and Merkle tree roots; SHA-256 for standard protocol compatibility.
- **Vault Encryption**: AES-256-GCM with PBKDF2-HMAC-SHA256 (100,000 iterations).

---

## Security Threat Matrix

| Threat ID | Threat Category | Risk Level | Mitigation Strategy |
|:---|:---|:---|:---|
| **THR-01** | Equivocation / Double Signing | **CRITICAL** | On-chain equivocation evidence collection, 5% slashing penalty, and permanent validator tombstoning. |
| **THR-02** | Transaction Replay Attacks | **HIGH** | Sequential monotonic account nonces + strict ChainId envelope binding. |
| **THR-03** | Mempool Denial-of-Service | **HIGH** | Static signature verification + minimum gas price check (`min_gas_price_atto`) before admission. |
| **THR-04** | Contract Reentrancy | **HIGH** | Active call-stack tracking (`WasmContractEngine::enter_call` / `exit_call`) prohibiting reentrant invocation. |
| **THR-05** | Unauthorized API Impersonation | **CRITICAL** | Replay-resistant cryptographic challenge nonce verified with Ed25519 signatures in `auth.py`. |
| **THR-06** | Weak Vault Degradation | **HIGH** | Hard-failure if SubtleCrypto / AES-GCM is absent; elimination of XOR fallbacks. |
