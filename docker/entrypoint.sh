#!/bin/bash
set -e

HOME_DIR="${SPRAX_HOME:-/root/.sprx}"
CHAIN_ID="${SPRAX_CHAIN_ID:-sprax-devnet-1}"
P2P_PORT="${SPRAX_P2P_PORT:-26656}"
RPC_PORT="${SPRAX_RPC_PORT:-26657}"
BOOTSTRAP_PEERS="${SPRAX_PEERS:-}"

# Initialize node state if not already initialized
if [ ! -f "$HOME_DIR/config.toml" ]; then
    echo "Initializing SPRX Node ($CHAIN_ID) at $HOME_DIR..."
    sprax init --chain-id "$CHAIN_ID" --home "$HOME_DIR"
fi

START_ARGS="--home $HOME_DIR --p2p-port $P2P_PORT --rpc-port $RPC_PORT"
if [ -n "$BOOTSTRAP_PEERS" ]; then
    START_ARGS="$START_ARGS --peers $BOOTSTRAP_PEERS"
fi

echo "Launching SPRX Local Node Daemon..."
exec sprax start $START_ARGS
