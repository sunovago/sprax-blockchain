# SPRX Flutter API Contract

**Version:** 1.0.0  
**Base URL:** `http://localhost:8000` (local) / `https://api.sprax.network` (production)  
**Auth:** Bearer JWT token in `Authorization` header  
**Response Format:** `{success, data, error, request_id}`

---

## Authentication

### POST /api/v1/auth/challenge
**Auth:** None  
**Request:**
```json
{"address": "sprax1..."}
```
**Response:**
```json
{"success": true, "data": {"address": "sprax1...", "nonce": "uuid", "expires_in": 300}}
```

### POST /api/v1/auth/verify
**Auth:** None  
**Request:**
```json
{"address": "sprax1...", "nonce": "uuid", "signature": "hex", "public_key": "hex"}
```
**Response:**
```json
{"success": true, "data": {"access_token": "...", "refresh_token": "...", "token_type": "bearer", "address": "sprax1..."}}
```

### POST /api/v1/auth/refresh
**Request:** `{"refresh_token": "..."}`  
**Response:** `{"access_token": "...", "token_type": "bearer"}`

### POST /api/v1/auth/logout
**Auth:** Required

---

## Public Config

### GET /api/v1/config/public
**Auth:** None  
**Response:**
```json
{
  "network": {
    "name": "testnet",
    "chain_id": "sprax-testnet-1",
    "rpc_url": "http://...:26657",
    "explorer_url": "https://explorer.sprax.network",
    "address_prefix": "sprax1",
    "decimals": 18,
    "default_fee_atto": "1000000000000000",
    "default_gas": 200000
  },
  "feature_flags": {
    "perps_enabled": false,
    "staking_enabled": true,
    "markets_enabled": true,
    "discover_enabled": true,
    "price_alerts_enabled": true
  },
  "supported_currencies": ["USD", "INR", "EUR", "GBP", "JPY"],
  "app_min_version": "1.0.0"
}
```

---

## Blockchain

### GET /api/v1/blockchain/status
**Auth:** None  
**Response:** `{chain_id, latestBlockHeight, syncStatus}`

### GET /api/v1/blockchain/accounts/{address}
**Auth:** None  
**Response:** `{address, balance (atto-SPRX), balance_sprx, nonce}`

### GET /api/v1/blockchain/accounts/{address}/balance
**Auth:** None  
**Response:** `{address, balance_atto, balance_sprx}`

### POST /api/v1/blockchain/transactions/broadcast
**Auth:** None  
**Request:**
```json
{
  "body": {"chain_id": "sprax-testnet-1", "sender": "sprax1...", "nonce": 0, "messages": [...], "fee": {...}},
  "key_type": "Ed25519",
  "public_key": "hex",
  "signature": "hex"
}
```
**Response:** `{txHash: "0x..."}`  
**IMPORTANT:** Backend NEVER receives or stores private keys. Sign offline in wallet.

---

## Markets

### GET /api/v1/markets
**Auth:** None  
**Response:** List of market assets with price, volume, market cap, sparkline

### GET /api/v1/markets/{symbol}
**Auth:** None  
**Response:** Full ticker data for a symbol

### GET /api/v1/markets/{symbol}/candles?interval=1h&limit=100
**Auth:** None  
**Intervals:** 1m, 5m, 15m, 1h, 4h, 1d  
**Response:** `[{timestamp, open, high, low, close}]`

### GET /api/v1/markets/{symbol}/value?currency=INR&amount=100
**Auth:** None  
**Response:** `{symbol, amount, price_usd, currency, value}`

### GET /api/v1/markets/gainers
### GET /api/v1/markets/losers
### GET /api/v1/markets/trending
### GET /api/v1/markets/volume-leaders

---

## FX Rates

### GET /api/v1/fx/rates
**Auth:** None  
**Response:** `{base: "USD", rates: {USD: 1.0, INR: 83.5, EUR: 0.92, GBP: 0.79, JPY: 155.0}}`

---

## Portfolio

### GET /api/v1/portfolio/{address}?currency=INR
**Auth:** None (public address data)  
**Response:**
```json
{
  "address": "sprax1...",
  "currency": "INR",
  "available": {
    "symbol": "SPRX",
    "balance_atto": "1000000000000000000",
    "balance_sprx": "1.0",
    "value_usd": 1.25,
    "value_fiat": 104.375
  },
  "staked": {"symbol": "SPRX", "balance_atto": "0", "balance_sprx": "0"},
  "delegations": [],
  "nonce": 5,
  "sprx_price_usd": 1.25,
  "fx_rates": {"USD": 1.0, "INR": 83.5}
}
```

