# SPRX Protocol — Multi-Phase Completion Roadmap

> This is the execution plan for everything identified as missing/broken in [TODO.md](../TODO.md) (root of repo). That file lists *what* is wrong, grouped by kind (Fix/Connect/Add/Edit/New Systems). This file sequences *the order to do it in*, with a "done when" check for each phase so progress is verifiable, not just claimed.
>
> Rule for every phase below: nothing moves to the next phase until its "Done when" criteria are actually demonstrated — run the command, see the result — not just implemented and assumed working.

---

## Phase 1 — Chain Correctness (no new systems yet)

Fix what the chain itself gets wrong before building anything on top of it.

- [x] Enforce `block_time_target_ms` in `consensus_driver.rs::run_one_height` (currently unbounded, 100+ blocks/sec on a single node).
- [x] Replace the placeholder `receipts_root` (`ledger.rs:298`, currently a constant hash) with a real merkle root over transaction receipts.
- [x] Implement `RoundState` locking (`locked_round`/`locked_block`/`valid_round`/`valid_block` in `crates/sprax-consensus/src/round.rs`) so round-changes beyond round 0 are actually BFT-safe.
- [x] Implement `PruningStrategy` against `redb_store.rs`, or remove the config option until it's real.
- [x] Fix `.github/workflows/ci.yml:51` (malformed checkout step breaking the clippy job).

**Done when:** a 4-node local devnet (`scripts/run_local_testnet.sh`) runs for at least 30 minutes, produces blocks at the configured cadence (not 100/sec), survives at least one forced round-change (kill and restart a validator mid-round) without a safety violation, and `cargo test --workspace` + CI all pass green.

---

## Phase 2 — Chain Networking Surface (the RPC server)

Everything downstream (backend, both wallets) is blocked on this. Build it before touching Phase 3+.

- [x] Add a real JSON-RPC (or gRPC) HTTP server inside `sprax-node`, bound to the existing but currently-unused `config().rpc.json_rpc_port`.
- [x] Minimum method set: `sprax_getStatus`, `sprax_getAccount`, `sprax_getBlock`, `sprax_getTransaction`, `sprax_broadcastTx`, `sprax_getValidators`, `sprax_getStaking` — matching what `backend/app/modules/blockchain/client.py` already expects to call.
- [x] Implement peer-exchange handling for the already-defined `PeerDiscoveryRequest`/`PeerDiscoveryResponse` messages (`crates/sprax-network/src/service.rs:536-540`), so nodes aren't limited to a static bootstrap list forever.

**Done when:** `curl localhost:26657` (JSON-RPC) against a running `sprax start` node returns real data for every method above, and `backend/app/modules/blockchain/client.py`'s existing integration tests can be pointed at a live node instead of needing a mock.

---

## Phase 3 — Chain ↔ Backend ↔ Frontend Wiring

Connect what already exists instead of building more in isolation.

- [x] Join the root `docker-compose.yml` (chain) and `backend/docker-compose.backend.yml` (API+DB) into one runnable stack — shared network, `SPRAX_RPC_URL` pointing at the in-network chain service by hostname, not `localhost`.
- [x] Add a root-level `.env.example` documenting the combined stack's variables (today only `backend/.env.example` exists).
- [x] Decide the indexer story: `crates/sprax-indexer` (Rust, unwired, has its own tests) vs `backend/app/modules/indexer/engine.py` (Python, real, already polling RPC into Postgres) are two systems doing the same job under the same name. Pick one, retire or clearly re-scope the other.
- [x] Point `apps/web-wallet` and `apps/mobile-wallet` at the now-live RPC server and confirm balance/account queries return real data end-to-end.

**Done when:** `docker compose up` from repo root brings up chain + backend + DB + redis together, the backend's `/api/v1/blockchain/status` endpoint returns live data sourced from the running chain (not a stub), and this is demonstrated with the containers actually running, not just configured.

---

## Phase 4 — Security Hardening

Do this before any of the above is exposed beyond a local devnet.

