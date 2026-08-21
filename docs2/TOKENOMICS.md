# SPRX Protocol: Native Tokenomics & Economic Model Specification
**Document Version:** 1.0.0-PROPOSAL  
**Status:** ARCHITECTURE_ONLY (Phase 01)  
**Native Asset:** SPRX (Scalable Protocol for Real-world X)  
**Classification:** Floating Utility & Staking Layer-1 Cryptocurrency

---

## 1. Native Asset Definition & Decimal Precision

The native cryptographic utility asset of the SPRX Protocol is **SPRX** (Scalable Protocol for Real-world X).

### Key Economic Properties:
1. **Floating Market Valuation**: SPRX is a **freely floating native cryptocurrency**. Its market value is determined dynamically by open global market supply and demand. It is **NOT** a stablecoin, nor is it pegged to the Indian Rupee (INR), United States Dollar (USD), or any other sovereign currency.
2. **Sub-Unit Precision (18 Decimals)**: SPRX supports 18 decimal places of sub-unit precision ($10^{18}$ base atomic units).

```
+---------------------------------------------------------------------------------------------------+
|                                  SPRX DENOMINATION SCALE                                          |
+---------------------------------------------------------------------------------------------------+
| Unit Name            | Multiplier (in atto-SPRX)  | Multiplier (in SPRX)  | Common Usage          |
+----------------------+----------------------------+-----------------------+-----------------------+
| atto-SPRX (wei-eq)   | 1                          | 10^-18                | Base EVM / VM Unit    |
| nano-SPRX (gwei-eq)  | 1,000,000,000 (10^9)       | 10^-9                 | Gas Price Quotation   |
| micro-SPRX           | 1,000,000,000,000 (10^12)  | 10^-6                 | Micro-transactions    |
| milli-SPRX           | 10^15                      | 10^-3                 | Sub-unit display      |
| SPRX (Whole Unit)    | 10^18                      | 1.0                   | Standard Native Asset |
| kilo-SPRX (kSPRX)    | 10^21                      | 10^3 (1,000)          | Staking & Delegation  |
| mega-SPRX (MSPRX)    | 10^24                      | 10^6 (1,000,000)      | Treasury & Supply     |
+----------------------+----------------------------+-----------------------+-----------------------+
```

---

## 2. Decoupled Global Currency Display Architecture

While consensus and on-chain state execution operate strictly in `atto-SPRX`, end-users interact in familiar global domestic currencies.

```
+---------------------------------------------------------------------------------------------------+
|                                 CURRENCY PRESENTATION FLOW                                        |
+---------------------------------------------------------------------------------------------------+
|                                                                                                   |
|   [On-Chain State: 2,500.00 SPRX]                                                                 |
|                |                                                                                  |
|                v                                                                                  |
|   [Decentralized Oracle Multi-Currency Price Feed]                                                |
|   - SPRX/USD: $4.200                                                                              |
|   - USD/INR:  ₹87.50   ->  Derived SPRX/INR: ₹367.50                                              |
|   - USD/EUR:  €0.920   ->  Derived SPRX/EUR: €3.864                                               |
|   - USD/GBP:  £0.790   ->  Derived SPRX/GBP: £3.318                                               |
|   - USD/JPY:  ¥155.0   ->  Derived SPRX/JPY: ¥651.00                                              |
|                |                                                                                  |
|                +-------------------------+-------------------------+                              |
|                |                         |                         |                              |
|                v                         v                         v                              |
|     [Indian User Display]      [US User Display]        [European User Display]                   |
|     ₹9,18,750.00 INR           $10,500.00 USD           €9,660.00 EUR                             |
|     (Formatted in Lakhs)       (Standard formatting)    (Standard formatting)                     |
|                                                                                                   |
+---------------------------------------------------------------------------------------------------+
```

### Presentation Guidelines for Wallets and Explorers:
- **Zero Consensus Contamination**: The conversion rate is completely off-chain/oracle-driven. If the price oracle goes offline, transactions continue executing seamlessly on-chain using native gas.
- **Localized Formatting**:
  - **INR Display**: Adheres to the Indian numbering system: thousands, lakhs, and crores (e.g., `₹12,45,000.00`).
  - **USD / EUR / GBP Display**: Standard Western international notation (e.g., `$1,245,000.00`).

---

## 3. Initial Genesis Supply & Allocation

The initial genesis supply of the SPRX Protocol is capped at **1,000,000,000 SPRX** (1 Billion SPRX).

```
+---------------------------------------------------------------------------------------------------+
|                                  GENESIS ALLOCATION BREAKDOWN                                     |
+---------------------------------------------------------------------------------------------------+
| Allocation Category            | Percentage | Amount (SPRX)    | Vesting Schedule                 |
+--------------------------------+------------+------------------+----------------------------------+
| Staking & Security Reserve     | 35.0%      | 350,000,000 SPRX | Emitted over 10 years via PoS    |
| Ecosystem & Developer Grants   | 25.0%      | 250,000,000 SPRX | 5-year linear vest, 6-mo cliff   |
| Community Treasury (Gov DAO)   | 15.0%      | 150,000,000 SPRX | Unlocked via on-chain governance |
| Core Contributors & Team       | 15.0%      | 150,000,000 SPRX | 4-year linear vest, 1-year cliff |
| Early Backers & Seed Funding   | 10.0%      | 100,000,000 SPRX | 2-year linear vest, 6-mo cliff   |
| Total Initial Genesis Supply   | 100.0%     | 1,000,000,000    | Hardcoded at Block Height 0      |
+--------------------------------+------------+------------------+----------------------------------+
```

