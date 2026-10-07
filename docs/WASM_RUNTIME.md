# Actual WASM execution profile

The chain executor now supports `StoreCode`, `InstantiateContract`, and `ContractCall`
through CosmWasm VM 3.0.9. The older `WasmContractEngine` simulation is retained for
legacy unit tests; it is not the runtime used by these chain transactions.

Code IDs are SHA-256 hashes of the uploaded bytes. Addresses derive from the chain ID,
sender, transaction nonce, message index, and code ID. Contract storage and metadata
participate in the chain state root and the block's atomic storage commit. Failed calls
discard writes and nonce/balance changes. Queries use read-only storage and consume gas.

## Build and verify the fixture

```sh
rustup target add wasm32-unknown-unknown
bash scripts/build_wasm_fixture.sh
export SPRX_TEST_CONTRACT_WASM="$PWD/contracts/examples/counter/target/wasm32-unknown-unknown/release/sprax_counter.wasm"
cargo test -p sprax-core --test wasm_execution_tests --locked
```

Install Binaryen 123 (providing `wasm-opt`) before running the build script. It lowers
LLVM memory.copy/fill instructions to the pinned VM profile.

The root Cargo WASM linker configuration retains undefined CosmWasm host imports.
The VM supplies these imports when executing a module. The integration test executes
the compiled counter on two ledgers, compares state roots, exercises failed calls and
gas exhaustion, closes the producer database, and queries again after opening it.

## Use a running node

```sh
cargo run -p sprax-cli -- contract --home .sprx store-code --from alice --wasm contracts/examples/counter/target/wasm32-unknown-unknown/release/sprax_counter.wasm
cargo run -p sprax-cli -- contract --home .sprx instantiate --from alice --code-id CODE_ID --label counter --msg '{"count":7}'
cargo run -p sprax-cli -- contract --home .sprx execute --from alice --contract ADDRESS --msg '{"increment":{}}'
cargo run -p sprax-cli -- contract query --contract ADDRESS --msg '{"count":{}}'
```

Transactions are submitted to the mempool. Wait for a finalized receipt from
`sprax_getTransaction` before using a code ID or address. Its `receipt.returnData`
field is hex-encoded JSON for code-upload and instantiation results. Amount options
use integer atto-SPRX. Set `--rpc-url` to the node's JSON-RPC endpoint when needed.

`sprax_queryContract` takes `[address, queryObject, optionalGasLimit]` and returns
`{data: byteArray, gasUsed: number}`. The endpoint caps query gas at 200,000 and runs
at most two concurrent contract queries.

## Remaining work

Storage host functions now cap keys at 1 KiB and values at 64 KiB. Scans collect at most
1,024 records and 1 MiB including scoped keys; oversized ranges fail before copying the
excess record. Range selection is scoped to the contract in all storage backends and
overlays. Each instance permits 16 active iterators, with exhausted handles released.
Contracts must narrow their scan range when a range exceeds the configured limits.

This execution profile rejects chain queries, contract submessages, and nonzero
attached contract funds until bank submessages can release them. Replies,
migrations, sudo, IBC, event indexing, compilation caching, paginated storage
iteration, and broader execution-equivalence/performance testing remain unfinished.
The VM implementation does not establish overall mainnet readiness; see
[MAINNET_READINESS.md](MAINNET_READINESS.md).