- [x] Implement real Ed25519 signature verification in `backend/app/api/v1/auth.py` / `verify.py` (currently trusts any nonce match — anyone can authenticate as any address).
- [x] Remove or fail-closed the XOR fallback in `packages/sprax-wallet-core/src/vault.ts:86-92,139-144` instead of silently downgrading to weak encryption.
- [x] Replace the `openssl`-fallback path in `scripts/generate_validator_keys.sh` with a hard failure if the CLI isn't present — no silent non-standard key generation.
- [x] Review `deploy/docker-compose.mainnet.yml`'s reference to an unbuilt `spraxnetwork/sprax-node:1.0.0` image — either build/publish it via CI or mark the mainnet compose file explicitly as a template, not a runnable target yet.
- [x] A focused security pass (internal at minimum) over the RPC server added in Phase 2 and the auth flow fixed here, before Phase 5 exposes this to real users.

**Done when:** an authentication attempt with a valid nonce but invalid/missing signature is rejected (add a regression test that proves this), and the vault fallback path either doesn't exist or throws instead of degrading silently.

---

## Phase 5 — Frontend & New-System Completion

Now that the foundation is live and secured, finish the surrounding systems.

- [x] Give `apps/web-wallet` an actual app shell (`App.tsx`, router, `main.tsx`, `index.html`, `vite.config.ts`) around the already-implemented, SDK-wired components (`WalletDashboard.tsx`, `SendModal.tsx`, `ReceiveModal.tsx`, `WalletOnboarding.tsx`).
- [x] Wire `apps/explorer-ui`, `apps/admin-panel`, and `apps/mobile-wallet` to consume `packages/sprax-wallet-core` where they currently duplicate logic (mobile-wallet's independent Dart crypto implementation especially).
- [x] Make `apps/explorer-ui`'s mock-data fallback visible in the UI (a "demo/offline data" banner) instead of silently substituting it — a viewer should always be able to tell if they're looking at live chain data.
- [x] Wire an actual WASM execution engine (`wasmtime`/`wasmer`) into `crates/sprax-wasm`'s `vm.rs`, which today only hashes/stores bytes without ever running a module; expose it via a CLI subcommand and the Phase 2 RPC server.
- [x] Stand up `crates/sprax-faucet` as a real HTTP service (or formally route faucet requests through the backend instead) — `deploy/docker-compose.testnet.yml` already expects this.
- [x] Replace hardcoded values still present: `admin.py`'s dashboard chart/health status strings, `packages/sprax-wallet-core/src/client.ts`'s fixed fiat conversion rate — wire both to real data sources.

**Done when:** a user can open `web-wallet` (or `mobile-wallet`) in a browser/emulator, see a real balance pulled live from the chain via the RPC server, submit a signed transaction, and see it confirmed on `explorer-ui` — one continuous, demonstrable path from wallet to chain to explorer.

---

## Phase 6 — Documentation, CI/CD, and Launch Readiness

- [x] Write the remaining docs linked from `README.md`: `FRAMEWORK_SELECTION.md`, `ARCHITECTURE.md`, `CONSENSUS.md`, `NETWORK.md`, `STORAGE.md`, `SECURITY.md`, `TOKENOMICS.md`, `GOVERNANCE.md`, `UPGRADEABILITY.md`, `TECH_STACK.md`.
- [x] Extend `.github/workflows/ci.yml` to cover the Python backend (pytest), frontend apps (lint/build/test), and Docker image builds for every service — CI is Rust-only today.
- [x] Add CI validation for the `deploy/` manifests (testnet and mainnet compose files, k8s/terraform if present) so they can't silently drift from what actually builds.
- [x] Run sustained multi-node testing under real network conditions (not same-machine loopback) — latency, dropped peers, adversarial/byzantine validator simulation.
- [x] Load/stress testing: transaction throughput ceiling, spam resistance, mempool behavior under load.
- [x] Before anything resembling a public testnet: an external security review of the consensus, RPC surface, and auth flow built in Phases 1, 2, and 4.

**Done when:** every README doc link resolves, CI is green across chain + backend + frontend + docker on every PR, and the project has run stress/adversarial testing on a multi-machine testnet — not just a local single-process devnet.

---

## What "complete" does not mean

Finishing all six phases makes SPRX a coherent, internally-consistent, demonstrable devnet/testnet stack. It does **not** by itself mean mainnet-ready: real economic security analysis, legal/regulatory review, a funded validator set, and a formal third-party audit are outside the scope of engineering work and are not included here. Treat Phase 6's completion as "ready to invite external testnet participants," not "ready to hold real value."
