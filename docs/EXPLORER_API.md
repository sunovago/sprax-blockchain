# SPRX Protocol: Explorer REST API Reference
**Document Version:** 1.0.0  
**Base Path:** `/api/v1`  
**Response Format:** `application/json`

---

## 1. Overview & Security Limits

The Explorer API provides public access to indexed blockchain data.

### 1.1 Rate Limiting & Query Controls
- **Default Page Limit**: 20 items.
- **Maximum Page Limit**: 100 items (`limit=100`).
- **Rate Limit**: 60 requests/minute per client IP (HTTP 429 upon exhaustion).
- **Input Validation**: Strict regex matching for 32-byte hex hashes (`0x[0-9a-fA-F]{64}`) and Bech32 addresses (`sprax1...`).

---

## 2. API Endpoints

### 2.1 Blocks Endpoints

#### `GET /api/v1/blocks`
Returns paginated list of indexed blocks in reverse chronological order.
- **Query Parameters**:
  - `limit` (integer, optional, default: 20, max: 100)
  - `offset` (integer, optional, default: 0)

#### `GET /api/v1/blocks/{height_or_hash}`
Fetches block details by integer height (e.g. `10`) or 32-byte hex hash.

---

### 2.2 Transactions Endpoints

#### `GET /api/v1/txs`
Returns paginated list of transactions in reverse chronological order.
- **Query Parameters**:
  - `limit` (integer, optional, default: 20, max: 100)
  - `offset` (integer, optional, default: 0)

#### `GET /api/v1/txs/{hash}`
Fetches complete transaction details, sender, recipient, fee, and execution receipt.

---

### 2.3 Addresses Endpoints

#### `GET /api/v1/addresses/{address}`
Fetches account balance (in atomic `atto-SPRX` and formatted SPRX), nonce, and transaction count.

#### `GET /api/v1/addresses/{address}/txs`
Fetches paginated transaction history associated with the address.
- **Query Parameters**:
  - `limit` (integer, optional, default: 20, max: 100)
  - `offset` (integer, optional, default: 0)

---

### 2.4 Validators Endpoints

#### `GET /api/v1/validators`
Returns all active validators ranked descending by voting power.

---

### 2.5 Network Statistics

#### `GET /api/v1/stats`
Returns high-level network health, latest block height, block hash, state root, total transactions, active validator count, and estimated TPS.

---

### 2.6 Universal Search

#### `GET /api/v1/search?q={query}`
Universal search resolver matching integer heights, 32-byte hashes, Bech32/Hex addresses, or validator monikers.
- **Response Structure**:
  ```json
  {
    "type": "Block | Transaction | Address | Validator",
    "data": { ... }
  }
  ```
