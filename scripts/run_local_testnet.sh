#!/bin/bash
# SPRX Local 3-Node Testnet Orchestration Script
set -e

echo "============================================================"
echo "  SPRX PROTOCOL: LOCAL 3-NODE TESTNET INITIALIZATION"
echo "============================================================"

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$DIR"

# Clean up previous testnet data
rm -rf .testnet
mkdir -p .testnet/node1 .testnet/node2 .testnet/node3

echo "[1/4] Compiling SPRX CLI Binary..."
cargo build --release -p sprax-cli
SPRAX_BIN="./target/release/sprax"

echo "[2/4] Initializing Node 1, Node 2, Node 3 Home Directories..."
$SPRAX_BIN init --chain-id sprax-devnet-1 --home .testnet/node1
$SPRAX_BIN init --chain-id sprax-devnet-1 --home .testnet/node2
$SPRAX_BIN init --chain-id sprax-devnet-1 --home .testnet/node3

# Sync genesis file from node1 to node2 and node3 to ensure identical state
cp .testnet/node1/genesis.json .testnet/node2/genesis.json
cp .testnet/node1/genesis.json .testnet/node3/genesis.json

echo "[3/4] Launching 3-Node Testnet in Background..."
$SPRAX_BIN start --home .testnet/node1 --p2p-port 26656 --rpc-port 26657 > .testnet/node1.log 2>&1 &
PID_NODE1=$!

sleep 2

$SPRAX_BIN start --home .testnet/node2 --p2p-port 26666 --rpc-port 26667 --peers 127.0.0.1:26656 > .testnet/node2.log 2>&1 &
PID_NODE2=$!

sleep 2

$SPRAX_BIN start --home .testnet/node3 --p2p-port 26676 --rpc-port 26677 --peers 127.0.0.1:26656,127.0.0.1:26666 > .testnet/node3.log 2>&1 &
PID_NODE3=$!

echo "------------------------------------------------------------"
echo "  Node 1 PID: $PID_NODE1 (P2P: 26656, RPC: 26657)"
echo "  Node 2 PID: $PID_NODE2 (P2P: 26666, RPC: 26667)"
echo "  Node 3 PID: $PID_NODE3 (P2P: 26676, RPC: 26677)"
echo "============================================================"
echo "  To stop the testnet, run: kill $PID_NODE1 $PID_NODE2 $PID_NODE3"
echo "============================================================"
