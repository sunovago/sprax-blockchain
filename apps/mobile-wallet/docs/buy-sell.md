# Buy / Sell integration

The wallet exposes two modes: **Cash / Fiat** and **Crypto / Spot**, each with Buy
and Sell. Home, Trade and asset details open the flow. Available pairs, payment
methods, chain IDs and amount limits come from the backend; there are no invented
balances, executable sample quotes or simulated success messages.

## Current operational status

The UI, quote API, validation and hosted checkout handoff are implemented. A real
fiat payment provider and a Spot exchange/DEX adapter have **not** been selected
or configured. Real INR/UPI/card payments and Spot executions are therefore not
active. This implementation does not include an exchange matching engine or an
on-chain liquidity pool. An external hosted provider must collect payment/KYC
where needed, or wallet approval/signatures for a supported Spot trade.

The prior Swap screen only displayed a success snackbar without execution. It
has been replaced by this provider-backed Buy/Sell flow. Futures is unchanged.

## Configuration

Mobile: build with `--dart-define=SPRAX_API_BASE_URL=http://<PC-IP>:8000/api/v1`
for same-Wi-Fi testing. This is the FastAPI service on port 8000, not the chain
RPC on port 26657. For production, use an HTTPS backend URL.

Backend environment:

- `TRADE_GATEWAY_URL`: HTTPS URL of a deployed provider adapter.
- `TRADE_GATEWAY_TOKEN`: server-side bearer credential for that adapter.
- `TRADE_CHECKOUT_HOSTS`: comma-separated exact hosted-checkout hostnames.
- `SECRET_KEY`: non-default secret used to sign expiring quote tokens.

These gateway endpoints are an integration contract defined by this project,
not endpoints claimed to exist on a particular vendor. A selected provider
requires an adapter implementing the following contract. Never put its API
secret or a wallet seed in Flutter configuration.

## Gateway contract

All requests have server-side Bearer authentication. JSON responses are raw
objects, without the backend's `success/data` envelope.

`GET /capabilities` returns `provider` and `options`:

```json
{
  "provider": "Configured provider name",
  "options": [{
    "id": "sprx-inr-upi-buy",
    "label": "SPRX / INR - UPI",
    "mode": "fiat",
    "side": "buy",
    "asset": "SPRX",
    "payment_asset": "INR",
    "payment_method": "UPI",
    "chain_id": "sprax-devnet-1",
    "min_amount": "100",
    "max_amount": "100000"
  }]
}
```

The example is not a statement that SPRX, UPI or this chain is supported by any
provider. Advertise only real supported combinations. `mode` is `fiat` or `spot`;
`side` is `buy` or `sell`. For Spot, `payment_asset` is the quote crypto asset.
For Buy, input amount is in `payment_asset`; for Sell, input amount is in `asset`.
The limits apply to that input asset. Options must have unique IDs.

`POST /quotes` receives `option_id`, decimal-string `amount`, `wallet_address`
and `chain_id`. Return `quote_id`, `provider`, `pay_amount`, `pay_asset`,
`receive_amount`, `receive_asset`, `fee_amount`, `fee_asset`, `expires_at`.
Amounts are decimal strings. `pay_amount` is the total including fees;
`receive_amount` is the net received. Expiry must be timezone-qualified and no
more than 15 minutes away. Quotes must be bound to their original request.

`POST /sessions` receives the original request plus `quote_id`, with a stable
`Idempotency-Key`. Validate the quote and expiry again; return the same session
for repeated use of that key. Return `checkout_url` on an allowlisted HTTPS
host. Do not execute any trade at this step. The user approves in the hosted
provider UI, which must authenticate ownership of assets independently.

The wallet does not label a session as a filled order or credit a balance after
opening checkout. Settlement/history/webhook integration must be implemented
with the chosen provider before adding in-app completed-order status. Current
on-chain wallet balances remain sourced from WalletService.

## Checks

- Flutter: `flutter test test/buy_sell_test.dart`
- Backend: `python -m pytest tests/test_trade.py -q`

Backend tests cover both modes/sides, exact decimal forwarding, unsupported
networks, mismatched quotes, signed-token validation and checkout allowlisting.
No real payment or order is placed by these checks.
