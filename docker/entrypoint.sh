#!/bin/bash
set -euo pipefail

NODE_HOME="${SPRAX_HOME:-/root/.sprx}"
CHAIN_ID="${SPRAX_CHAIN_ID:-sprax-devnet-1}"
ENVIRONMENT="${SPRAX_ENV:-development}"
P2P_PORT="${SPRAX_P2P_PORT:-26656}"
RPC_PORT="${SPRAX_RPC_PORT:-26657}"
BOOTSTRAP_PEERS="${SPRAX_PEERS:-}"

if [ ! -f "$NODE_HOME/config.toml" ]; then
    if [[ "$ENVIRONMENT" == "development" && "$CHAIN_ID" != "sprax-devnet-1" ]]; then
        echo "Explicit SPRAX_ENV and agreed SPRAX_GENESIS are required to initialize a non-default chain." >&2
        exit 1
    fi
    init_args=(--chain-id "$CHAIN_ID" --env "$ENVIRONMENT" --home "$NODE_HOME")
    if [[ "$ENVIRONMENT" != "development" ]]; then
        : "${SPRAX_GENESIS:?Set SPRAX_GENESIS to the agreed public genesis JSON outside the node home}"
        init_args+=(--genesis "$SPRAX_GENESIS")
    fi
    sprax init "${init_args[@]}"
fi

start_args=(--home "$NODE_HOME" --p2p-port "$P2P_PORT" --rpc-port "$RPC_PORT")
if [ -n "$BOOTSTRAP_PEERS" ]; then
    start_args+=(--peers "$BOOTSTRAP_PEERS")
fi
exec sprax start "${start_args[@]}"
