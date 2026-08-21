# SPRX Protocol: Public Testnet Faucet Specification
**Document Version:** 1.0.0  
**Target:** Abuse Prevention, Rate Limiting, Audit Logging

---

## 1. Operational Policies & Limits

> [!WARNING]
> **NO MAINNET FUNDS**: The faucet is hardcoded to the testnet chain ID `sprax-testnet-1`. It cannot hold or distribute mainnet cryptocurrency.

### 1.1 Rate Limiting Model
- **Address Limit**: Maximum **1 disbursement per 24 hours** per Bech32 address.
- **IP Address Limit**: Maximum **1 disbursement per 24 hours** per client IP.
- **Default Payout**: $100\text{ tSPRX}$.
- **Maximum Payout**: $500\text{ tSPRX}$ (for automated test harness scripts).

---

## 2. API Endpoints

### 2.1 Request Funds
`POST /api/v1/faucet/claim`

**Request Payload:**
```json
{
  "recipient": "sprax1sdtpz05x2r7d70u428v2ff3rtyj49z65a380gr",
  "amount_sprx": 100
}
```

**Success Response (HTTP 200):**
```json
{
  "success": true,
  "tx_hash": "0x3a1f4b8c9d2e1f0a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a9b8c7d6e5f4a",
  "recipient": "sprax1sdtpz05x2r7d70u428v2ff3rtyj49z65a380gr",
  "amount": 100000000000000000000,
  "amount_sprx": "100 tSPRX",
  "message": "Tokens successfully disbursed to testnet address",
  "network_notice": "SPRX TESTNET ONLY — TOKENS HAVE NO REAL WORLD VALUE"
}
```

**Rate Limit Exceeded (HTTP 429):**
```json
{
  "error": "Rate limit exceeded for address/IP: retry in 82400 seconds"
}
```

---

## 3. Anti-Bot / CAPTCHA Protection

For web-based UI interactions, the faucet integrates Cloudflare Turnstile / hCaptcha verification before accepting disbursement requests.