---

## 4. Dynamic Inflation & Issuance Formula

SPRX utilizes an algorithmic, self-balancing inflation curve that adjusts block rewards based on the network's **Bonding Ratio** (percentage of circulating SPRX staked by validators).

### 4.1 Inflation Variables & Bounds
- **Target Staking Bonding Ratio ($B_{target}$)**: $67.0\%$ ($0.67$).
- **Minimum Annual Inflation Rate ($I_{min}$)**: $3.0\%$ per annum.
- **Maximum Annual Inflation Rate ($I_{max}$)**: $8.0\%$ per annum.
- **Inflation Adjustment Factor ($\Delta I$)**: $0.02$ ($2.0\%$ per annum max adjustment).

### 4.2 Inflation Update Formula
At each epoch boundary $E$:
$$B_{actual} = \frac{\text{TotalBondedStake}}{\text{TotalCirculatingSupply}}$$

$$I_{t+1} = I_t + \left(1 - \frac{B_{actual}}{B_{target}}\right) \cdot \Delta I$$

Bounded strictly by:
$$I_{min} \le I_{t+1} \le I_{max}$$

```
Behavior:
- If B_actual < 67% (Low Staking Participation): Inflation rises up to 8.0%, increasing staking yields to incentivize token holders to bond and secure the network.
- If B_actual > 67% (High Staking Participation): Inflation drops towards 3.0%, reducing token dilution and encouraging capital deployment into DeFi / real-world dApps.
```

---

## 5. Transaction Fee Economics & EIP-1559 Dynamic Gas Burn

SPRX enforces a dual-component fee model with algorithmic base fee burning to create structural deflation during high network utilization.

```
+---------------------------------------------------------------------------------------------------+
|                                  TRANSACTION FEE FLOW (EIP-1559)                                  |
+---------------------------------------------------------------------------------------------------+
|                                                                                                   |
|   User Transaction Fee = GasUsed * (BaseFee + PriorityTip)                                        |
|                                |                                                                  |
|               +----------------+----------------+                                                 |
|               |                                 |                                                 |
|               v                                 v                                                 |
|   [Base Fee: 100% BURNED]          [Priority Tip: Validator Reward]                               |
|   - Permanently destroyed          - Transferred directly to proposer                             |
|   - Reduces total SPRX supply      - Incentivizes fast inclusion                                  |
|   - Deflationary pressure          - Direct economic incentive                                    |
|                                                                                                   |
+---------------------------------------------------------------------------------------------------+
```

### 5.1 Dynamic Base Fee Adjustment Formula
Target block gas is set to $50\%$ of maximum block gas limit:
$$G_{target} = \frac{G_{max}}{2}$$

The base fee for the next block $H+1$ adjusts based on the current block's gas utilization:
$$\text{BaseFee}_{H+1} = \text{BaseFee}_H \cdot \left(1 + \frac{1}{8} \cdot \frac{G_H - G_{target}}{G_{target}}\right)$$

- If $G_H = G_{target}$: $\text{BaseFee}_{H+1} = \text{BaseFee}_H$ (Unchanged).
- If $G_H = G_{max}$ (Full Block): $\text{BaseFee}_{H+1} = \text{BaseFee}_H \cdot 1.125$ ($+12.5\%$ max increase per block).
- If $G_H = 0$ (Empty Block): $\text{BaseFee}_{H+1} = \text{BaseFee}_H \cdot 0.875$ ($-12.5\%$ max decrease per block).

---

## 6. Staking Yield & Validator Economics

Validators and delegators earn staking yields composed of newly minted inflationary rewards plus priority gas tips, minus commission and burnt base fees.

### 6.1 Nominal Annual Percentage Rate (APR) Formula
$$\text{Nominal APR} = \frac{\text{AnnualInflationSupply} + \text{AnnualPriorityTips}}{\text{TotalBondedStake}}$$

$$\text{Net Real APR} = \text{Nominal APR} - I_{actual} - \text{AnnualBurnRate}$$

```
Example Staking Yield Scenario:
- Total Supply: 1,000,000,000 SPRX
- Total Bonded Stake: 670,000,000 SPRX (Bonding Ratio = 67%)
- Current Inflation Rate: 5.5% (55,000,000 SPRX emitted per year)
- Annual Priority Tips: 5,000,000 SPRX

Gross Nominal Staking APR = (55,000,000 + 5,000,000) / 670,000,000 = 8.95%
With a 5% Validator Commission: Delegator Net Yield = 8.95% * 0.95 = 8.50%
```

### 6.2 Economic Flywheel
1. **Network Growth**: High transaction volume burns massive quantities of base fee SPRX.
2. **Deflationary Turning Point**: If annual burnt SPRX exceeds annual newly minted inflation, SPRX becomes net-deflationary (Supply contraction: $\Delta S < 0$).
3. **Staker Value Accretion**: Stakers earn real yield backed by authentic transaction demand and floating global utility.
