# SPRX WebSocket Contract

**Version:** 1.0.0  
**Protocol:** WebSocket (RFC 6455)  
**Base URL:** `ws://localhost:8000` (local) / `wss://api.sprax.network` (production)

---

## Channel URLs

| Channel | URL | Auth |
|---------|-----|------|
| Markets | `/ws/markets` | Public |
| Blockchain | `/ws/blockchain` | Public |
| Explorer | `/ws/explorer` | Public |
| Perps | `/ws/perps` | Public read / Auth for positions |
| Notifications | `/ws/notifications` | Auth Required |

**Auth:** Append `?token=<access_token>` to URL for private channels.

---

## Client Messages

### Subscribe
```json
{"action": "subscribe", "channel": "markets"}
```

### Unsubscribe
```json
{"action": "unsubscribe", "channel": "markets"}
```

### Ping
```json
{"action": "ping"}
```

---

## Server Event Format

All events follow this versioned structure:

```json
{
  "type": "ticker.update",
  "version": 1,
  "timestamp": "2026-08-20T17:00:00.000Z",
  "data": {}
}
```

---

## Event Types

### markets channel

#### ticker.update
```json
{
  "type": "ticker.update",
  "version": 1,
  "timestamp": "ISO8601",
  "data": {
    "symbol": "BTC",
    "price_usd": 67500.00,
    "price_change_pct_24h": 2.45,
    "volume_24h": 28000000000
  }
}
```

### blockchain channel

#### block.created
```json
{
  "type": "block.created",
  "version": 1,
  "timestamp": "ISO8601",
  "data": {
    "height": 50001,
    "hash": "0x...",
    "tx_count": 42,
    "proposer": "spraxvaloper1...",
    "timestamp": "ISO8601"
  }
}
```

#### transaction.confirmed
```json
{
  "type": "transaction.confirmed",
  "version": 1,
  "timestamp": "ISO8601",
  "data": {
    "hash": "0x...",
    "block_height": 50001,
    "status": "success",
    "sender": "sprax1...",
    "recipient": "sprax1...",
    "amount_sprx": "1.0"
  }
}
```

### notifications channel (Auth Required)

#### notification.created
```json
{
  "type": "notification.created",
  "version": 1,
  "timestamp": "ISO8601",
  "data": {
    "id": "uuid",
    "type": "price_alert",
    "title": "SPRX price alert triggered",
    "body": "SPRX is above $2.00. Current: $2.05"
  }
}
```

### System events (all channels)

#### heartbeat
```json
{"type": "heartbeat", "version": 1, "timestamp": "ISO8601", "data": {}}
```
**Interval:** Every 30 seconds. Clients must respond to stay connected.

#### subscribed
```json
{"type": "subscribed", "version": 1, "timestamp": "ISO8601", "data": {"channel": "markets"}}
```

#### error
```json
{"type": "error", "version": 1, "timestamp": "ISO8601", "data": {"message": "Authentication required for private channel"}}
```

---

## Reconnection Policy (Flutter client)

1. Detect disconnect via heartbeat timeout or WebSocket close event
2. Wait 1 second before first reconnect attempt
3. Exponential backoff: 1s, 2s, 4s, 8s, 16s, 30s max
4. Re-authenticate if token expired
5. Re-subscribe to previous channels after reconnect

---

## Limits

- Max connections per IP: 10
- Max subscriptions per connection: 5 channels
- Heartbeat interval: 30 seconds
- Max message size: 64KB
```
