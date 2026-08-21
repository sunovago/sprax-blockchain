# SPRX Protocol: Complete Ecosystem Page Map & Routing Specification
**Document Version:** 1.0.0-PAGEMAP  
**Date:** 2026-08-21  
**Target:** SPRX (Scalable Protocol for Real-world X) Ecosystem

---

## 1. Master Page & Route Inventory

The SPRX Ecosystem Portal is structured as a comprehensive multi-section web application, avoiding single-page landing site limitations. Every route is engineered with dedicated sub-components, layout hierarchies, API bindings, and SEO metadata:

```
+======================================================================================================+
|                                    SPRX ECOSYSTEM SITEMAP & ROUTING                                  |
+======================================================================================================+
| 1. CORE PORTAL           2. LEARN HUB             3. DEVELOPERS HUB         4. NETWORK HUB           |
|  - / (Homepage)           - /learn                 - /developers             - /network              |
|  - /about                 - /learn/what-is-sprx    - /developers/docs        - /network/validators   |
|  - /whitepaper            - /learn/real-world-x    - /developers/rpc         - /network/staking      |
|  - /brand                 - /learn/tokenomics      - /developers/smart-contracts - /network/status   |
|                           - /learn/consensus       - /developers/sdk         - /network/parameters   |
|                           - /learn/wallets         - /developers/nodes       - /network/upgrades     |
|                           - /learn/security        - /developers/standards   - /network/validator/:id|
|                                                    - /developers/faucet                              |
+------------------------------------------------------------------------------------------------------+
| 5. BLOCKCHAIN EXPLORER   6. ECOSYSTEM & DISCOVER  7. GOVERNANCE & COMMUNITY 8. RESEARCH HUB          |
|  - /explorer              - /ecosystem             - /governance             - /research             |
|  - /explorer/blocks       - /discover              - /sips (SIPs Proposal)   - /research/consensus   |
|  - /explorer/block/:id    - /markets               - /sips/:id               - /research/rwa         |
|  - /explorer/transactions - /ecosystem/submit      - /community              - /research/storage     |
|  - /explorer/tx/:hash                              - /community/grants       - /security/bug-bounty  |
|  - /explorer/address/:addr                         - /community/events                               |
|  - /explorer/contracts                                                                               |
|  - /explorer/analytics                                                                               |
+======================================================================================================+
```

---

## 2. Detailed Route Specifications

### 2.1 Core Portal Routes
- **`/` (Homepage)**:
  - *Header*: Logo, Primary Navigation with Mega-Menus, Omni-Search (`⌘K`), Network Selector, Fiat Currency Selector, Theme Toggle, "Get Started" / "Explore SPRX" CTA.
  - *Hero Section*: "Infrastructure for the Real-World Decentralized Economy" + 2-sentence summary + Primary/Secondary CTAs + Interactive Protocol Canvas.
  - *Live Network Strip*: Real-time block height, TPS, 1.5s finality badge, active validator count, and connection health status.
  - *What Makes SPRX Different*: 6 core pillars (Scalable by Architecture, Real-World Ready, Deterministic Execution, Enterprise Interoperability, Dual-Tier Storage, Open Governance).
  - *Built for Real-World X*: 8 category cards (Payments & Settlements, Tokenized Real-World Assets, Digital Identity & Credentials, Global Commerce, Financial Infrastructure, DePIN & IoT Data Rails, Creator Economy, Public Infrastructure).
  - *Interactive Protocol Architecture Visualizer*: 5-tier layer explorer (Applications -> Smart Contracts/WASM -> CometBFT Consensus -> P2P GossipSub -> Storage/Redb).
  - *Ecosystem Snapshot*: Featured dApps, wallets, and tooling.
  - *Developer Quickstart Strip*: 3-step interactive code box (Install SDK -> Connect Node -> Deploy Contract).
  - *Network & Validator Highlights*: Staking APR, validator distribution graph, and node operator CTA.
  - *Latest Protocol Updates & Roadmap*: Milestone progress tracker from Genesis to Mainnet.
  - *Structured Footer*: Multi-column links (Explore, Learn, Developers, Network, Ecosystem, Community, Security, Legal).

- **`/about`**: Mission statement, core principles, contributor culture, and foundational architecture.
- **`/whitepaper`**: Comprehensive technical whitepaper viewer with embedded LaTeX mathematical formulas, downloadable PDF export, and section bookmarks.
- **`/brand`**: Media kit, official SPRX SVG logomarks, typography rules, color palettes, and press guidelines.

---

### 2.2 Learn Hub (`/learn/*`)
- **`/learn`**: Interactive learning pathway selector for beginners, developers, and enterprises.
- **`/learn/what-is-sprx`**: Fundamental primer explaining blockchain technology through the lens of tangible real-world utility.
- **`/learn/real-world-x`**: Deep dive into how SPRX handles high-volume payments, micro-transactions, and asset tokenization without network congestion.
- **`/learn/tokenomics`**: 1 Billion genesis supply breakdown, 18 decimal precision (`atto-SPRX`), EIP-1559 base fee burning mechanism, and staking reward curves.
- **`/learn/consensus`**: Byzantine Fault Tolerance (BFT-PoS), 2/3+ cryptographic voting rounds, Deterministic Weighted Round Robin (DWRR) proposer election, and slashing.
- **`/learn/wallets`**: Self-custody education, BIP-39 mnemonic seed phrase security, hardware wallets, and transaction signing.
- **`/learn/security`**: Threat prevention, phishing awareness, smart contract audits, and incident reporting.

---

