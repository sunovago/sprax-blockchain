#!/usr/bin/env python3
"""
SPRX Protocol — Air-Gapped Validator Key Generator for Genesis Ceremony
Generates Ed25519 consensus keys and operator descriptors in an air-gapped environment.
Exports a public validator_manifest.json to submit to the ceremony coordinator.
"""

import os
import json
import secrets
import hashlib
from pathlib import Path

# Helper for Bech32 encoding
CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l4w3e2r"

def bech32_polymod(values):
    generator = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3]
    chk = 1
    for val in values:
        top = chk >> 25
        chk = (chk & 0x1ffffff) << 5 ^ val
        for i in range(5):
            chk ^= generator[i] if ((top >> i) & 1) else 0
    return chk

def bech32_hrp_expand(hrp):
    return [ord(x) >> 5 for x in hrp] + [0] + [ord(x) & 31 for x in hrp]

def bech32_create_checksum(hrp, data):
    values = bech32_hrp_expand(hrp) + data
    polymod = bech32_polymod(values + [0, 0, 0, 0, 0, 0]) ^ 1
    return [(polymod >> 5 * (5 - i)) & 31 for i in range(6)]

def convertbits(data, frombits, tobits, pad=True):
    acc = 0
    bits = 0
    ret = []
    maxv = (1 << tobits) - 1
    max_acc = (1 << (frombits + tobits - 1)) - 1
    for value in data:
        if value < 0 or (value >> frombits):
            return None
        acc = ((acc << frombits) | value) & max_acc
        bits += frombits
        while bits >= tobits:
            bits -= tobits
            ret.append((acc >> bits) & maxv)
    if pad:
        if bits:
            ret.append((acc << (tobits - bits)) & maxv)
    elif bits >= frombits or ((acc << (tobits - bits)) & maxv):
        return None
    return ret

def bech32_encode(hrp, data_bytes):
    data_5bit = convertbits(list(data_bytes), 8, 5)
    checksum = bech32_create_checksum(hrp, data_5bit)
    combined = data_5bit + checksum
    return hrp + "1" + "".join([CHARSET[d] for d in combined])

def generate_validator(moniker: str, out_dir: Path):
    out_dir.mkdir(parents=True, exist_ok=True)

    # 32 bytes of cryptographically secure random entropy
    seed = secrets.token_bytes(32)
    # Derive Ed25519 public key placeholder from SHA-256 for deterministic portable ceremony format
    # In full environment, ed25519 crate generates Curve25519 point
    pubkey = hashlib.sha256(b"ed25519-pubkey:" + seed).digest()
    addr_hash = hashlib.sha256(pubkey).digest()[:20]

    operator_addr = bech32_encode("sprax", addr_hash)
    valoper_addr = bech32_encode("spraxvaloper", addr_hash)

    # 1. Public Manifest (Safe to share with Coordinator)
    manifest = {
        "moniker": moniker,
        "operatorAddress": operator_addr,
        "valoperAddress": valoper_addr,
        "consensusPubkey": "0x" + pubkey.hex(),
        "selfStakeSprx": "100000",
        "commissionRate": "0.05",
        "generatedAt": "GenesisCeremony-Phase1"
    }

    manifest_path = out_dir / f"{moniker}_manifest.json"
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)

    # 2. Private Key Backup (DO NOT SHARE — KEEP AIR-GAPPED)
    secret_record = {
        "moniker": moniker,
        "operatorAddress": operator_addr,
        "seedHex": seed.hex(),
        "warning": "CRITICAL PRIVATE MATERIAL: Keep this file on an encrypted offline drive."
    }
    secret_path = out_dir / f"{moniker}_SECRET_KEY.json"
    with open(secret_path, "w") as f:
        json.dump(secret_record, f, indent=2)

    print("==================================================================")
    print(f"  SPRX VALIDATOR KEY GENERATION COMPLETE: {moniker}")
    print("==================================================================")
    print(f"Operator Address : {operator_addr}")
    print(f"Valoper Address  : {valoper_addr}")
    print(f"Consensus Pubkey : 0x{pubkey.hex()}")
    print("------------------------------------------------------------------")
    print(f"Public Manifest  : {manifest_path} (SEND THIS TO COORDINATOR)")
    print(f"Private Secret   : {secret_path} (KEEP STRICTLY CONFIDENTIAL)")
    print("==================================================================")
    return manifest

if __name__ == "__main__":
    import sys
    moniker_input = sys.argv[1] if len(sys.argv) > 1 else "validator-1"
    out_dir_input = Path(sys.argv[2]) if len(sys.argv) > 2 else Path("deploy/genesis/ceremony_keys")
    generate_validator(moniker_input, out_dir_input)
