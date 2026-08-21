# SPRX Protocol: Engineering & Coding Standards
**Document Version:** 1.0.0  
**Enforcement:** Automated via CI & Workspace Compiler Lints

---

## 1. Core Principles

1. **Safety & Correctness Over Brevity**: Code must be explicit, strongly typed, and structurally immune to panics in production execution paths.
2. **Deterministic State Transitions**: Never introduce floating-point arithmetic or non-deterministic operations within core execution contexts.
3. **Zero Secrets in Source**: Hardware Security Modules (HSMs) or isolated keyrings must manage private keys. Zero test keys in production binaries.
4. **Zero Unjustified Unsafe**: `#![deny(unsafe_code)]` is universally enforced across all workspace crates.

---

## 2. Formatting & Style Rules

All code must format cleanly with standard `rustfmt`:

- **Line Width**: Maximum 100 characters (`max_width = 100`).
- **Indentation**: 4 spaces (tabs forbidden).
- **Import Ordering**: Grouped by:
  1. Standard library (`std::*`).
  2. External dependencies (`serde::*`, `ed25519_dalek::*`).
  3. Internal workspace crates (`sprax_types::*`, `sprax_crypto::*`).
  4. Local module imports (`use crate::*`, `use super::*`).

---

## 3. Error Handling Architecture

```
+---------------------------------------------------------------------------------------------------+
|                                  ERROR HANDLING CONVENTIONS                                       |
+---------------------------------------------------------------------------------------------------+
| Crate Profile       | Error Crate / Pattern             | Rule                                    |
+---------------------+-----------------------------------+-----------------------------------------+
| Library Crates      | `thiserror::Error`                | Typed enums with descriptive display    |
| (sprax-types, etc.) |                                   | messages. Zero `unwrap()` or `expect()` |
+---------------------+-----------------------------------+-----------------------------------------+
| Binaries / Daemons  | `anyhow::Result<T>`               | Top-level context propagation for CLI   |
| (sprax-cli, node)   |                                   | and user-facing error reporting.        |
+---------------------+-----------------------------------+-----------------------------------------+
```

### Prohibited Patterns in Libraries:
- **NO `panic!()`** in runtime code.
- **NO `.unwrap()`** in non-test library code. Use `.expect("invariant description")` only where a mathematical invariant is provably impossible to violate.
- **NO bare integer math on token amounts**: Always use `Amount::checked_add` or `Amount::checked_sub`.

---

## 4. Cryptographic & Memory Hygiene

1. **Zeroize on Drop**: All structures containing private keys, mnemonics, or master seeds must derive or implement `zeroize::ZeroizeOnDrop` or call `.zeroize()` upon drop.
2. **Constant-Time Operations**: Cryptographic signature checks and key comparisons must use audited constant-time routines (`ed25519-dalek`, `k256`, `subtle`).
3. **Approved Primitives Only**: Do not add raw cryptographic hash functions or encryption algorithms from unvetted crates.

---

## 5. Testing Requirements

1. **Unit Tests**: Every public module must contain a dedicated `#[cfg(test)]` submodule testing:
   - Happy paths.
   - Edge cases (zero amounts, empty inputs, overflow limits).
   - Malformed data rejection.
2. **Integration Tests**: Multi-crate interactions belong in the root `tests/` directory.
3. **Determinism Verification**: Cryptographic and hashing tests must explicitly assert deterministic identical outputs across repeated executions.

---

## 6. Git & Commit Guidelines

Commit messages must adhere to the **Conventional Commits** specification:

```
feat(crypto): add bip-44 key derivation path support
fix(consensus): correct proposer priority decrement in DWRR
docs(architecture): update Mermaid diagram for indexer pipeline
test(storage): add atomic batch rollback unit tests
ci(github): add secret scanning stage with trufflehog
```