### 2.3 Developers Portal (`/developers/*`)
- **`/developers`**: Developer command center featuring quickstarts, SDK libraries, and API playgrounds.
- **`/developers/docs`**: Nested technical documentation with left sidebar tree, center Markdown renderer, right Table of Contents, and multi-language code snippets.
- **`/developers/rpc`**: Complete JSON-RPC 2.0 and REST API interactive reference with live query runner (`sprax_getStatus`, `sprax_getBlock`, `sprax_getTransaction`, `sprax_broadcastTx`, etc.).
- **`/developers/smart-contracts`**: CosmWasm Rust smart contract guides, CW20 token standards, escrow templates, and gas optimization rules.
- **`/developers/sdk`**: TypeScript (`@sprax/wallet-core`), Rust (`sprax-crypto`, `sprax-types`), Python, and Go library guides.
- **`/developers/nodes`**: Bare-metal / Cloud node operator setup guide, CLI commands (`sprax node init`, `sprax node start`), and sentry architecture.
- **`/developers/standards`**: SXS (SPRX eXtensible Standards): SXS-20 Fungible Tokens, SXS-721 RWAs, SXS-1155 Multi-Assets, and Account Abstraction.
- **`/developers/faucet`**: Interactive Testnet faucet dispenser with rate-limiting, wallet address auto-fill, and transaction receipt confirmation.

---

### 2.4 Network & Staking Hub (`/network/*`)
- **`/network`**: Comprehensive network dashboard displaying consensus parameters, genesis telemetry, and live sync status.
- **`/network/validators`**: Filterable leaderboard of active (Top 100) and candidate validators with moniker, voting power %, commission, uptime %, and slashes.
- **`/network/validator/:id`**: Deep validator profile showing operator address (`spraxvaloper1...`), self-bonded stake, delegators list, proposed blocks, and commission history.
- **`/network/staking`**: Staking calculator (input amount -> estimate daily/annual yield), delegation modal, unbonding queue timeline, and slashing risk education.
- **`/network/parameters`**: On-chain consensus and state parameters (Block gas limits, base fee multiplier, voting windows).
- **`/network/status`**: Detailed node telemetry, CometBFT consensus engine round state, and P2P gossip peer table.
- **`/network/upgrades`**: Protocol version upgrade tracker, coordinate heights, and backward compatibility notes.

---

### 2.5 Blockchain Explorer (`/explorer/*`)
- **`/explorer`**: Live block stream, real-time transaction ticker, gas price tracker, and omni-search interface.
- **`/explorer/blocks`**: Paginated blocks table with height, block hash, timestamp, proposer address, tx count, and gas consumed.
- **`/explorer/block/:id`**: Comprehensive block inspector with parent hash, state root, tx root, receipts root, validator commit signatures, and raw JSON view.
- **`/explorer/transactions`**: Paginated transaction stream with message types (`Transfer`, `Delegate`, `ContractCall`), gas fees, and execution status.
- **`/explorer/tx/:hash`**: Visual transfer flow card (`Sender -> Payload -> Recipient`), gas limit vs used, cryptographic receipt, and event logs.
- **`/explorer/address/:addr`**: Account overview, balance in SPRX + selected fiat currency, sequence nonce, QR code, and transaction chronology.
- **`/explorer/contracts`**: Verified WASM smart contracts, creator address, bytecode size, execution count, and contract state inspector.
- **`/explorer/analytics`**: Interactive charts for transaction history, TPS peaks, gas trends, and validator decentralization metrics.

---

### 2.6 Ecosystem & Discover (`/ecosystem/*`, `/discover`, `/markets`)
- **`/ecosystem`**: Categorized directory of all verified projects built on SPRX (DeFi, RWA, Payments, Wallets, Bridges, Developer Tools).
- **`/discover`**: Curated application showcase highlighting trending apps, top utility protocols, and new mainnet deployments.
- **`/markets`**: Real-time market data terminal featuring native SPRX price, trading pairs, 24h volume, market cap, and multi-timeframe candlestick/line charts.
- **`/ecosystem/submit`**: Standardized project listing application form for ecosystem builders.

---

### 2.7 Governance & Community (`/governance/*`, `/sips/*`, `/community/*`)
- **`/governance`**: Active on-chain governance proposals, deposit period proposals, passed proposals, voting participation metrics, and timelock status.
- **`/sips`**: SPRX Improvement Proposals browser with status filters (`Draft`, `Review`, `Accepted`, `Final`, `Rejected`) and category filters (`Core`, `Networking`, `Execution`, `Standards`, `Meta`).
- **`/sips/:id`**: Dedicated SIP proposal detail page displaying motivation, technical specification, rationale, test cases, and discussion links.
- **`/community`**: Hub for developer ambassadors, regional communities, social channels (GitHub, Discord, X, Telegram), and contributor badges.
- **`/community/grants`**: SPRX Ecosystem Foundation Grants program, eligibility criteria, funding tiers, and application guidelines.
- **`/community/events`**: Protocol hackathons, developer workshops, AMA schedules, and community calls.
- **`/security/bug-bounty`**: Vulnerability disclosure terms, reward matrix (Critical/High/Medium/Low), PGP encryption keys, and security advisories.

---

### 2.8 Research Hub (`/research/*`)
- **`/research`**: Academic and technical research index covering consensus, cryptography, data availability, and state scaling.
- **`/research/consensus`**: Mathematical proofs on BFT-PoS liveness, safety under asynchrony, and dynamic proposer rotation.
- **`/research/rwa`**: Frameworks for legally compliant real-world asset tokenization and oracle-mediated settlement.
- **`/research/storage`**: Dual-tier storage architecture, Jellyfish Merkle Trees, and zero-knowledge state proofs.
