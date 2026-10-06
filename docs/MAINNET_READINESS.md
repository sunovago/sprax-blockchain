# Mainnet readiness

Updated: 2026-10-06. Status: INCOMPLETE. This file supersedes historical "12/12 complete",
"genesis ready", and "cleared for mainnet" claims.

## Current hardening changes

- Transactions execute against an isolated storage overlay. Failed execution discards
  balance, nonce, supply, delegation, and unbonding changes.
- Proposal construction and validation do not advance finalized height or persist state.
- Finalized blocks commit state, block data, transaction indexes, and height in one redb
  transaction. Memory caches are updated only after that transaction succeeds.
- Validator identities/power come from the genesis registry and committed ledger stake.
  A block commits the active set from state at height H-1; delegation changes take effect
  in the certificate for height H+1. Top-100 selection and ordering are deterministic.
  Commit verification, block gossip, historical catch-up, startup and validator RPCs use
  the same canonical set instead of trusting staking.json or an in-memory staking cache.
- Round proposer selection is deterministically weighted from genesis fingerprint, height,
  round and the canonical active-set commitment. It no longer depends on process-local DWRR
  priority, so restarts and missed rounds do not reset a node onto a different proposer.
  This replaces the earlier DWRR schedule and requires a coordinated protocol upgrade.
- Signed equivocation observations are bounded/deduplicated in a local memory pool. Peer
  arrival never changes stake, jail status, state root or consensus power. The unsafe direct
  ledger slash API was removed. Finalized, chain-bound economic evidence processing remains
  unfinished; the observation pool is not persistent and does not implement slashing.
- Vote signatures use the `sprax/vote/v2` domain and include the exact genesis fingerprint,
  validator address, step, height, round and optional block hash. Nil votes, certificates and
  equivocation observations cannot be reused on another genesis. Old wire votes without
  genesis are rejected; signing journal identities include signing version 2 and reject
  legacy identities without resetting existing history. This is a coordinated protocol change.
- The consensus driver waits for precommit quorum before applying a proposal.
- Block commit certificates carry the consensus round and are checked for valid signatures,
  unique known signers, a consistent round, and more than two-thirds voting power.
- Non-development nodes and consensus-enabled nodes reject uncertified block gossip.
  Development nodes with consensus disabled retain explicit local/manual mining support.
- Proposal timeouts continue collecting votes, emit nil precommits after prevote timeout,
  and finish a nil-quorum round without finalizing a block. A quorum for unknown proposal
  data cannot establish a signing lock; a lock is recorded after validated, durable precommit.
  Round numbers cannot regress and retained vote history is bounded to current/locked/valid
  rounds. Restarts account for proposal-only signing history, not only the last signed vote.
- Consensus locks survive retries at the same height; conflicting proposals receive nil
  prevotes unless a later-round signed prevote quorum certifies a replacement block. The
  signing journal verifies that certificate and durably stores the replacement block and
  lock before releasing the precommit signature. The driver now bounds future-round vote
  retention and jumps to a later round only after authenticated +2/3 prevotes agree on one
  block hash or nil. On entry, same-round certificates and buffered prevotes (including nil)
  are replayed before local voting, and expired round buffers are removed. Four-validator
  signed-message driver tests cover jump-to-finalization and lock-preserving nil transitions.
  The signing journal also verifies the replacement block and later-round quorum before
  signing a conflicting prevote. Such a prevote preserves the old lock across reopen; only
  a durably signed non-nil precommit replaces it. Driver tests cover both initially locked
  and unlocked validators participating in certified-round finalization.
  Authenticated, ledger-valid proposals that arrive before their round are retained for
  the current height: at most eight rounds, a 32-round lookahead and 8 MiB of serialized
  proposal data. Cached proposals do not advance rounds or create locks; they are checked
  again on consumption. Driver tests finalize from an early cached proposal even after
  its inbound channel closes, with both locked and unlocked validators. Proposals never
  received at all still need a peer request/retransmission mechanism. TCP partition and
  multi-node restart/catch-up tests remain outstanding.
- Validator votes are durably journaled before broadcast. Restarts restore the signing
  coordinates and precommit lock; conflicting/reversed coordinates are refused. The
  signing database is bound to the genesis and validator public key. A durable marker
  rejects accidental journal deletion. Restoring old backups of both files is not protected.
- Proposals carry validator signatures bound to the genesis, chain ID, height, round and
  block-header hash. The selected proposer is checked; invalid signatures are ignored
  while waiting for a valid proposal. Proposal v3 can carry a prior-round signed prevote
  quorum for the exact block, allowing validators with older locks to safely prevote it.
  Proposal signatures are durably journaled, and a conflicting proposal is accepted only
  with that later-round proof. Replacing a BFT lock still requires the later-round quorum
  and durable replacement state.
