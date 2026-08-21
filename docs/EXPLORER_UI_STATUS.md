# SPRX Explorer UI Status

**Document Version:** 1.0.0  
**Target Product:** Sprax Explorer (SPRX — Scalable Protocol for Real-world X)  
**Location:** `apps/explorer-ui`

---

## 1. Executive Summary
Sprax Explorer is the official, high-performance, responsive blockchain explorer for Sprax Chain. It provides real-time visibility into CometBFT blocks, normalized state transactions, accounts and balances, validator rankings and staking metrics, smart contracts, developer tools, and omni-search capabilities.

---

## 2. Screens & Features Matrix

| Screen / Feature | Status | Route / View | Notes |
| :--- | :--- | :--- | :--- |
| **Global Layout & Navigation** | ✅ Completed | All pages | Sticky header, brand wordmark, live status indicator, network switcher, theme toggle, currency switcher, developer dropdown, mobile drawer |
| **Universal Omni-Search** | ✅ Completed | Global (`/` & modal `Ctrl+K`) | Auto-detects Tx Hash (`0x...`), Address (`sprax1...` / `0x...`), Block Height, Block Hash, Validator moniker, with keyboard shortcut and query parsing |
| **Home Dashboard** | ✅ Completed | `/` | Hero status cards (Height, TPS, Finality, Validators, Staked, Fiat Feed), real-time block and transaction feeds, live polling control |
| **Blocks Explorer** | ✅ Completed | `/blocks` | Paginated blocks table, block time, gas used, validator attribution, search and sorting |
| **Block Details** | ✅ Completed | `/block/:id` | Height, hash, parent hash, state root, tx count, gas metrics, prev/next block navigation, transactions tab & raw JSON inspector |
| **Transactions Explorer** | ✅ Completed | `/transactions` | Paginated transactions table, message types (`Transfer`, `Delegate`, `Undelegate`, `ContractCall`), status badges, filtering |
| **Transaction Details** | ✅ Completed | `/tx/:hash` | Visual transfer flow card (`From -> Amount -> To`), full cryptographic receipt, gas metrics, nonce, memo, event logs, raw JSON payload |
| **Address / Account Explorer** | ✅ Completed | `/address/:address` | Native balance, fiat conversion, nonce, transaction count, QR code modal, tabbed transaction chronology (All, In, Out), staking stake |
| **Validators Leaderboard** | ✅ Completed | `/validators` | Ranked validator set, voting power %, bonded stake, commission, uptime score, status filters (Active/Inactive) |
| **Validator Details** | ✅ Completed | `/validator/:id` | Detailed validator identity, operator address, voting power, uptime, proposed blocks, delegators distribution |
| **Staking Dashboard** | ✅ Completed | `/staking` | Staking ratio, total bonded SPRX, active validators, network staking metrics calculator, delegation distribution |
| **Smart Contracts** | ✅ Completed | `/contracts` & `/contract/:address` | Verified contracts, creator, bytecode size, execution count, ABI/read interface, event logs |
| **Network Analytics** | ✅ Completed | `/analytics` | Transaction growth trends, TPS dynamics, block time distribution, validator decentralization metrics |
| **Developer Portal** | ✅ Completed | `/developers` | RPC endpoints, Chain ID, Decimals, REST API playground, SDK snippets (cURL, TS, Rust), Faucet connector |
| **Network Status** | ✅ Completed | `/network` | CometBFT consensus engine health, round state, peer connections, indexer sync latency |
| **Testnet Faucet** | ✅ Completed | `/faucet` & in Dev portal | Direct rate-limited testnet faucet request UI with auto-address fill |
| **Theme System** | ✅ Completed | Global | Dark-first fintech aesthetic, light mode toggle, system preference sync, CSS variables token architecture |
| **Currency Display** | ✅ Completed | Global | On-chain balance + reference market values in USD ($), INR (₹), EUR (€), GBP (£), JPY (¥) |
| **Error Handling & 404** | ✅ Completed | `/404` & ErrorBoundary | Graceful fallbacks, network offline indicators, search not-found states |

---

## 3. API Integration Architecture
- **Centralized REST Client**: Built in `src/services/api.ts` targeting `/api/v1` (`GET /blocks`, `GET /blocks/{id}`, `GET /txs`, `GET /txs/{hash}`, `GET /addresses/{addr}`, `GET /validators`, `GET /stats`, `GET /search?q={query}`).
- **Network Switcher**: Supports Mainnet (`sprax-mainnet-1`), Public Testnet (`sprax-testnet-1`), and Local Devnet (`http://127.0.0.1:8080`).
- **Resilient Fallback Mode**: Gracefully provides sample testnet genesis telemetry when backend node is offline, enabling uninterrupted development, demonstration, and automated test execution.

---

## 4. Test Coverage & Quality Gate
- **Unit Tests**: Formatter utilities, hash truncation, address verification, search classifier, currency calculations.
- **Component Tests**: Search bar, navigation, data tables, status badges, details views.
- **Build Verification**: TypeScript strict mode (`tsc`), Vite build bundling.
