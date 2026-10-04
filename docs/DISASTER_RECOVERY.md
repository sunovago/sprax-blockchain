# Node recovery procedure

Status: engineering runbook for the hardening branch, not an operator-certified mainnet
recovery guarantee. See [mainnet readiness](MAINNET_READINESS.md). The historical snapshot
CLI, automated HSM failover and recovery-time guarantees in this document were not
implemented and must not be used as operational instructions.

## Before copying a validator home

Fence the original validator: ensure the old process and any standby using its key cannot
sign. Stop the daemon with SIGINT or SIGTERM and wait for process exit. Do not copy a live
redb database with ordinary file-copy commands. Keep the exact binary revision and agreed
genesis with the backup. A complete stopped home contains:

- `config.toml` and `genesis.json`;
- `data/state.redb`, containing canonical state and committed block/index metadata;
- `data/validator-signing.redb` and `data/validator-signing.initialized`;
- `keyring/keys.json`, which contains local private key material and requires restricted,
  encrypted offline backup storage.

The initialization marker replaces the `.redb` extension; it is not named
`validator-signing.redb.initialized`. The current signer is a local software keyring.
Remote signing/HSM integration has not been implemented.

## Restore without resetting signing history

Work on a copy, preserving the failed disk and logs. Keep consensus signing disabled while
verifying the genesis checksum, expected chain ID and committed-height/state-root evidence
from trusted operators. Starting an existing home uses `sprax start --home /path/to/home`;
`sprax init` rejects a nonempty home and must not be used as a recovery command. The home
selected on the command line governs state and signing journal paths after directory moves.

Recover the latest intact signing database and its marker, independently of an older ledger
backup. Never resume signing with a signing journal from an earlier snapshot: it can forget
votes/proposals already released to the network. Restoring both an old database and marker
is not detected by the current local journal. If the latest signing history is unavailable,
leave signing disabled and coordinate an independently reviewed validator-key transition.
Do not delete the journal or marker to make startup succeed.

The node refuses a changed genesis, a legacy database without its genesis identity, and a
missing signing database when its marker exists. Preserve these errors for investigation;
they do not authorize resetting the directory. Catch-up with historical validator changes
and locked proposals after restart still needs consensus work and adversarial verification.

## Verify recovery in an isolated rehearsal

Use a stopped-home copy, restricted networking and an observer configuration. Compare the
reopened committed height, block hash and state root with the original backup evidence.
Check that tampered genesis, missing signing database and conflicting/reversed signing
coordinates are refused. Rehearse SIGTERM shutdown and listener reuse, then record the
revision, public genesis hash, observations and operator acceptance. Automated redb restart
and signing-journal regressions do not replace this operator-run exercise.

Do not promise a recovery time or zero data loss until a real deployment has completed these
rehearsals. There is no `sprax snapshot download` command in the current CLI.
