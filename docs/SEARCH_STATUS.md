# SPRX Mobile Wallet — Global Search Status

## Status: FULLY INTEGRATED & CONNECTED

### 1. Overview
Global Search is an omnichannel search experience accessible from all major screens (App Bar, Home quick search, Markets, Discover). It instantly detects entity query types and provides grouped results across all blockchain and market categories.

### 2. Auto-Detection Capabilities
The search engine analyzes query patterns in real-time:
- **Sprax Bech32 Address** (`sprax1...`): Directly routes to account detail / send funds / address inspect.
- **Transaction Hash** (`0x...` or 64 hex characters): Routes to Transaction Details screen.
- **Block Height** (pure digits): Routes to Block Explorer query.
- **Trading Pairs** (e.g., `SPRX/USDT`, `BTC/USDT`, `ETH/USDT`): Directly routes to Perps or Markets terminal.
- **Token Symbol / Name** (e.g., `SPRX`, `USDT`, `SOL`, `Bitcoin`): Direct match to Asset Detail or Market view.

### 3. Result Grouping & Filtering
Results are categorized into distinct groups:
1. **Assets & Tokens** (SPRX, sUSD, BTC, ETH, SOL, etc.)
2. **Markets & Trading Pairs** (SPRX/USDT, BTC/USDT Perps, etc.)
3. **Wallet Addresses** (Known contacts, active accounts)
4. **Transactions & Blocks** (Recent txs, block hashes)
5. **Validators & Infrastructure** (Active validator nodes)
6. **Discover & Ecosystem dApps** (DEX, Lending, Bridge, Explorer)

### 4. Performance & UX
- **Debounced Input (250ms)**: Eliminates UI lag and wasteful queries during typing.
- **Recent Searches**: Offline persistent search history with item deletion and full clear.
- **Trending Searches**: Preset suggestions for quick discovery.
