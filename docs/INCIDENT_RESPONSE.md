# SPRX Protocol: Security Incident Response Plan (IRP)
**Document Version:** 1.0.0  
**Target:** Core Contributors, Validator Operators, Security Response Team

---

## 1. Incident Severity Levels

```
+-------------------------------------------------------------------------+
| SEV-1 (CRITICAL)   : Consensus Halt, Fund Drainage, Chain Reorganization|
| SEV-2 (HIGH)       : State Divergence, Validator Mass Jailing           |
| SEV-3 (MEDIUM)     : RPC Degradation, Faucet Flooding, Indexer Lag      |
| SEV-4 (LOW)        : Minor CLI bugs, non-critical telemetry failures    |
+-------------------------------------------------------------------------+
```

---

## 2. Emergency Response Protocol

```mermaid
sequenceDiagram
    autonumber
    participant Mon as Monitoring / Alert System
    participant Team as Security Response Team
    participant Val as Validator Emergency Channel
    participant Patch as Emergency Hotfix Release
    participant Chain as SPRX Blockchain Network

    Mon->>Team: 1. Critical Alert Triggered (e.g. Consensus Halt or State Root Divergence)
    Team->>Team: 2. Convene War Room (Triage within 15 minutes)
    Team->>Val: 3. Issue Network Hold Advisory via encrypted validator channel
    Team->>Patch: 4. Reproduce bug in isolated sandbox & author deterministic hotfix
    Patch->>Patch: 5. Execute regression test matrix & consistency check
    Patch->>Val: 6. Distribute signed patch binary & coordinated restart height
    Val->>Chain: 7. Apply hotfix & resume block production
    Team->>Team: 8. Publish Post-Mortem within 48 hours
```

---

## 3. Validator Emergency Key Compromise Procedure

If a validator operator suspects consensus private key compromise:
1. **Immediate Unstaking / Jailing**: Submit self-jailing or unbonding transaction from operator key.
2. **Key Rotation**: Generate new Ed25519 consensus keypair on an air-gapped machine.
3. **State Migration**: Submit on-chain validator key update message signed by operator address.
4. **Post-Mortem**: Document root cause (e.g. compromised server, exposed secrets).
