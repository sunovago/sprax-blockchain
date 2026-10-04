# Mainnet readiness

Updated: 2026-10-04. Status: INCOMPLETE. This file supersedes historical "12/12 complete",
"genesis ready", and "cleared for mainnet" claims.

## Current hardening changes

- Transactions execute against an isolated storage overlay. Failed execution discards
  balance, nonce, supply, delegation, and unbonding changes.
- Proposal construction and validation do not advance finalized height or persist state.
- Finalized blocks commit state, block data, transaction indexes, and height in one redb
  transaction. Memory caches are updated only after that transaction succeeds.
- The consensus driver waits for precommit quorum before applying a proposal.
- Block commit certificates carry the consensus round and are checked for valid signatures,
  unique known signers, a consistent round, and more than two-thirds voting power.
- Non-development nodes and consensus-enabled nodes reject uncertified block gossip.
  Development nodes with consensus disabled retain explicit local/manual mining support.
- Consensus locks survive retries at the same height; conflicting proposals receive nil
  prevotes. Full round synchronization and unlocking/liveness still need work.
- Validator votes are durably journaled before broadcast. Restarts restore the signing
  coordinates and precommit lock; conflicting/reversed coordinates are refused. The
  signing database is bound to the genesis and validator public key. A durable marker
  rejects accidental journal deletion. Restoring old backups of both files is not protected.
- Proposals carry validator signatures bound to the genesis, chain ID, height, round and
  block-header hash. The selected proposer is checked; invalid signatures are ignored
  while waiting for a valid proposal. Proposal signatures are also durably journaled,
  with conflicting retries and regressions refused. This does not implement BFT unlocking.
- Existing chain state is bound to the exact genesis fingerprint. Changed or missing
  fingerprints fail closed. Genesis self-stake is deducted from the operator allocation
  and credited to a self-delegation; unfunded or duplicate validators are rejected.
- Contract scans enforce 1,024 records / 1 MiB before copying records in memory, redb and
  overlays. Storage keys/values are capped at 1 KiB / 64 KiB; at most 16 active iterator
  handles are allowed. Scans use the requested key range and contract prefix.
- Live P2P channels are bounded. Archive catch-up serves one block per response with
  inbound backpressure. Mempool admission is capped at 128 transactions / 2 MiB per transaction.
- The browser wallet persists only an encrypted vault, supports unlock and encrypted backups,
  and locks on tab hide or five minutes of inactivity. SDK address hashing, canonical
  transaction signing, and REST address/balance handling match the Rust node. Both Ed25519
  and Secp256k1 SDK transfers are verified on the Rust ledger.
- Unsupported messages fail instead of returning successful no-op receipts.
- StoreCode, InstantiateContract, and ContractCall use the actual pinned CosmWasm VM,
  content-addressed code, deterministic addresses, scoped persistent storage, and gas metering.
  Query RPC and contract CLI commands are implemented. Submessages, chain queries, replies,
  migrations, and IBC are explicitly unsupported in this execution profile.
- Expiration, transaction gas, account nonce exhaustion, block gas/size, increasing block
  timestamps, and bounded unique validator voting power are checked.
- Testnet/mainnet require explicit genesis; known public development validator keys are
  rejected. Fresh non-development keyrings do not seed development private keys.

## Verification

Rust regression tests were added for rejected block rollback, proposal isolation, repeated
unbond failure, unsupported message rejection, nonzero-round commit verification, certificate
forgery/duplication/replay, and invalid validator sets.

