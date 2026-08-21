# SPRX Mobile Wallet — Perpetual Futures (Perps) Status

## Status: UI COMPLETE · TESTNET / DEMO MODE CONNECTED · PRODUCTION BLOCKED

### 1. Safety & Production Compliance Notice
> **IMPORTANT NOTICE**: Production real-money Perpetual Futures trading is strictly **BLOCKED** and safely isolated until Sprax Chain mainnet perpetual settlement smart contracts, oracle feeds, margin vaults, and liquidation keepers have completed independent formal security audits and legal sign-off.
>
> All Perpetual Futures trading within the mobile wallet executes against the **SPRX Testnet / Demo Engine**, which simulates real-world order matching, leverage margin risk, funding rate ticks, and liquidation engines without risking user capital.

### 2. Implemented Features
- **Mobile-First Trading Terminal**:
  - Compact pair selector (`SPRX/USDT`, `BTC/USDT`, `ETH/USDT`, `SOL/USDT`).
  - Real-time Mark Price, Index Price, 24h High/Low, 24h Volume, and 8h Funding Rate countdown.
  - Interactive Candlestick / Line Chart with timeframe selector (`1m`, `5m`, `15m`, `1h`, `4h`, `1D`).
  - Live animated Order Book with visual depth visualization and Spread calculation.
  - Recent Trades streaming tape with size and direction.
- **Order Management & Leverage Control**:
  - Long (Buy) / Short (Sell) execution.
  - Order Types: Market, Limit, Stop Limit.
  - Leverage Slider (1x to 50x) with dynamic risk level indicator and warnings.
  - Margin calculation, estimated liquidation price preview, slippage tolerance, and estimated trading fees.
  - Optional Take Profit (TP) and Stop Loss (SL) triggers.
  - Pre-execution Trade Confirmation Review Sheet.
- **Position Tracking & Management**:
  - Live positions summary: Pair, Side, Size, Entry Price, Mark Price, Margin, Leverage, Unrealized PnL ($ and %), Liquidation Price.
  - Close Position, Add Margin, and Adjust TP/SL actions.
  - Open Orders list with cancellation support.
  - Trade and Order execution history.
- **Risk Disclosures**:
  - First-time Perps risk acknowledgment sheet detailing liquidation risks, funding rate volatility, and leverage amplifiers.
