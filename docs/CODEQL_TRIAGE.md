# CodeQL review for the hardening revision

Reviewed revision: `b6b33fa11f47e89a3d651c029ae7fdfacf4c79bd`, PR #4.
All language analysis jobs completed successfully. The PR scan reports 12 open findings.
Passing the workflow does not mean zero findings. No alerts were dismissed by this review.

| Findings | Rule and location | Source assessment |
| --- | --- | --- |
| 4 critical | `rust/hard-coded-cryptographic-value`, `crates/sprax-core/tests/wasm_execution_tests.rs` | These are transaction account sequence numbers, initially zero and then three for calls after three successful transactions. They are public replay-protection counters inside signed transaction bodies, not cipher nonces or signature ephemeral secrets. Failed calls intentionally reuse the unchanged account sequence to prove rollback. This is a false-positive classification for these locations. |
| 7 high | `rust/cleartext-logging`, CLI query/transfer commands | The output contains public on-chain balances, account sequence numbers, and reference-currency conversions that the invoked commands are designed to display. `AccountState` contains only nonce, balance, code hash, and storage root; `NodeService::get_account` returns this type. These findings do not identify exposure of a private key, password, or seed. |
| 1 high | `rust/cleartext-logging`, multi-node consensus integration test | The assertion's failure text prints Bob's public on-chain balance to diagnose transfer/reward accounting. It does not print signing material. |

This assessment covers the listed locations, not every logging path in the repository.
An independent security review and the remaining mainnet gates are still required.

Verification runs:

- [Hardening tests, lint, and frontend checks](https://github.com/sunovago/sprax-blockchain/actions/runs/37187790914)
- [Protocol CI, backend tests, and Docker builds](https://github.com/sunovago/sprax-blockchain/actions/runs/37187794189)
- [CodeQL analysis](https://github.com/sunovago/sprax-blockchain/actions/runs/37187794167)
