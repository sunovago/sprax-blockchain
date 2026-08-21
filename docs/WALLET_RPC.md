# SPRX Protocol: Wallet JSON-RPC Specification
**Document Version:** 1.0.0  
**Target:** Client Platforms (Web & Flutter Mobile)

---

## 1. Supported JSON-RPC Methods

| Method | Parameters | Return Payload | Description |
| :--- | :--- | :--- | :--- |
| `sprax_getAccount` | `[address: string]` | `{balance: string, nonce: number}` | Queries on-chain balance and current sequence nonce |
| `sprax_broadcastTx` | `[signedTxJson: object]` | `{txHash: string}` | Submits an offline-signed transaction to the node mempool |
| `sprax_getValidators`| `[]` | `Array<{address, name, votingPower, commissionRate, status}>` | Fetches active validators and staking parameters |
| `sprax_getStatus` | `[]` | `{chainId, latestBlockHeight, syncStatus}` | Queries node health and synchronization height |

---

## 2. Canonical Transaction Submission Payload
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "sprax_broadcastTx",
  "params": [
    {
      "body": {
        "chain_id": "sprax-testnet-1",
        "sender": "sprax1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq40",
        "nonce": 0,
        "messages": [
          {
            "type": "transfer",
            "to": "sprax1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq30",
            "amount": "1000000000000000000"
          }
        ],
        "fee": {
          "amount": "1000000000000000",
          "gas_limit": 100000
        },
        "memo": "SPRX Mobile Transfer",
        "timeout_height": 1000000
      },
      "key_type": "Ed25519",
      "public_key": "3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
      "signature": "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b"
    }
  ]
}
```
