# SPRX Backend Architecture Status

**Last Updated:** 2026-08-20  
**Environment:** TESTNET READY  
**Backend Stack:** Python / FastAPI / PostgreSQL / Redis / Celery / WebSocket

---

## Module Status Matrix

| Phase | Module | Status | Notes |
|-------|--------|--------|-------|
| B01 | Repository Audit | COMPLETE | Rust chain phases 01-12 complete. No Python backend existed. |
| B02 | Core FastAPI Architecture | COMPLETE | main.py, router, middleware, CORS, rate limit, request ID |
| B03 | PostgreSQL + Alembic | COMPLETE | 15 models, migrations env.py, seed script |
| B04 | Redis | COMPLETE | Caching layer, pub/sub, TTL constants |
| B05 | Sprax Chain Client | COMPLETE | SpraxChainClient: all 12 RPC methods, JSON-RPC 2.0 |
| B06 | Indexer | COMPLETE | IndexerEngine: blocks, txs, validators, network stats |
| B07 | Explorer APIs | COMPLETE | blocks, txs, addresses, validators, network endpoints |
| B08 | Markets | COMPLETE | CoinGecko provider, Redis cache, gainers/losers/trending |
| B09 | FX | COMPLETE | USD/INR/EUR/GBP/JPY, exchangerate-api, Redis cache |
| B10 | Search | COMPLETE | Block/tx/address/validator/asset detection |
| B11 | Discover | COMPLETE | Aggregates markets + chain + ecosystem projects |
| B12 | Portfolio | COMPLETE | Available + staked + delegations + fiat conversion |
| B13 | Validators / Staking | COMPLETE | Indexed validators + live chain delegations |
| B14 | Watchlists / Auth | COMPLETE | Signed challenge auth, JWT, watchlist CRUD |
| B15 | WebSockets | COMPLETE | 5 channels, heartbeat, auth for private channels |
| B16 | Notifications | COMPLETE | Notification list/read, price alerts CRUD |
| B17 | Workers | COMPLETE | Celery + beat: market refresh, indexer, price alerts, discover |
| B18 | Admin + Feature Flags | COMPLETE | Admin auth, RBAC, feature flags, audit logs |
| B19 | Perps Read Architecture | COMPLETE | Public read APIs: markets, ticker, funding, trades |
| B20 | Perps Testnet Adapter | TESTNET ONLY | is_testnet=True enforced. Production BLOCKED. |
| B21 | Security Hardening | PARTIAL | CORS, rate limit, input validation done. Ed25519 sig verify TODO. |
| B22 | Load Testing | MISSING | Requires running stack |
| B23 | Flutter API Contract | COMPLETE | docs/FLUTTER_API_CONTRACT.md |
| B24 | Explorer API Contract | COMPLETE | docs/EXPLORER_API_CONTRACT.md |
| B25 | End-to-end Testnet | BLOCKED | Requires live testnet RPC node |

---

## Chain Integration Facts

- **RPC Protocol:** JSON-RPC 2.0 on port 26657
- **Chain ID:** `sprax-testnet-1` (testnet) / `sprax-mainnet-1` (mainnet)
- **Address Format:** Bech32 `sprax1...` (38 chars after prefix)
- **Validator Operator:** `spraxvaloper1...`
- **SPRX Decimals:** 18 (atto-SPRX = 10^-18 SPRX)
- **Default Fee:** 0.001 SPRX = `1000000000000000` atto-SPRX
- **Default Gas:** 200,000
- **Signature:** Ed25519 (primary), Secp256k1 (EVM compat)
- **Transaction Format:** JSON-RPC `sprax_broadcastTx` with signed body + public_key + signature

---

## Critical Blockers

1. **Ed25519 Signature Verification in `/auth/verify`:** Currently trusted on nonce match for testnet. Production requires cryptographic Ed25519 verification of the challenge signature against the provided public key.
2. **Live RPC Node:** Backend requires a running Sprax node on port 26657 for chain data.
3. **Market API Key:** CoinGecko API key required for production rates. Without it, free tier applies.
4. **FX API Key:** exchangerate-api key required for live FX rates. Without it, fallback static rates are used (must be updated).
5. **Production Perps:** BLOCKED until oracle, risk engine, liquidation, and legal sign-off.
6. **Mainnet:** BLOCKED until genesis ceremony and external security audit.

---

## Production Perps Gate

**STATUS: PRODUCTION BLOCKED**

Required before production Perps activation:
- [ ] Approved on-chain perpetual settlement smart contracts
- [ ] Decentralized oracle feed review
- [ ] Risk engine formal audit
- [ ] Liquidation keeper audit  
- [ ] Independent security audit
- [ ] Infrastructure review
- [ ] Legal/compliance sign-off
- [ ] Explicit human production approval

API enforcement: `perps_enabled` feature flag defaults to `false`. The admin API blocks setting it via API and requires environment-level override with explicit human approval.