---

## Search

### GET /api/v1/search?q={query}
**Auth:** None  
**Detection:**
- Integer → block height
- 64-char hex / 0x+64hex → tx hash or block hash
- `sprax1...` → address
- `spraxvaloper1...` → validator
- Token symbol → market asset

**Response:** `{query, results: [{type, data}]}`

---

## Discover

### GET /api/v1/discover
**Auth:** None  
**Response:** trending, gainers, losers, volume_leaders, sprax_ecosystem, validators, network_highlights

---

## Validators

### GET /api/v1/validators
**Auth:** None  
**Response:** List sorted by voting power desc

### GET /api/v1/validators/{operator_address}
**Auth:** None  
**Response:** Validator detail + delegations list

---

## Staking

### GET /api/v1/staking/{address}
**Auth:** None  
**Response:** staked_atto, staked_sprx, rewards, delegations

### GET /api/v1/staking/delegations/{address}
### GET /api/v1/staking/rewards/{address}
### GET /api/v1/staking/validators

---

## Watchlists (Auth Required)

### GET /api/v1/watchlists
### POST /api/v1/watchlists
### POST /api/v1/watchlists/assets — Body: `{"symbol": "BTC"}`
### DELETE /api/v1/watchlists/assets/{symbol}

---

## Notifications (Auth Required)

### GET /api/v1/notifications?limit=20&offset=0
### PATCH /api/v1/notifications/{id}/read
### POST /api/v1/notifications/alerts/price — Body: `{symbol, condition, target_price_usd}`
### GET /api/v1/notifications/alerts/price
### DELETE /api/v1/notifications/alerts/price/{id}

---

## Perps

> **ALL PRODUCTION PERPS ARE DISABLED.** Only testnet/demo mode.

### GET /api/v1/perps/markets
### GET /api/v1/perps/markets/{symbol}
### GET /api/v1/perps/markets/{symbol}/ticker
### GET /api/v1/perps/markets/{symbol}/funding
### GET /api/v1/perps/markets/{symbol}/trades

### Private (Auth + perps_enabled flag):
### GET /api/v1/perps/positions
### POST /api/v1/perps/orders

---

## WebSocket Events

**Base URL:** `ws://localhost:8000`

| Channel | URL | Auth Required |
|---------|-----|---------------|
| markets | /ws/markets | No |
| blockchain | /ws/blockchain | No |
| explorer | /ws/explorer | No |
| perps | /ws/perps | No (public), Yes (positions) |
| notifications | /ws/notifications | Yes |

**Connect with auth:** `?token=<access_token>`

**Subscribe message:**
```json
{"action": "subscribe", "channel": "markets"}
```

**Event format:**
```json
{"type": "ticker.update", "version": 1, "timestamp": "ISO8601", "data": {}}
```

**Event types:**
- `ticker.update` — Market price update
- `block.created` — New block indexed
- `transaction.confirmed` — Transaction finalized
- `network.status` — Network health update
- `notification.created` — User notification
- `heartbeat` — Keepalive ping

---

## Error Format

```json
{
  "success": false,
  "data": null,
  "error": {"code": "INVALID_ADDRESS", "message": "Invalid Sprax address format"},
  "request_id": "uuid"
}
```

**Error Codes:** INVALID_REQUEST, UNAUTHORIZED, FORBIDDEN, NOT_FOUND, RATE_LIMITED, RPC_UNAVAILABLE, INDEXER_BEHIND, INVALID_ADDRESS, INVALID_TRANSACTION, MARKET_NOT_FOUND, PRICE_UNAVAILABLE, PERPS_DISABLED, INVALID_LEVERAGE, INSUFFICIENT_MARGIN, ORDER_REJECTED, STALE_PRICE, SERVICE_UNAVAILABLE

---

## Pagination

```json
{
  "success": true,
  "data": [...],
  "total": 100,
  "limit": 20,
  "offset": 0,
  "has_more": true,
  "request_id": "uuid"
}
```

Query params: `?limit=20&offset=0` (max limit: 100)