Windows Application Control blocks local Cargo-generated build executables with OS error 4551.
Verification uses the authorized GitHub Actions Linux environment on
`codex/mainnet-hardening-20261004`. Revision `b6b33fa11f47e89a3d651c029ae7fdfacf4c79bd`
passed formatting, all workspace tests (including the compiled real WASM fixture), strict
Clippy with warnings denied, and the admin/explorer/web-wallet/SDK build and test jobs.
[Verified hardening run](https://github.com/sunovago/sprax-blockchain/actions/runs/37187790914).
[Protocol CI](https://github.com/sunovago/sprax-blockchain/actions/runs/37187794189) also passed,
including backend tests, frontend builds, security/secret scan, and Docker image builds.
[CodeQL analysis](https://github.com/sunovago/sprax-blockchain/actions/runs/37187794167) completed
successfully but reported 12 open findings; see [source triage](CODEQL_TRIAGE.md).
Passing these jobs does not establish overall mainnet readiness or independent audit clearance.

The additional journal/genesis/accounting/scan/queue/wallet changes at
`cf815398bf68b44847e5f088c60e47b64eaef647` passed the full workspace tests, strict Clippy,
reproducible SDK signing fixtures, and frontend builds/tests in the
[extended verification](https://github.com/sunovago/sprax-blockchain/actions/runs/37190567013).
The SDK has 26 tests and browser wallet has 3 lifecycle/storage tests at this revision.
Wallet REST routing and mempool capacity tests also passed at
`df7478654adca5540b057df0a21350b6702d9d79` in the
[REST integration verification](https://github.com/sunovago/sprax-blockchain/actions/runs/37190819979).
Signed-proposal verification is tracked in subsequent branch runs.

```sh
cargo fmt --all -- --check
bash scripts/build_wasm_fixture.sh # requires Binaryen 123 wasm-opt
export SPRX_TEST_CONTRACT_WASM="$PWD/contracts/examples/counter/target/wasm32-unknown-unknown/release/sprax_counter.wasm"
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

The preceding application review passed backend (83), explorer (28), admin (5), wallet SDK
(12), and Flutter (37) tests, plus frontend builds and Flutter analysis. These results do not
verify the chain changes above or a live deployment.

## Remaining engineering gates

1. Extend verification with adversarial/fuzz tests, cross-platform execution equivalence,
   resource-exhaustion tests, and operator-run recovery/soak tests. Current regression and
   strict static checks passed for the revision above.
2. The compiled counter, replica roots, disk restart, failed-write rollback, and gas-exhaustion
   rollback are verified. Complete chain query/submessage/migration capabilities, contract events,
   compilation caching, and execution performance testing. Bounded iteration is implemented;
   pagination beyond the configured scan limits remains a future capability.
3. Complete BFT round synchronization and locked proposal handling,
   safe unlocking/reproposal after restart, commit validation during historical catch-up,
   and deterministic validator-set transitions and slashing. Current staking-cache and
   peer-evidence updates are not sufficient evidence of consensus-safe economic transitions.
4. Exercise three or more active validators through partitions, Byzantine input, proposer
   failures, restarts, catch-up, and sustained load. Existing two-validator-plus-observer tests
   do not establish these properties.
5. Finish per-height validator commitments, storage recovery, pruning/rebuild compatibility,
   byte budgets and overload recovery for bounded network queues, block resource limits
   including commit overhead, and performance of archive/overlay reads. Genesis accounting
   and configured genesis identity on restart have regression coverage.
6. Validate the full Linux deployment: reproducible images, node/backend/database/indexer
   connectivity, TLS, backups/restores, monitoring, alerting, and upgrade procedures.
7. Validate wallet transactions through a deployed node/browser and a hardened public RPC.
   Persistence/unlock and SDK-to-Rust transfer integration are implemented. Telegram mining is
   off-chain points; a chain-backed token claim flow is a separate unfinished feature.

## External launch inputs

- Actual validator operators, public consensus keys, allocation addresses, bootnodes, and
  agreed genesis parameters/checksum. Do not paste private keys or seed phrases into chat.
- An independent audit with an identified auditor, reviewed revision, and remediation evidence.
- Stable public testnet evidence and an operator-run key/genesis ceremony.

Passing local tests or producing configuration files alone does not close these gates.

## Upgrade compatibility

These protocol changes require a coordinated new testnet/genesis or a separately reviewed
migration. Legacy databases without the genesis identity are rejected, never silently reset.
Keep existing directories and backups. Preserve the validator signing database and its
`.initialized` marker with the validator key; never restore an older journal and resume signing.
See [wallet security and recovery](WALLET_SECURITY.md) for the SDK address compatibility change.

## Application dependency work

The SDK and web-wallet npm audits reported zero findings after development dependency
updates. Admin/explorer retain high-severity findings in the Tailwind 3 dependency tree;
the local Telegram miniapp also requires its own framework/security upgrade. A successful
build does not mean all application dependency findings are resolved.
