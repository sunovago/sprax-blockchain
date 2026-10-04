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
`codex/mainnet-hardening-20261004`. Revision `bd9e105ce3defaaf772b46fc93ce4d0ce3e8c1b1`
passed formatting, all workspace tests (including the compiled real WASM fixture), strict
Clippy with warnings denied, and the admin/explorer/web-wallet/SDK build and test jobs.
[Verified GitHub Actions run](https://github.com/sunovago/sprax-blockchain/actions/runs/37187489667).
Subsequent backend dependency/documentation changes require their own final CI result;
passing this revision does not establish overall mainnet readiness.

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
   resource-bounded iteration, compilation caching, and execution performance testing.
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

## Application dependency work

The SDK and web-wallet npm audits reported zero findings after development dependency
updates. Admin/explorer retain high-severity findings in the Tailwind 3 dependency tree;
the local Telegram miniapp also requires its own framework/security upgrade. A successful
build does not mean all application dependency findings are resolved.
