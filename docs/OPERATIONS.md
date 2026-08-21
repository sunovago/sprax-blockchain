# SPRX Protocol: Mainnet Operations & Key Management Manual
**Document Version:** 1.0.0  
**Target:** SREs, Validator Operators, Network Administrators

---

## 1. Validator Sentry Architecture

To prevent Direct Denial of Service (DDoS) and IP discovery of validator nodes, production operators must use a **Sentry Node Architecture**:

```
+-----------------+     +-----------------+     +-----------------+
| Public Seed #1  |     | Public Seed #2  |     | Public Sentry   |
+--------+--------+     +--------+--------+     +--------+--------+
         |                       |                       |
         +-----------+           |           +-----------+
                     |           |           |
                     v           v           v
               +-----------------------------------+
               |       Private Sentry Nodes        |
               +-----------------+-----------------+
                                 |  (Private WireGuard / VPC)
                                 v
               +-----------------------------------+
               |     Validator Signing Node        |
               | (No Public IP / HSM Key Signing)  |
               +-----------------------------------+
```

---

## 2. Key Management & Operational Security

> [!CAUTION]
> **Zero Plaintext Secrets**: Never store validator consensus private keys, treasury keys, or operator credentials in Git repositories or unencrypted cloud disks.

1. **Validator Consensus Keys**:
   - Must be stored in Hardware Security Modules (HSM) such as YubiHSM2, AWS CloudHSM, or Ledger KMS with remote signing via Privval daemon.
2. **Treasury & Governance Authority**:
   - Minimum 4-of-7 multi-signature requirement for Treasury Reserve transactions.
3. **Upgrade Authority**:
   - Code upgrades require on-chain governance proposal passage (2/3 majority vote across active validator set) with a mandatory 7-day timelock.
