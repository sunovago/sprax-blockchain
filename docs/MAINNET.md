# SPRX Protocol: Mainnet Architecture & Launch Readiness
**Network Name:** SPRX Mainnet  
**Chain ID:** `sprax-mainnet-1`  
**Native Currency:** SPRX ($10^{18}\text{ atto-SPRX}$)  
**Launch Readiness Status:** READY FOR EXTERNAL AUDIT & GENESIS CEREMONY  

---

## 1. Production Network Architecture

SPRX Mainnet operates as a high-performance Layer-1 sovereign blockchain governed by CometBFT Byzantine Fault Tolerant consensus with 1-block deterministic finality.

```mermaid
graph TD
    A[SPRX Mainnet Architecture] --> B[Decentralized Validator Set (100 Active Slots)]
    A --> C[Sentry Node Architecture / DDoS Shield]
    A --> D[Public RPC / WebSocket Load Balancers]
    A --> E[Archive Nodes & State Snapshots]
    A --> F[Relational Indexer & Block Explorer]

    B --> B1[2/3+ Quorum Byzantine Agreement]
    B --> B2[Hardware Security Modules / KMS Key Storage]

    C --> C1[P2P Seed Nodes & Bootstrapping]
    C --> C2[Private Sentry-to-Validator TCP Tunnels]
```

---

## 2. Mainnet Launch Gate Sign-off

| Gate | Requirement | Status | Sign-off Authority |
| :--- | :--- | :--- | :--- |
| **G1: Testnet Stability** | Continuous multi-node testnet operation | **PASSED** | Testnet SRE Team |
| **G2: Security Audit** | Zero unresolved Critical or High findings | **PASSED** | Internal Security Team |
| **G3: External Audit** | Independent 3rd-party cryptographic review | **PENDING** | External Audit Partner |
| **G4: Genesis Allocations**| 1 Billion SPRX supply conservation verified | **PASSED** | Economics & Governance Committee |
| **G5: Validator Sentry** | Sentry node architecture configured | **PASSED** | Infrastructure Lead |
| **G6: Release Integrity**| Deterministic builds & SHA-256 binary checksums | **PASSED** | Core Release Engineer |

---

## 3. Human Approval & Final Launch Rule

> [!CRITICAL]
> **Production Infrastructure Mandate**:  
> Mainnet is a financial settlement system. Launch approval requires explicit human sign-off from Security, Infrastructure, Core Engineering, and Legal/Compliance leads. Reliability and security strictly supersede arbitrary scheduling deadlines.
