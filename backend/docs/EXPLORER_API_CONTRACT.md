# SPRX Explorer API Contract

**Version:** 1.0.0  
**Base URL:** `http://localhost:8000`  
**Rate Limit:** 60 req/min per IP  
**Response Format:** `{success, data, error, request_id}`

---

## Blocks

### GET /api/v1/explorer/blocks
**Params:** limit (1-100, default 20), offset  
**Response:** Paginated list of blocks, newest first
```json
{
  "data": [
    {"height": 100, "hash": "0x...", "proposer": "spraxvaloper1...", "tx_count": 5, "gas_used": 450000, "timestamp": "ISO8601"}
  ],
  "total": 100, "limit": 20, "offset": 0, "has_more": true
}
```

### GET /api/v1/explorer/blocks/{height_or_hash}
**Accepts:** integer height or 32-byte hex hash  
**Response:** Block detail + list of transaction hashes

---

## Transactions

### GET /api/v1/explorer/transactions
**Params:** limit, offset  
**Response:** Paginated transaction list
```json
{
  "data": [
    {"hash": "0x...", "block_height": 100, "sender": "sprax1...", "recipient": "sprax1...",
     "amount_atto": "1000000000000000000", "amount_sprx": "1", "fee_atto": "1000000000000000",
     "gas_used": 21000, "nonce": 5, "memo": "", "type": "transfer", "status": "success", "timestamp": "ISO8601"}
  ]
}
```

### GET /api/v1/explorer/transactions/{hash}
**Accepts:** 64-char hex or 0x-prefixed hash

---

## Addresses

### GET /api/v1/explorer/addresses/{address}
**Accepts:** Bech32 `sprax1...` address  
**Response:**
```json
{
  "address": "sprax1...",
  "balance_atto": "1000000000000000000",
  "balance_sprx": "1",
  "nonce": 5,
  "tx_count": 42,
  "first_seen_height": 1,
  "last_seen_height": 99
}
```

### GET /api/v1/explorer/addresses/{address}/transactions
**Params:** limit, offset  
**Response:** Paginated transaction history for address (sent + received)

---

## Validators

### GET /api/v1/explorer/validators
**Response:** All validators sorted by voting power descending  
```json
{
  "data": [
    {
      "operator_address": "spraxvaloper1...",
      "moniker": "SPRX Core Foundation",
      "commission_rate": "0.0500",
      "voting_power": 4000,
      "voting_power_share_pct": "40.0000",
      "status": "ACTIVE",
      "jailed": false,
      "uptime_percent": "99.97",
      "missed_blocks": 3
    }
  ]
}
```

### GET /api/v1/explorer/validators/{operator_address}

---

## Network

### GET /api/v1/explorer/network
**Response:**
```json
{
  "latest_chain_height": 50000,
  "latest_indexed_height": 49998,
  "lag_blocks": 2,
  "active_validators": 100,
  "tps": "4850.00",
  "avg_block_time_ms": 480,
  "total_transactions": 1250000
}
```

---

## Search

### GET /api/v1/search?q={query}
**Resolves:** height → block, hash → tx or block, `sprax1` → address, `spraxvaloper1` → validator, symbol → asset  
**Response:** `{query, results: [{type: "block|transaction|address|validator|asset", data: {...}}]}`

---

## Rate Limits

- 60 requests/minute per IP
- Max page limit: 100 items
- HTTP 429 on exhaustion with error code `RATE_LIMITED`

---

## Input Validation

- Address: `sprax1` + 38 alphanumeric chars
- Validator: `spraxvaloper1` + 38 alphanumeric chars  
- Tx hash: 64-char hex or `0x` + 64-char hex
- Block height: positive integer
