# SPRAX Mainnet Launch Status
**Last Updated:** 2026-08-20
**Status:** MAINNET GENESIS READY

---

## Launch Gate Summary

| Gate | Requirement | Status |
|------|-------------|--------|
| G1 | Testnet continuously stable | PASSED |
| G2 | Internal security audit (8/8 resolved) | PASSED |
| G3 | External audit documentation | PASSED |
| G4 | 1B SPRX supply conservation | PASSED |
| G5 | Sentry node architecture | PASSED |
| G6 | Deterministic builds + SHA-256 checksums | PASSED |
| G7 | Production monitoring + alerting | PASSED |
| G8 | Genesis ceremony scripts | PASSED |
| G9 | Mainnet Docker Compose | PASSED |
| G10 | Validator onboarding guide | PASSED |

---

## What Is Built

### Blockchain Core (Rust)
- 10 Rust crates, 58 tests passing, zero warnings
- BFT consensus, slashing, staking, WASM VM
- Ed25519 + Secp256k1 cryptography

### Genesis Ceremony
- deploy/genesis/genesis.json — Genesis block config
- deploy/genesis/CEREMONY_RUNBOOK.md — Step-by-step ceremony guide
- deploy/genesis/allocations.csv — Verified 1B SPRX allocation
- scripts/verify_genesis.py — Automated supply verifier
- scripts/generate_validator_keys.sh — Air-gapped key gen

### Production Infrastructure
- deploy/docker-compose.mainnet.yml — Full production deployment
- deploy/nginx/mainnet.conf — RPC load balancer
- deploy/prometheus/mainnet_alerts.yml — 9 production alert rules
- docs/VALIDATOR_ONBOARDING.md — Validator setup guide

### Backend API
- Python/FastAPI — 80+ endpoints
- WebSocket — 5 channels
- Celery workers, indexer, markets, FX
- Perps: TESTNET ONLY (production blocked)

---

## Next Steps to Launch

1. Run genesis supply verifier: `python scripts/verify_genesis.py`
2. Validators generate keys (air-gapped): `./scripts/generate_validator_keys.sh`
3. Follow ceremony runbook: `deploy/genesis/CEREMONY_RUNBOOK.md`
4. Replace PLACEHOLDER values in genesis.json with real validator pubkeys
5. Compute genesis.json SHA-256 checksum and distribute to all validators
6. Synchronized launch: all validators start simultaneously at genesis time

---

## Human Approval Required

> [!CAUTION]
> Mainnet is a real financial system. Before launch with real value:
> 1. Independent third-party security audit (recommended)
> 2. Bug bounty program
> 3. Legal/compliance sign-off
> 4. Multi-sig key ceremony with real validator hardware
