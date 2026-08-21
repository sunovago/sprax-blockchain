# Contributing to SPRX

Thank you for your interest in contributing to SPRX. This document describes how to set up the project, the standards contributions are held to, and the review process for changes across the monorepo.

**Contents:** [Project Overview](#project-overview) · [Before You Contribute](#before-you-contribute) · [Development Setup](#development-setup) · [Branching](#branching-strategy) · [Commits](#commit-convention) · [PR Rules](#pull-request-rules) · [Code Review](#code-review) · [Security](#security-contributions) · [Testing](#testing-requirements) · [Documentation](#documentation) · [Style Rules](#style-rules) · [Checklist](#contributor-checklist) · [License](#license-agreement)

## Project Overview

SPRX (**S**calable **P**rotocol for **R**eal-world **X**), also referred to as **Sprax Chain**, is a Layer-1 blockchain protocol. Its native asset is **SPRX**, a freely floating cryptocurrency with 18-decimal precision (see [docs/TOKENOMICS.md](docs/TOKENOMICS.md)). Start with [README.md](README.md) for the full architecture diagram and component map before working through this document.

This repository is a monorepo containing:

- **Rust blockchain core** — [crates/](crates/): `sprax-types`, `sprax-crypto`, `sprax-storage`, `sprax-network`, `sprax-consensus`, `sprax-core`, `sprax-node`, `sprax-cli`, `sprax-indexer`, `sprax-wasm`, `sprax-faucet`.
- **Flutter mobile wallet** — [apps/mobile-wallet/](apps/mobile-wallet/) (non-custodial Android/iOS wallet), plus a web wallet at [apps/web-wallet/](apps/web-wallet/) and a shared TypeScript wallet SDK at [packages/sprax-wallet-core/](packages/sprax-wallet-core/).
- **Explorer UI** — [apps/explorer-ui/](apps/explorer-ui/) (React/TypeScript/Vite).
- **Admin Panel** — [apps/admin-panel/](apps/admin-panel/) (React/TypeScript/Vite operations console).
- **FastAPI backend** — [backend/](backend/) (REST/WebSocket API, Alembic migrations, Celery workers).
- **Indexer** — the `sprax-indexer` crate and [backend/scripts/start_indexer.py](backend/scripts/start_indexer.py).
- **Smart contracts** — the `sprax-wasm` contract execution engine (see [docs/SMART_CONTRACTS.md](docs/SMART_CONTRACTS.md) and [docs/CONTRACT_SECURITY.md](docs/CONTRACT_SECURITY.md)).
- **Infrastructure** — [docker/](docker/), [deploy/](deploy/) (devnet/testnet/mainnet Docker Compose stacks, genesis files, nginx, Prometheus), and [scripts/](scripts/).

### Component Setup at a Glance

| Component | Path | Toolchain | Verify locally |
|---|---|---|---|
| Rust chain workspace | [crates/](crates/) | Rust ≥ 1.80.0 | `cargo test --workspace --all-targets` |
| Backend API | [backend/](backend/) | Python ≥ 3.11 | `pytest` |
| Mobile wallet | [apps/mobile-wallet/](apps/mobile-wallet/) | Flutter ≥ 3.10 / Dart ≥ 3.0 | `flutter test` |
| Explorer UI | [apps/explorer-ui/](apps/explorer-ui/) | Node 18+ / Vite / Vitest | `npm run build && npm run test` |
| Admin panel | [apps/admin-panel/](apps/admin-panel/) | Node 18+ / Vite / Vitest | `npm run build && npm run test` |
| Wallet SDK | [packages/sprax-wallet-core/](packages/sprax-wallet-core/) | Node 18+ / Jest | `npm run build && npm test` |
| Web wallet | [apps/web-wallet/](apps/web-wallet/) | Node 18+ / Vite | `npm run build` (no test runner yet) |

Full details, including lint/typecheck commands, are in [Development Setup](#development-setup) below.

## Before You Contribute

Before opening a pull request, please:

- Read [README.md](README.md) and the relevant documents in [docs/](docs/) for the component you're changing (e.g. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/CONSENSUS.md](docs/CONSENSUS.md), [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md), [docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md), [docs/CODING_STANDARDS.md](docs/CODING_STANDARDS.md)).
- Follow the security posture defined in [docs/SECURITY.md](docs/SECURITY.md) and [docs/BUG_BOUNTY.md](docs/BUG_BOUNTY.md) — see [Security Contributions](#security-contributions) below.
- **Never commit secrets** (API keys, database credentials, RPC endpoints with embedded auth, `.env` files — see `.env.example` for the template of what belongs in an untracked `.env`).
- **Never commit seed phrases or private keys**, including test/demo keys, in any wallet, node, or validator context.
- **Never modify consensus-critical code casually.** Changes to `sprax-consensus`, `sprax-core`'s state-transition logic, or block/transaction serialization require design discussion first — see [Code Review](#code-review).
- **Never change token supply, genesis parameters, or validator economics** (anything touched by [docs/TOKENOMICS.md](docs/TOKENOMICS.md), [docs/GENESIS.md](docs/GENESIS.md), [docs/VALIDATORS.md](docs/VALIDATORS.md), [docs/SLASHING.md](docs/SLASHING.md), or files under [deploy/genesis/](deploy/genesis/)) without prior maintainer approval.

## Development Setup

### Rust workspace (`crates/`)

Requires Rust stable ≥ 1.80.0 (see [rust-toolchain.toml](rust-toolchain.toml)). Install via `rustup default stable && rustup component add rustfmt clippy`.

```bash
cargo check --all-targets --all-features
cargo test --workspace --all-targets
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

These are the same commands run by the `check`, `test`, `fmt`, and `clippy` jobs in [.github/workflows/ci.yml](.github/workflows/ci.yml); the workspace builds with `RUSTFLAGS="-D warnings"` in CI, so treat compiler warnings as errors locally too.

### Backend (`backend/`)

FastAPI + SQLAlchemy + Alembic, Python ≥ 3.11 (see [backend/pyproject.toml](backend/pyproject.toml)).

```bash
cd backend
python -m venv .venv && source .venv/bin/activate   # or .venv\Scripts\activate on Windows
pip install -e ".[dev]"
pytest
ruff check .
mypy .
```

`ruff` and `mypy` are configured under `[tool.ruff]` / `[tool.mypy]` in `backend/pyproject.toml` and are installed via the `dev` extra; there is currently no dedicated lint/typecheck script beyond invoking them directly.

### Mobile wallet (`apps/mobile-wallet/`)

Flutter ≥ 3.10, Dart SDK ≥ 3.0 (see [apps/mobile-wallet/pubspec.yaml](apps/mobile-wallet/pubspec.yaml)).

```bash
cd apps/mobile-wallet
flutter pub get
flutter analyze
flutter test
```

### Explorer UI / Admin Panel (`apps/explorer-ui/`, `apps/admin-panel/`)

React + TypeScript + Vite + Vitest.

```bash
cd apps/explorer-ui   # or apps/admin-panel
npm install
npm run build   # tsc && vite build — this performs type-checking
npm run test    # vitest run
```

Note: these packages do not currently define separate `lint` or `typecheck` scripts — type-checking happens as part of `npm run build` via `tsc`. Don't invent a `npm run lint` invocation that isn't in `package.json`.

### Web wallet & wallet SDK (`apps/web-wallet/`, `packages/sprax-wallet-core/`)

```bash
cd packages/sprax-wallet-core && npm install && npm run build && npm test   # jest
cd apps/web-wallet && npm install && npm run build                          # tsc && vite build; no test script defined yet
```

## Branching Strategy

The CI workflow already targets `main` and `develop` (see [.github/workflows/ci.yml](.github/workflows/ci.yml)), so the recommended workflow is:

- `main` — stable, deployable.
- `develop` — integration branch for upcoming changes.
- `feature/<short-description>` — new functionality.
- `fix/<short-description>` — bug fixes.
- `security/<short-description>` — security-related changes (coordinate privately first if the change addresses an undisclosed vulnerability).
- `docs/<short-description>` — documentation-only changes.

Only `main` is guaranteed to exist today; treat the others as the expected naming convention for branches you create off `main`.

## Commit Convention

Follow [Conventional Commits](https://www.conventionalcommits.org/), matching the style already used in [docs/CODING_STANDARDS.md](docs/CODING_STANDARDS.md):

```
feat(crypto): add bip-44 key derivation path support
fix(consensus): correct proposer priority decrement in DWRR
security(node): patch RPC auth bypass in challenge verification
docs(architecture): update Mermaid diagram for indexer pipeline
refactor(storage): extract batch-write helper from ChainLedger
test(storage): add atomic batch rollback unit tests
chore(ci): bump rust-toolchain pin
```

## Pull Request Rules

- Keep PRs focused on a single logical change; avoid bundling unrelated fixes or rewrites.
- Write a clear description of what changed and why; link the related issue if one exists.
- Include tests for new behavior or bug fixes (see [Testing Requirements](#testing-requirements)).
- No secrets, credentials, or private keys anywhere in the diff.
- No unrelated formatting-only rewrites of files you didn't otherwise touch.
- No generated/build artifacts (`target/`, `build/`, `node_modules/`, `.dart_tool/`) — check `.gitignore` coverage before committing.
- Update documentation when behavior changes (see [Documentation](#documentation)).
- Resolve all critical warnings before requesting review — CI runs Rust checks with `-D warnings` and Clippy with `-D warnings`, so warnings fail the build outright.
- Your branch must pass the relevant CI jobs before merge (see [.github/workflows/ci.yml](.github/workflows/ci.yml)):

| Job | Checks |
|---|---|
| `check` | `cargo check --all-targets --all-features` |
| `test` | `cargo test --all-targets --all-features` |
| `fmt` | `cargo fmt --all -- --check` |
| `clippy` | `cargo clippy --all-targets --all-features -- -D warnings` |
| `backend-test` | `pytest` against the FastAPI backend |
| `frontend-build` | Builds wallet SDK, web wallet, explorer UI, admin panel |
| `docker-build` | Builds the node and backend Docker images |
| `security` | TruffleHog verified-secret scan across the diff |

## Code Review

The following areas require extra scrutiny and at least one maintainer review beyond a standard approval, because mistakes here are costly or hard to reverse:

- Consensus logic (`sprax-consensus`, proposer election, voting power, slashing evidence).
- Cryptography (`sprax-crypto`: signing, hashing, key derivation, zeroization).
- Transaction and block serialization (`sprax-types`, `sprax-core`).
- Address format (bech32 encoding/decoding).
- Staking and validator logic (`sprax-consensus`, [docs/VALIDATORS.md](docs/VALIDATORS.md), [docs/SLASHING.md](docs/SLASHING.md)).
- Token supply and genesis parameters ([docs/TOKENOMICS.md](docs/TOKENOMICS.md), [docs/GENESIS.md](docs/GENESIS.md), [deploy/genesis/](deploy/genesis/)).
- Wallet signing paths (`packages/sprax-wallet-core`, `apps/mobile-wallet`, `apps/web-wallet`).
- Perps/risk logic (e.g. `apps/mobile-wallet/lib/services/perps_service.dart`, [docs/PERPS_STATUS.md](docs/PERPS_STATUS.md)).
- Mainnet configuration ([deploy/docker-compose.mainnet.yml](deploy/docker-compose.mainnet.yml), [docs/MAINNET.md](docs/MAINNET.md), [docs/MAINNET_SECURITY_CHECKLIST.md](docs/MAINNET_SECURITY_CHECKLIST.md)).

## Security Contributions

**Do not open public GitHub issues for sensitive vulnerabilities.** Report them privately through the responsible disclosure process defined in [docs/BUG_BOUNTY.md](docs/BUG_BOUNTY.md). Refer to [docs/SECURITY.md](docs/SECURITY.md) for the current threat model and cryptographic standards that inform how a report will be triaged.

## Testing Requirements

Minimum expectations by component:

- **Rust crates**: unit tests for new/changed public functions (happy path, edge cases, malformed input) plus `cargo test --workspace --all-targets`; multi-crate interactions belong in integration tests per [docs/CODING_STANDARDS.md](docs/CODING_STANDARDS.md).
- **Backend**: `pytest` coverage for new endpoints/services under `backend/tests/`.
- **Mobile wallet**: `flutter test` coverage for new services/widgets under `apps/mobile-wallet/test/`.
- **Explorer UI / Admin Panel**: `vitest` coverage (`npm run test`) for new components/hooks.
- **Wallet SDK**: `jest` coverage (`npm test` in `packages/sprax-wallet-core`) for new signing/derivation logic.
- **Web wallet**: no test runner is configured yet — describe manual verification steps in the PR description until one exists.

## Documentation

Update the corresponding file in [docs/](docs/) whenever a change affects:

- Public API behavior — [docs/EXPLORER_API.md](docs/EXPLORER_API.md), [docs/ADMIN_API_CONTRACT.md](docs/ADMIN_API_CONTRACT.md).
- RPC surface — [docs/WALLET_RPC.md](docs/WALLET_RPC.md).
- Transaction format — [docs/TRANSACTION_FLOW.md](docs/TRANSACTION_FLOW.md).
- Address format — [docs/WALLET_ARCHITECTURE.md](docs/WALLET_ARCHITECTURE.md).
- Deployment/runtime configuration — [docs/OPERATIONS.md](docs/OPERATIONS.md), [docs/MAINNET.md](docs/MAINNET.md).
- User-visible feature behavior — the relevant `*_STATUS.md` document for that component.
- Consensus rules — [docs/CONSENSUS.md](docs/CONSENSUS.md), [docs/CONSENSUS_IMPLEMENTATION.md](docs/CONSENSUS_IMPLEMENTATION.md).

## Style Rules

**Rust**: format with `rustfmt` ([.rustfmt.toml](.rustfmt.toml)); zero-warning `clippy`; `#![deny(unsafe_code)]` is enforced workspace-wide — do not introduce `unsafe` without a documented, reviewed justification (see [docs/CODING_STANDARDS.md](docs/CODING_STANDARDS.md)).

**Python**: type hints on new code; pass `ruff check .`; pass `mypy .`; add `pytest` coverage for behavior changes.

**TypeScript**: strict typing as configured by each package's `tsconfig.json`; avoid `any`; prefer small, reusable components/hooks over one-off inline logic.

**Flutter**: null safety throughout (Dart SDK `>=3.0.0`); never log secrets, seed phrases, or private keys; prefer reusable widgets/services over duplicated UI or service logic (see `apps/mobile-wallet/lib/services/`).

## Contributor Checklist

- [ ] Read the relevant docs for the component you changed.
- [ ] No secrets, seed phrases, private keys, or credentials in the diff.
- [ ] Code formatted/linted per the Style Rules above.
- [ ] Tests added/updated and passing locally.
- [ ] No unresolved critical warnings.
- [ ] Documentation updated if behavior, API, RPC, or format changed.
- [ ] PR description explains what changed and why, with a linked issue if applicable.
- [ ] Consensus-critical, cryptographic, staking, genesis, wallet-signing, perps/risk, or mainnet-config changes flagged explicitly for extra review.

## License Agreement

By contributing to SPRX, you agree that your contributions are provided under the terms of this repository's [LICENSE](LICENSE).