- Existing chain state is bound to the exact genesis fingerprint. Changed or missing
  fingerprints fail closed. Genesis self-stake is deducted from the operator allocation
  and credited to a self-delegation; unfunded or duplicate validators are rejected.
- Contract scans enforce 1,024 records / 1 MiB before copying records in memory, redb and
  overlays. Storage keys/values are capped at 1 KiB / 64 KiB; at most 16 active iterator
  handles are allowed. Scans use the requested key range and contract prefix.
- Live P2P channels are bounded. Archive catch-up serves one block per response with
  inbound backpressure. Mempool admission is capped at 128 transactions / 2 MiB per transaction.
- P2P handshakes reject frames over 4 KiB before allocating. Configured inbound/outbound
  connection limits also count pending handshakes; a direction is capped at 1,024 slots.
  Peer-advertised addresses must parse as socket addresses and the learned set is bounded.
- Steady-state TCP frames enforce the configured payload limit before allocation. Outgoing
  JSON serialization is bounded by the same limit. Duplicate connected peer IDs are rejected
  without removing another live connection routing entry. Simultaneous dials select the
  same TCP direction on both ends and disconnect the redundant socket. Peer IDs remain
  self-asserted; authenticated transport identity still needs implementation.
- P2P shutdown cancels listener waits, pending handshakes and active connections. Stop
  generations survive rapid restarts; failed listener binds reset the running state.
- CLI testnet/mainnet initialization requires a validated explicit genesis and rejects public
  development validator keys. Unknown environments, chain-ID mismatches and nonempty node
  homes fail before writes. Non-development initialization creates no development keys and
  leaves signing disabled until operator configuration. CLI port overrides affect listeners.
  Containers quote argument arrays and require explicit non-development initialization;
  the daemon handles both SIGINT and SIGTERM for shutdown.
- Shutdown aborts and joins service-owned consensus/gossip/evidence tasks before returning,
  releasing the validator signing database before an immediate restart. Runtime startup errors
  share the same cleanup path instead of leaving partially started consensus tasks online.
- Node startup reports P2P/RPC bind failures instead of reporting an online service. RPC
  handles drain on stop, allowing listener reuse. An explicitly configured signing key must
  exist and match an active validator; the selected home governs the journal path after moves.
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
- Transaction selection reserves a serialized worst-case certificate for every active
  validator, including maximum round/timestamp widths and signature byte-array encoding.
  Transactions that only fit an unsigned proposal remain pending. Certificate verification
  rejects excessive signer counts and non-64-byte signatures before hashing/signature work.
- Testnet/mainnet require explicit genesis; known public development validator keys are
  rejected. Fresh non-development keyrings do not seed development private keys.

## Verification

Latest hardening checkpoint: revision `74de42faef3273dc71b1b03ff298a41d4189cad1`.
[Mainnet hardening verification](https://github.com/sunovago/sprax-blockchain/actions/runs/37434790190),
[Protocol CI, including Rust integration tests and Docker builds](https://github.com/sunovago/sprax-blockchain/actions/runs/37434790634),
and [all four CodeQL language analyses](https://github.com/sunovago/sprax-blockchain/actions/runs/37434790217)
completed successfully. New regressions cover locked-block recovery/reproposal and proposal-carried
valid-round certificates. CodeQL workflow success does not clear the repository's open findings
or establish independent audit clearance.

The proposer schedule and proposal v3 valid-round certificate are protocol changes: validator
operators must coordinate the upgrade. Existing mainnet readiness remains incomplete pending
multi-validator round-transition/reproposal/catch-up testing, finalized canonical slashing,
broader fault and recovery testing, audit/remediation, and launch operations.

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
Signed-proposal and recovery verification are included in the latest checkpoint above.

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
3. Complete and test BFT round synchronization, future-proposal recovery and multi-validator locked proposal recovery,
   reproposal/catch-up after restart, commit validation during historical catch-up,
   and complete validator registration/jail/tombstone transitions and canonical slashing.
   Delegation-driven power commitments and sequential certificate catch-up are implemented;
   peer observations are quarantined and have no economic effect.
4. Exercise three or more active validators through partitions, Byzantine input, proposer
   failures, restarts, catch-up, and sustained load. Existing two-validator-plus-observer tests
   do not establish these properties.
5. Finish storage recovery, pruning/rebuild compatibility,
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

Vote signing version 2 and proposal signing version 3 replace the older encodings. Mixed-version
consensus, old commit certificates and legacy signing journals are incompatible. Never delete
or reset an existing signing journal to bypass this rejection. A fresh coordinated network
with new operator keys or a separately reviewed migration is required; automatic migration is
absent.

The canonical validator commitment encoding replaces the historical genesis-validator JSON
hash and the copied-parent header placeholder. Restart rejects an old genesis commitment,
invalid persisted ancestry, and state that differs from the finalized tip. Existing block histories require an explicit,
reviewed migration; do not mix binaries with the two commitment encodings.

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
