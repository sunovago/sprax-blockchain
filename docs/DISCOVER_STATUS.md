# SPRX Mobile Wallet — Discover Status

## Status: FULLY INTEGRATED & CONNECTED

### 1. Overview
The Discover system is a first-class product hub within the SPRX Mobile Wallet that allows users to explore the Sprax ecosystem, trending crypto assets, top market movers (gainers, losers, volume leaders), validator infrastructure, ecosystem dApps, network on-chain metrics, and educational guides.

### 2. Capabilities & Sub-systems
- **Trending Assets Carousel**: Real-time snapshot of hottest assets across SPRX and major crypto ecosystems.
- **Top Movers Tabs**: Live filtered lists for Top Gainers (+%), Top Losers (-%), and 24h Volume Leaders.
- **SPRX Ecosystem Showcase**:
  - Native SPRX token details.
  - Ecosystem tokens (e.g., sUSD, SpraxSwap, SpraxPerp, SpraxLend).
  - Ecosystem dApps with category badges, descriptions, and deep-link / web launch triggers.
  - Validator Set snapshot and active staking APY metrics.
  - Network Health Metrics: Active TPS, Avg Block Time, Staking Ratio, Total Value Bonded.
- **Category Filtering**: Instant filter tags (All, SPRX Ecosystem, Trending, DeFi, Infrastructure, Staking).
- **Educational Guides**: Interactive beginner & security tutorials (Non-custodial security, Staking mechanics, Sprax Chain architecture).

### 3. Data Architecture
- **Primary Source**: `DiscoverService` connected to Sprax Chain RPC + Market Data feeds.
- **Category Taxonomy**: Verified metadata registry with on-chain fallback.
- **Personalization**: Local storage preference cache for recently viewed and favorited sectors without requiring central server tracking.
