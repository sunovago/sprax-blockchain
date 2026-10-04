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
- Unsupported messages fail instead of returning successful no-op receipts. ContractCall
  remains disabled pending actual WASM execution.
- Expiration, transaction gas, account nonce exhaustion, block gas/size, increasing block
  timestamps, and bounded unique validator voting power are checked.
- Testnet/mainnet require explicit genesis; known public development validator keys are
  rejected. Fresh non-development keyrings do not seed development private keys.

## Verification

Rust regression tests were added for rejected block rollback, proposal isolation, repeated
unbond failure, unsupported message rejection, nonzero-round commit verification, certificate
forgery/duplication/replay, and invalid validator sets.

Rust formatting parses the changed sources. Compilation and test execution are NOT verified:
Windows Application Control blocked Cargo-generated build executables with OS error 4551.
Run the existing CI or an approved Rust environment before accepting these changes:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

The preceding application review passed backend (83), explorer (28), admin (5), wallet SDK
(12), and Flutter (37) tests, plus frontend builds and Flutter analysis. These results do not
verify the chain changes above or a live deployment.

## Remaining engineering gates

1. Execute all Rust tests and static checks; fix compilation, behavioral, and network failures.
2. Implement real CosmWasm execution, persistent content-addressed code and contract state,
   deterministic addresses, metered queries/execution, CLI/RPC integration, real compiled
   contract fixtures, restart tests, and cross-node state-root equivalence.
3. Complete BFT round synchronization, locked proposal handling, authenticated proposals,
   durable signing protection across restarts, commit validation during historical catch-up,
   and deterministic validator-set transitions and slashing. Current staking-cache and
   peer-evidence updates are not sufficient evidence of consensus-safe economic transitions.
4. Exercise three or more active validators through partitions, Byzantine input, proposer
   failures, restarts, catch-up, and sustained load. Existing two-validator-plus-observer tests
   do not establish these properties.
5. Verify genesis accounting/validator commitments, configured genesis identity on restart,
   storage recovery, pruning/rebuild compatibility, bounded mempools/network queues, block
   resource limits including commit overhead, and performance of archive/overlay reads.
6. Validate the full Linux deployment: reproducible images, node/backend/database/indexer
   connectivity, TLS, backups/restores, monitoring, alerting, and upgrade procedures.
7. Finish wallet persistence/unlock and real transaction integration. Telegram mining is
   off-chain points; a chain-backed token claim flow is a separate unfinished feature.

## External launch inputs

- Actual validator operators, public consensus keys, allocation addresses, bootnodes, and
  agreed genesis parameters/checksum. Do not paste private keys or seed phrases into chat.
- An independent audit with an identified auditor, reviewed revision, and remediation evidence.
- Stable public testnet evidence and an operator-run key/genesis ceremony.

Passing local tests or producing configuration files alone does not close these gates.
