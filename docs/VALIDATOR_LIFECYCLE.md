# Canonical validator lifecycle and slashing

Implemented for a coordinated new network revision. This is not mainnet audit clearance.

## Policy and state transitions

The user-selected default is **500 basis points (5%) plus permanent tombstoning** for proven
same-height, same-round, same-step double signing. The genesis `validator_policy` also contains
`min_self_stake` (default 1 SPRX) and `jail_blocks` (default 10). These are development defaults;
operators must agree production parameters and the resulting genesis fingerprint.

Registration is an operator-signed `RegisterValidator` transaction. It reserves self-stake,
checks a genesis/operator/nonce-bound Ed25519 proof, and rejects reused operator addresses or
consensus keys, including tombstoned keys. This revision requires the consensus key to derive
the operator address; separate operator/consensus keys and key rotation are not supported.
The registry is capped at 1,024 entries, with at most 100 eligible active validators. Selection
is by committed stake and deterministic address ordering. A transaction finalized at height H
changes the active set for H+1; H is still certified by its pre-execution set.

`JailValidator` is an operator-authorized voluntary jail. Falling below minimum self-bond also
jails the operator. `UnjailValidator` requires the operator signature, sufficient self-bond and
expiry of the genesis cooldown. A jailed operator can top up its self-bond. A tombstoned
operator cannot unjail, receive new delegations, or register again. Remaining funds can unbond.
Automatic missed-block/downtime jailing and penalties are not implemented.

## Evidence processing

Peer observations never mutate canonical stake. A funded reporter submits a signed
`SubmitEquivocationEvidence` transaction containing JSON-encoded `EquivocationEvidence` bytes.
The observation pool/RPC supports registered validators, not only genesis validators. It is
bounded, nonpersistent and prunes expired/tombstoned observations. Inclusion is **not automatic**:
an operator or relayer must submit a fee-paying transaction before the evidence expires.

Execution checks both signatures, genesis, all equivocation coordinates, and membership in the
committed validator-set snapshot for the infraction height. Evidence must satisfy
`0 < infraction_height < processing_height` and
`processing_height - infraction_height < unbonding_period_blocks`. Snapshots outside that
window are removed deterministically. The unbonding period must be at least two blocks;
the default ten-block period is intended for development, not a production evidence SLA.

The 5% fraction is applied to current bonded balances and pending unbondings created at or
after the infraction height. Pending balances prevent moving stake out just before evidence
inclusion from escaping its penalty. Each record rounds down to whole atto-SPRX. Bonded
validator tokens, delegation balances/shares, pending withdrawals, circulating supply and
burn accounting change together. A permanent tombstone prevents a second penalty, including
reversed or alternative evidence pairs. A receipt records evidence hash, offense/processing
heights and total burned. No reporter reward or inflation is introduced.

Proposal construction/validation uses overlays. Gossip, rejected evidence, gas exhaustion,
failed transactions and failed block application cannot leave partial penalties behind.
The executor meters each slash record; per-validator delegation/unbonding indexes are capped
at 1,024 entries each. Multiple unbonds sharing transaction coordinates merge rather than
overwriting and losing a pending withdrawal.

This follows the distinction between observed evidence and committed economic processing
also described in the [Cosmos staking specification](https://github.com/cosmos/cosmos-sdk/blob/main/x/staking/README.md).
It is an independent implementation, not a claim of compatibility with Cosmos.

## Operator commands

Use an existing local keyring. Obtain the agreed genesis hash from the operators' ceremony,
then compare it with `sprax validator policy`; do not trust an unknown endpoint as the source
of the agreed hash. CLI amounts and fees use integer atto-SPRX, with no floating-point conversion.

```text
sprax validator --rpc-url http://127.0.0.1:26657 policy
sprax validator --rpc-url http://127.0.0.1:26657 list
sprax validator --home .sprx --from validator --genesis-hash HASH register --self-stake-atto 100000000000000000000 --moniker operator-1
sprax validator --home .sprx --from validator --genesis-hash HASH jail
sprax validator --home .sprx --from validator --genesis-hash HASH unjail
sprax validator --home .sprx --from delegator --genesis-hash HASH delegate --validator ADDRESS --amount-atto 1000000000000000000
sprax validator --home .sprx --from delegator --genesis-hash HASH unbond --validator ADDRESS --amount-atto 1000000000000000000
sprax validator evidence
sprax validator --home .sprx --from reporter --genesis-hash HASH submit-evidence --file evidence.json
```

`evidence.json` contains **one evidence object**, selected from the evidence query's array.
A broadcast acknowledgment means mempool admission. Confirm a finalized transaction receipt
before relying on registration, penalties, or eligibility. RPCs `sprax_getValidatorRegistry`,
`sprax_getValidatorPolicy`, and `sprax_getEquivocationEvidence` expose the relevant public data.
`sprax_getStaking` and `sprax_getDelegations` read committed ledger state, including penalties,
instead of the old in-memory staking display cache.

## Verification and compatibility

Regression coverage includes registration proof failure, duplicate registration, jail/unjail
cooldown, self-bond exit, historical membership, forged/foreign/future/expired/replayed evidence,
proposal isolation, transaction and mid-loop gas rollback, unbonding evasion, token conservation,
next-height certificates, disk reopen, catch-up and canonical staking RPC values. A driver test
checks that a jailed validator releases no new votes/proposals. Linux workspace CI remains the
integration gate; adversarial multi-operator transitions and independent economic/security
review are still required before launch.

Registry, indexes, historical snapshots and policy are new consensus state. Genesis fingerprints
and state roots change. Existing database identities reject this revision; no automatic migration
exists. Coordinate a new testnet/genesis or a separately reviewed migration. Never delete or
rewind an existing validator signing journal to force a restart. Preserve old homes/backups.
