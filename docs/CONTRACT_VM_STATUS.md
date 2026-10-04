# Smart Contract VM (CosmWasm) — Work Status

Tracks the "Phase 1: Real CosmWasm Smart Contract Execution" effort. Full plan/rationale:
`crates/sprax-wasm` was previously a stub (`store_code` just hashed bytes, `execute`/`instantiate`/`query`
never ran any WASM). This effort wires in the real `cosmwasm-vm` crate (wraps `wasmer`) so contracts
actually execute, with gas metering and consensus-verified persistent storage.

## Done

- **Tokenomics (separate, unrelated feature — fully shipped)**: block-reward minting (halving +
  hard cap) and fee-burn accounting. See `docs/TOKENOMICS.md`. All tests pass, clippy clean.
- **Dependency feasibility confirmed**: `cosmwasm-vm = "3.0"` + `cosmwasm-std = "3.0"` added to
  workspace `Cargo.toml` and `crates/sprax-wasm/Cargo.toml`. Builds clean on this Windows machine
  (~4 min cold build) — no cmake/libclang issues like the earlier RocksDB attempt. cosmwasm-vm 3.x
  no longer exposes a Singlepass/Cranelift feature choice (one backend is baked in), which also
  removes the cross-platform gas-metering-determinism concern raised earlier.
- **`sprax-storage` range-scan support**: added `ReadonlyKVStore::scan_prefix(prefix) -> Vec<(Vec<u8>, Vec<u8>)>`,
  implemented for both `MemKVStore` and `RedbStore` (needed for CosmWasm's `db_scan`/`db_next`
  iterator host functions). Tests pass, including a redb prefix-upper-bound edge case
  (`0xff`-byte boundary). This was the only structural gap in the storage layer.
- **Exact `cosmwasm-vm` 3.0.9 API confirmed from actual installed source** (not docs, which can lag):
  `Storage`/`BackendApi`/`Querier`/`Backend<A,S,Q>` traits (`backend.rs`), `Instance::from_code`/
  `InstanceOptions{gas_limit}`/`get_gas_left`/`create_gas_report` (`instance.rs`), `call_instantiate`/
  `call_execute`/`call_query` signatures (`calls.rs`), `Cache::store_code` → `cosmwasm_std::Checksum`
  as the natural content-addressed code ID (`cache.rs`). Full details captured in the approved plan
  file this session (`~/.claude/plans/hashed-dreaming-llama.md` on this machine) and in this
  session's transcript — re-derivable by reading
  `%CARGO_HOME%/registry/src/*/cosmwasm-vm-3.0.9/src/{backend,instance,calls,cache}.rs` again if lost.
- `crates/sprax-core/src/state.rs`: briefly added then **reverted** a `contract_key`/`contract_prefix`
  helper — decided the contract-storage key convention belongs inside `sprax-wasm` itself (which
  owns the `Storage` adapter), not `sprax-core`, since `sprax-wasm` doesn't depend on `sprax-core`.

## Not started yet (the bulk of the work)

1. **`crates/sprax-wasm/src/backend.rs` (new)** — adapter structs implementing `cosmwasm_vm`'s
   `Storage`, `BackendApi`, `Querier` traits:
   - `Storage`: wrap the chain's `KVStore`, scope every key by contract address using a `'c' + addr + key`
     prefix (owned inside `sprax-wasm`, mirroring the `'a'`/`'s'` convention already in
     `sprax-core/src/state.rs`), back `scan`/`next` with the new `scan_prefix`.
   - `BackendApi`: `addr_validate`/`addr_canonicalize`/`addr_humanize`. Decided approach: use
     `Address::to_hex()`/`Address::from_hex()` for Phase 1 (simple, already exists on `Address`)
     rather than bech32, to keep first-slice scope small — `Address::to_bech32()`/`from_bech32()`
     already exist too, so switching to real `sprax1...` addresses later is a small follow-up, not
     a redesign.
   - `Querier`: Phase 1 scope is minimal — support a bank-balance query via `StateAccessor::get_account`
     (from `sprax-core`, only reachable if `sprax-wasm` takes that dependency — **open question,
     see Decisions Needed below**), return `SystemResult::Err(Unsupported)` for everything else.

