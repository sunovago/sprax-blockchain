# SPRX Mobile Wallet — Markets Status

## Status: FULLY INTEGRATED & CONNECTED

### 1. Overview
The Markets module provides a professional market data interface supporting Sprax Chain native assets and top global cryptocurrencies. It features live floating price tickers, 24h market metrics, interactive candlestick and line charts across multiple timeframes, persistent user watchlists, and currency conversions (INR, USD, EUR, GBP, JPY).

### 2. Core Capabilities
- **Multi-Tab Market Explorer**: Watchlist, Top Assets, Gainers, Losers, Volume Leaders, and SPRX Ecosystem.
- **Precision Price Handling**: High precision representation for micro-cap assets (e.g. ₹0.000012) up to high-value assets ($98,745.32) without floating point loss.
- **Interactive Multi-Timeframe Charts**:
  - Timeframes: 1H, 1D, 1W, 1M, 1Y.
  - Modes: Line sparklines and OHLCV Candlesticks.
  - Interactive touch pan, crosshair with timestamp & exact price inspection.
- **Asset Detail View**:
  - Live price & 24h change badge.
  - 24h High, 24h Low, 24h Trading Volume, Market Capitalization, Circulating Supply, All-Time High.
  - Action Sheet: Direct Buy/Trade routing, Send/Receive, Add/Remove Watchlist, View on Block Explorer.
- **Local Watchlist**: High-speed offline-first persistence with instant real-time price synchronization.

### 3. Separation of Concerns
- **Market Data Service**: Dedicated HTTP & WebSocket pricing pipelines; strictly separated from blockchain consensus/RPC nodes.
- **FX Conversion**: Real-time currency exchange rates applied on base USD or native pricing.
