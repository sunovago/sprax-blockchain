# SPRX Protocol — Gap Analysis & Build Plan

> Generated from a full codebase scan (Rust workspace, Python backend, frontend apps, wallet SDK, infra/docs) on 2026-08-21.
> Status snapshot: the chain runs as a working single-node BFT devnet (see `sprax start`), but almost nothing downstream of it is actually wired to real chain data yet — that is the single biggest theme in this document.

## Quick status table

| Layer | Maturity | Connected to real chain? |
|---|---|---|
| `crates/` (Rust chain core) | Working single-node devnet | — |
| `crates/sprax-wasm` (contracts) | Code exists, no VM execution | Not wired into node/CLI |
| `crates/sprax-indexer` | Code + tests exist | Not wired, no server |
| `crates/sprax-faucet` | Code + tests exist | Not wired, no server |
| `backend/` (FastAPI) | Broad, mostly real (DB + market APIs) | RPC client exists but **has nothing to call** — node has no RPC server |
| `apps/explorer-ui` | 15 pages, real API layer | Silently falls back to mock data on failure |
| `apps/admin-panel` | 18 pages, real JWT API layer | Yes, to backend (not chain directly) |
| `apps/mobile-wallet` | ~20 real screens, real services | Assumes chain RPC exists (it doesn't) |
| `apps/web-wallet` | 4 components, no app shell | Uses real SDK, but app won't run (no entry point) |
| `packages/sprax-wallet-core` | Fully implemented | Used by exactly 1 of 4 apps |
| Docs (`docs/*.md`) | 1 file exists | 11 files linked from README are missing |
| CI (`.github/workflows/ci.yml`) | Rust-only, one job has invalid YAML | No backend/frontend/docker CI |

---

## 🔧 FIX — broken or incomplete as-is

- **CI is broken**: [.github/workflows/ci.yml:51](.github/workflows/ci.yml#L51) — a step is `- actions/checkout@v4` with no `uses:` key, invalid workflow syntax; the clippy job fails to check out the repo.
- **Blocks finalize absurdly fast (100+/sec)**: [crates/sprax-node/src/consensus_driver.rs](crates/sprax-node/src/consensus_driver.rs) `run_one_height`/`run` never sleeps against `block_time_target_ms` (defined in [crates/sprax-core/src/genesis.rs:31](crates/sprax-core/src/genesis.rs#L31)); the value is read from genesis JSON and never used again anywhere in the workspace.
- **`receipts_root` is fake**: [crates/sprax-core/src/ledger.rs:298](crates/sprax-core/src/ledger.rs#L298) hashes a constant string (`"receipts-placeholder"`) instead of building a real merkle root over tx receipts — unverifiable for light clients.
- **Auth is a trust-the-client stub**: [backend/app/api/v1/auth.py:61-63](backend/app/api/v1/auth.py#L61-L63) — comment admits signatures aren't verified ("For testnet: trust the submission if nonce matched"); anyone who reads/guesses the nonce can authenticate as any address.
- **Wallet SDK has a weak crypto fallback**: [packages/sprax-wallet-core/src/vault.ts:86-92,139-144](packages/sprax-wallet-core/src/vault.ts#L86-L92) — falls back to plain XOR "encryption" when WebCrypto isn't available instead of failing closed.
- **Mainnet compose references an image that's never built**: [deploy/docker-compose.mainnet.yml:13](deploy/docker-compose.mainnet.yml#L13) pulls `spraxnetwork/sprax-node:1.0.0` from Docker Hub — doesn't exist, not built anywhere in this repo.
- **Validator key script has an insecure fallback**: `scripts/generate_validator_keys.sh` falls back to raw `openssl` "for demonstration" if the CLI isn't found — silently produces non-standard keys.
- **Storage pruning does nothing**: `PruningStrategy` enum ([crates/sprax-storage/src/pruning.rs](crates/sprax-storage/src/pruning.rs)) is configured but has zero references anywhere else in the crate — every node is an unbounded archive node regardless of config.
- **BFT round-change safety hole**: `RoundState.locked_round/locked_block/valid_round/valid_block` in `crates/sprax-consensus/src/round.rs:21-36` are declared but never read/written — no lock-on-PoLC protection, so once round > 0 is actually exercised (multi-node, real network delay) a validator could equivocate across rounds. Only round-0 convergence is tested today.
- **Explorer masks backend outages**: [apps/explorer-ui/src/services/api.ts](apps/explorer-ui/src/services/api.ts) silently falls back to hardcoded `mockData.ts` on any fetch failure/empty response — makes it impossible to tell from the UI whether the backend is actually reachable.

## 🔗 CONNECT — pieces that exist separately but aren't wired together

- **Nothing can query a running node remotely.** This is the root cause of most other disconnections: no JSON-RPC/gRPC server exists anywhere in `crates/` (no axum/hyper/warp/tonic dependency in the whole workspace). `sprax query`/`sprax tx`/`sprax status` ([crates/sprax-cli/src/commands/{query,tx,status}.rs](crates/sprax-cli/src/commands/query.rs)) talk directly to local redb/JSON files via `NodeService::new_or_load`, not over the network — despite `config().rpc.json_rpc_port` existing as a config field that nothing binds to.
- **Backend's chain client has nothing to call.** `backend/app/modules/blockchain/client.py` is a real JSON-RPC 2.0 client posting to `SPRAX_RPC_URL` (default `http://localhost:26657`) — but per the point above, no server answers on that port. `blockchain.py`, `staking.py`, `portfolio.py`, `network.py` all depend on this.
- **Two separate Docker stacks, never combined.** Root [docker-compose.yml](docker-compose.yml) runs a 3-node chain devnet on network `sprax-net`; `backend/docker-compose.backend.yml` runs api+worker+postgres+redis on network `sprax-backend`. No script or compose override (`-f x -f y`) joins them; the backend only reaches the chain via a manually-set env var.
- **`sprax-wasm` is not reachable from anywhere.** No `sprax` CLI subcommand for contracts exists; `crates/sprax-node`/`sprax-cli` never import `sprax_wasm`.
- **`crates/sprax-indexer` is not run by anything** — no HTTP server, nothing subscribes it to node block events. Confusingly, **the Python backend has its own, separate, real indexer** (`backend/app/modules/indexer/engine.py`, `IndexerEngine`) that polls chain RPC and writes into Postgres — two same-purpose systems with the same name, one dead, one live. Needs a decision (see New Systems).
- **`crates/sprax-faucet` is not served** — no HTTP endpoint, no `sprax start` integration, despite `deploy/docker-compose.testnet.yml` already expecting a faucet service.
- **`packages/sprax-wallet-core` is used by exactly 1 of 4 frontend apps** (`apps/web-wallet`, and that app has no entry point so it can't even run). `explorer-ui`, `admin-panel`, `mobile-wallet` never import it. `mobile-wallet` reimplements equivalent crypto independently in Dart (`bip39`, `ed25519_edwards`, `pointycastle`) — duplicate logic to maintain in two languages.
- **P2P peer discovery messages are defined but dropped.** [crates/sprax-network/src/service.rs:536-540](crates/sprax-network/src/service.rs#L536-L540) matches `PeerDiscoveryRequest`/`PeerDiscoveryResponse` and does nothing with them; peers are strictly limited to the static `bootstrap_peers` list, no gossip-based exchange.

## ➕ ADD — missing pieces inside systems that already exist

- Real JSON-RPC (or gRPC) HTTP server inside `sprax-node`, exposing at minimum: `sprax_getStatus`, `sprax_getAccount`, `sprax_getBlock`, `sprax_broadcastTx`, `sprax_getValidators` — this single addition unblocks the backend, mobile-wallet, and web-wallet all at once.
- `tokio::time::sleep` (or equivalent pacing) in `consensus_driver.rs::run_one_height`, threading `block_time_target_ms` from genesis into `ConsensusDriver`.
- Real merkle root construction for `receipts_root` in `ledger.rs`.
- Peer-exchange/gossip handling for the already-defined `PeerDiscoveryRequest`/`PeerDiscoveryResponse` messages.
- State pruning implementation honoring `PruningStrategy`.
- Real signature verification in `backend/app/api/v1/auth.py` (Ed25519 check against the nonce, using the address's public key).
- Live subsystem/health checks in `admin.py` (`/system/health`, `/dashboard` `subsystems` block) to replace hardcoded "OPERATIONAL"/latency/connection-count strings ([backend/app/api/v1/admin.py:170-228](backend/app/api/v1/admin.py#L170-L228)).
- Live price oracle for `packages/sprax-wallet-core/src/client.ts:72-75` (currently a hardcoded `sprxFloat * 4.5` USD conversion).
- A root-level `.env.example` (only `backend/.env.example` exists today) documenting `SPRAX_RPC_URL` and friends for the whole stack.
- CI coverage for Python backend (pytest), frontend apps (lint/build/test), and Docker image builds — current CI is Rust-only (`cargo check/test/fmt/clippy` + secret scan).
- The 11 doc files linked from `README.md` but missing from `docs/`: `FRAMEWORK_SELECTION.md`, `ARCHITECTURE.md`, `CONSENSUS.md`, `NETWORK.md`, `STORAGE.md`, `SECURITY.md`, `TOKENOMICS.md`, `GOVERNANCE.md`, `UPGRADEABILITY.md`, `TECH_STACK.md`, `ROADMAP.md`.

## ✏️ EDIT — existing code that needs modification

- `crates/sprax-node/src/consensus_driver.rs` — add timing/backoff (see Fix + Add above).
- `crates/sprax-core/src/ledger.rs:298` — replace placeholder `receipts_root` hash with a real computation.
- `backend/app/api/v1/auth.py` + `verify.py` — replace trust-the-nonce logic with real signature verification.
- `packages/sprax-wallet-core/src/vault.ts` — remove the XOR fallback or make it fail loudly instead of silently downgrading security.
- `.github/workflows/ci.yml:51` — fix the malformed checkout step; extend the pipeline per the CI item above.
- `apps/explorer-ui/src/services/api.ts` — make the mock-data fallback explicit in the UI (e.g. a visible "demo/offline data" banner) instead of silently substituting it.
- `crates/sprax-storage/src/pruning.rs` — either implement pruning against `redb_store.rs` or remove the unused config surface until it's real.

## 🆕 NEW SYSTEMS — build from scratch

1. **Rust-node JSON-RPC/gRPC server** (foundational — almost everything else in this document is blocked on this one piece not existing).
2. **Unified deployment wiring** joining the chain, backend, and frontends into one runnable stack (shared Docker network, consistent env var for `SPRAX_RPC_URL`, one `docker-compose` entry point or documented multi-file invocation).
3. **WASM contract execution runtime** — wire an actual engine (e.g. `wasmtime`/`wasmer`) into `sprax-wasm`'s `vm.rs`, which today only hashes/stores bytes and registers metadata without ever loading or running a module; expose it via a CLI subcommand and the new RPC server.
4. **Indexer consolidation** — decide whether `crates/sprax-indexer` (Rust, unwired) or the backend's `IndexerEngine` (Python, real, already working) is the canonical indexer, and retire or properly wire the other.
5. **Faucet service** — stand `sprax-faucet` up as an actual HTTP service (or formally route faucet requests through the backend instead), matching what `deploy/docker-compose.testnet.yml` already expects.
6. **`web-wallet` app shell** — the SDK-wired components (`WalletDashboard.tsx`, `SendModal.tsx`, `ReceiveModal.tsx`, `WalletOnboarding.tsx`) exist but there's no `App.tsx`/router/`index.html`/`main.tsx`/`vite.config.ts` around them; the app cannot currently run.
7. **P2P peer discovery/gossip protocol** implementation, beyond the current static-bootstrap-list-only network.
8. **Full documentation set** — the 11 missing `docs/*.md` files (architecture, consensus, network, storage, security, tokenomics, governance, upgradeability, tech stack, roadmap, framework selection). Note: `docs/Blockchain.md`, the one file that does exist, is not documentation — it's a Hinglish phase-by-phase prompt script for driving Claude Code, not a spec.
9. **End-to-end CI/CD** covering backend tests, frontend builds/tests, Docker image builds for all services, and validation of the `deploy/` manifests (currently untested by CI).

---

### Suggested build order

1. RPC server in `sprax-node` (unblocks backend + both wallets in one shot).
2. Fix consensus block-timing + `receipts_root` (chain correctness).
3. Wire backend ↔ chain end-to-end via Docker (shared network, one compose entry point).
4. Fix `auth.py` signature verification (security — currently exploitable).
5. Give `web-wallet` an app shell so the SDK integration is actually reachable.
6. Everything else (wasm runtime, indexer consolidation, faucet, docs, CI) can proceed in parallel once #1–#3 are solid.