2. **`crates/sprax-wasm/src/vm.rs` (rewrite)** — `WasmContractEngine` currently holds its own
   in-memory `HashMap`s (`codes`, `contracts`) and non-deterministic counters (`next_code_id`,
   `instance_nonce`) that are **never persisted and never shared across nodes** — this is a real
   correctness gap for a multi-node chain, not just "stub code": two nodes executing the same tx
   history would assign different code IDs/contract addresses. Fix, decided this session:
   - `CodeId` becomes `cosmwasm_std::Checksum` (content-addressed hash of the bytecode) instead of
     an auto-incrementing `u64` — eliminates the non-determinism outright, no shared counter needed.
   - Contract address derivation should use the sender's already-deterministic tx nonce (tracked in
     `AccountState.nonce`) instead of an in-memory `instance_nonce` counter — needs the nonce value
     threaded in from `executor.rs`.
   - All engine state (code bytecode + metadata, contract metadata, contract KV storage) must move
     into the shared `KVStore` so it persists across restarts and stays part of `compute_root()` —
     `WasmContractEngine` should end up **stateless** (no `RwLock<HashMap<...>>` fields), taking
     `store: &S` per call, matching how `TxExecutor`/`ChainLedger` are already structured.
   - Real bytecode validation: use `cosmwasm_vm::Cache` (`unsafe fn new(CacheOptions)`, needs a
     per-node on-disk cache directory — standard practice, not a scope concern) rather than the
     lower-level `internals::check_wasm` (explicitly an unstable/semver-exempt module in this
     crate — too fragile to build Phase 1 on). `Cache::store_code(wasm, checked=true, persist=false)`
     validates + compiles + returns the canonical `Checksum` in one call.
   - Reentrancy guard (`enter_call`/`exit_call`/`active_call_stack`) can be dropped for Phase 1 —
     nothing can make a nested contract call yet since cross-contract sub-messages aren't
     implemented (see Explicitly Out of Scope below), so the guard currently protects against a
     scenario that can't happen. Note when sub-message dispatch is eventually built.

3. **`crates/sprax-types/src/transaction.rs`** — extend `TxMessage`: add `StoreCode { wasm_bytecode: Vec<u8> }`,
   `InstantiateContract { code_id, msg: Vec<u8>, funds: Amount, label: String }`, add `funds: Amount`
   to the existing `ContractCall { contract, data }` variant. Additive, non-breaking (transient tx
   data, and existing match sites already have a catch-all arm).

4. **`crates/sprax-core/src/executor.rs`** — replace the no-op catch-all (currently around
   lines 154-172, may have shifted slightly) with real calls into `sprax-wasm`. **Requires
   `sprax-core` to add a dependency on `sprax-wasm`** — confirmed clean (one-directional, no cycle:
   `sprax-wasm` only depends on `sprax-types`/`sprax-crypto`/`sprax-storage`).

5. **Real example contract** — `contracts/examples/cw20-basic/` (new crate, real `cosmwasm-std`
   `#[entry_point]`s, compiled to `wasm32-unknown-unknown`). Needed as the end-to-end proof fixture;
   supersedes `crates/sprax-wasm/src/contracts/token.rs` (a hand-simulated, non-WASM fixture — kept
   for now per the "flag, don't silently delete" call in the plan). Requires adding
   `targets = ["wasm32-unknown-unknown"]` to root `rust-toolchain.toml`.

6. **`crates/sprax-cli`** — `contract store-code / instantiate / execute / query` subcommands, per
   `TODO.md`'s explicit ask. Reuses the existing `submit_transaction`/`mine_block` CLI flow already
   used for transfers.

7. **Tests** — adapter unit tests against `MemKVStore`; a `sprax-core` integration test that stores
   the compiled example contract, instantiates/executes/queries it and asserts non-flat gas
   consumption; a determinism test mirroring `test_scenario_11_apply_block_matches_mine_block_reward`
   (mine_block vs apply_block on two independent ledgers must reach identical state roots).

## Decisions needed before resuming (flag to user, don't just pick silently)

- **Querier's bank-balance query needs `sprax-core::StateAccessor::get_account`, but `sprax-wasm`
  currently has zero dependency on `sprax-core`.** Two options: (a) give `sprax-wasm` a dependency
  on `sprax-core` too (then both `sprax-core → sprax-wasm` for message execution AND
  `sprax-wasm → sprax-core` for queries — a **cycle**, not allowed by Cargo); or (b) have
  `executor.rs` inject a small closure/trait object for "get account balance" into the `Querier`
  adapter at call time, so `sprax-wasm` only depends on a tiny local trait, not all of `sprax-core`.
  (b) is almost certainly right but wasn't finalized.
- Confirm the Phase 1 scope trade-offs already made are still acceptable: hex (not bech32)
  addresses in the `BackendApi` adapter, `Cache`-based validation with a new on-disk cache
  directory per node, and dropping the reentrancy guard until sub-message dispatch exists.

## Explicitly out of scope for Phase 1 (already agreed, not forgotten)

`reply`/`sudo`/`migrate`/IBC entry points, cross-contract sub-message dispatch, removing/rewriting
the existing hand-simulated `crates/sprax-wasm/src/contracts/{token,escrow,governance}.rs` fixtures.

## Broader project gaps (unrelated to the VM work, found during an earlier audit — still open)

- No mainnet validator set defined yet (`genesis.json` has `PLACEHOLDER` values).
- No genuine independent third-party security audit (one existing "audit report" in the repo is
  self-authored, not real).
- `apps/web-wallet` is thin (6 source files) relative to the other apps.
