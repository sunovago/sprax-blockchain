# SPRX Protocol — Native Tokenomics & Global Display

## Native Cryptocurrency Parameters
- **Symbol**: `SPRX`
- **Asset Name**: Scalable Protocol for Real-world X
- **Sub-Unit Precision**: 18 decimal places ($1 \text{ SPRX} = 10^{18} \text{ atto-SPRX}$).
- **Genesis Total Supply**: 1,000,000,000.00 SPRX (1 Billion SPRX).

---

## Initial Genesis Distribution

| Allocation Category | Percentage | Amount (SPRX) | Amount (atto-SPRX) |
|:---|:---|:---|:---|
| **Community Pool** | 40% | 400,000,000 | $400,000,000 \times 10^{18}$ |
| **Ecosystem & Grants** | 25% | 250,000,000 | $250,000,000 \times 10^{18}$ |
| **Treasury Reserve** | 15% | 150,000,000 | $150,000,000 \times 10^{18}$ |
| **Validator Incentives** | 10% | 100,000,000 | $100,000,000 \times 10^{18}$ |
| **Core Contributors (Vested)**| 10% | 100,000,000 | $100,000,000 \times 10^{18}$ |
| **Total Supply** | **100%** | **1,000,000,000** | $1,000,000,000 \times 10^{18}$ |

---

## Ongoing Emission & Burn

Beyond the fixed genesis allocation above, SPRX has an ongoing monetary policy implemented directly in consensus-critical state (`ConsensusParams`, `SupplyState` — `crates/sprax-core`), following the same shape as Bitcoin's halving issuance and Ethereum's fee burn:

- **Block reward (mint)**: each block's proposer is minted a reward, starting at **2 SPRX** per block.
- **Halving**: the reward halves every **84,096,000 blocks** (~4 calendar years at the chain's 1500ms block-time target) — the same cadence as Bitcoin's 4-year halving, expressed in this chain's block count.
- **Hard supply cap**: minting stops the instant total circulating supply would exceed **1,500,000,000 SPRX**; no block ever mints past this cap, mirroring Bitcoin's 21M hard limit.
- **Fee burn**: every transaction fee is destroyed rather than paid to anyone (as it always has been) — this is now explicitly tracked as `SupplyState.total_burned`, EIP-1559-style, so the chain's actual circulating supply is a queryable, consensus-verified figure rather than only inferable from account balances.

Both figures live in `SupplyState` (`circulating_supply`, `total_burned`), stored in the same key-value store as account balances, so they participate in the block's state root and are verified by every node exactly like a balance would be.

---

## Global Presentation Layer Display
While the blockchain exclusively computes in `atto-SPRX`, client wallets and explorer UIs present real-time converted currency estimates:
- **USD** ($)
- **INR** (₹)
- **EUR** (€)
- **JPY** (¥)

Conversions are performed client-side using decentralized oracle feeds, ensuring that core chain consensus is decoupled from external fiat fluctuations.
