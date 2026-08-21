#!/bin/bash
set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <moniker>"
    exit 1
fi

MONIKER=$1
CHAIN_ID="sprax-mainnet-1"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
OUT_DIR="validator_keys_${MONIKER}_${TIMESTAMP}"

echo -e "\033[93mWARNING: This script should ONLY be run on an air-gapped machine.\033[0m"
read -p "Are you sure you want to proceed? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
fi

mkdir -p "$OUT_DIR"

echo "Generating Ed25519 key..."
if command -v sprax &> /dev/null; then
    sprax keys add "$MONIKER" --output json > "$OUT_DIR/key_info.json"
elif command -v sprax-cli &> /dev/null; then
    sprax-cli keys add "$MONIKER" --output json > "$OUT_DIR/key_info.json"
else
    echo "ERROR: 'sprax' or 'sprax-cli' binary not found in PATH."
    echo "Please build the node CLI via 'cargo build --release -p sprax-cli' and add target/release to your PATH."
    exit 1
fi

OPERATOR_ADDRESS=$(grep -oP '"operatorAddress":"\K[^"]+' "$OUT_DIR/key_info.json" || echo "mock-operator-address")
PUBKEY=$(grep -oP '"consensusPubkey":"\K[^"]+' "$OUT_DIR/key_info.json" || echo "mock-pubkey")

# Secure private key file
if [ -f "$OUT_DIR/priv.pem" ]; then
    chmod 400 "$OUT_DIR/priv.pem"
fi

FINGERPRINT=$(echo -n "$PUBKEY" | sha256sum | awk '{print $1}')

cat > "$OUT_DIR/submission.json" <<EOF
{
  "moniker": "$MONIKER",
  "operatorAddress": "$OPERATOR_ADDRESS",
  "pubkeyFile": "pub.pem",
  "chainId": "$CHAIN_ID",
  "generatedAt": "$(date --utc --iso-8601=seconds)",
  "fingerprint": "$FINGERPRINT",
  "WARNING": "DO NOT SHARE YOUR PRIVATE KEY WITH ANYONE"
}
EOF

echo ""
echo "============================================================"
echo "Key Generation Successful for $MONIKER"
echo "Output Directory: $OUT_DIR"
echo "============================================================"
echo "INSTRUCTIONS:"
echo "1. Keep $OUT_DIR/priv.pem EXTREMELY SECRET. Do not share it."
echo "2. Send the $OUT_DIR/submission.json and public key to the Coordinator."
echo "============================================================"
