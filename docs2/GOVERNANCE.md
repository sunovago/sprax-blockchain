# SPRX Protocol: On-Chain Governance Architecture & Specification
**Document Version:** 1.0.0-PROPOSAL  
**Status:** ARCHITECTURE_ONLY (Phase 01)  
**Governance Model:** Token-Weighted On-Chain Governance with Timelock & Security Council

---

## 1. Governance Overview & Design Philosophy

The SPRX Protocol implements an on-chain, decentralized, stake-weighted governance mechanism designed to enable seamless protocol evolution, parameter optimization, community treasury allocation, and coordinated network upgrades without contentious chain splits.

### Core Governance Principles:
1. **Stake-Weighted Democratic Participation**: 1 Bonded SPRX = 1 Vote.
2. **Delegator Sovereignty & Vote Override**: Delegators inherit their chosen validator's vote by default, but maintain absolute cryptographic authority to cast an independent vote that overrides their validator's position.
3. **Sybil & Spam Resistance**: Strict upfront deposit requirements with a burn penalty for malicious proposals.
4. **Deliberate Timelock Delays**: Enforced time buffers between proposal approval and autonomous execution, providing node operators, exchanges, and users sufficient time to prepare.
5. **Separation of Powers**: A strictly constrained **Emergency Security Council** equipped only with defensive powers (pausing/delaying) with zero power to seize funds or alter balances.

---

## 2. Proposal Lifecycle & State Machine

```
+---------------------------------------------------------------------------------------------------+
|                                  GOVERNANCE PROPOSAL LIFECYCLE                                    |
+---------------------------------------------------------------------------------------------------+
|                                                                                                   |
|  [1. Draft / Forum RFC] ---> [2. On-Chain Deposit Period] ---> [3. Active Voting Period (7 Days)] |
|   Off-Chain Discourse          Requires 5,000 SPRX Deposit      Validators & Delegators Vote      |
|                                (Timeout: 7 Days to reach min)   (Yes, No, NoWithVeto, Abstain)    |
|                                                                                |                  |
|                                                                                v                  |
|  +---------------------------------------------------------------------------------------------+  |
|  |                                  4. TALLY & QUORUM EVALUATION                               |  |
|  +---------------------------------------------------------------------------------------------+  |
|        |                                       |                                    |             |
|        | Quorum >= 40% AND                     | NoWithVeto >= 33.4%                | Quorum < 40%|
|        | Yes > 50% (or 66.7% for upgrade)      | (Malicious / Scam Proposal)        | OR No > 50% |
|        v                                       v                                    v             |
|  [5. Timelock Queue (48 Hours)]         [REJECTED & BURNED]                    [REJECTED]         |
|   Pending Execution                     Deposit Burned (100%)                  Deposit Refunded   |
|        |                                                                                          |
|        v                                                                                          |
|  [6. Autonomous Execution]                                                                        |
|   State Parameters Mutated / Code Upgraded                                                        |
+---------------------------------------------------------------------------------------------------+
```

---

## 3. Proposal Classifications & Execution Thresholds

```
+---------------------------------------------------------------------------------------------------+
|                                  PROPOSAL THRESHOLD MATRIX                                        |
+---------------------------------------------------------------------------------------------------+
| Proposal Type          | Min Deposit | Voting Window | Quorum | Majority Threshold | Timelock     |
+------------------------+-------------+---------------+--------+--------------------+--------------+
| Parameter Change       | 5,000 SPRX  | 7 Days        | 40.0%  | > 50.0% (Simple)   | 48 Hours     |
| Community Pool Spend   | 2,500 SPRX  | 7 Days        | 33.4%  | > 50.0% (Simple)   | 24 Hours     |
| Software Upgrade       | 10,000 SPRX | 14 Days       | 50.0%  | >= 66.7% (Super)   | 72 Hours     |
| Text / Signaling       | 1,000 SPRX  | 7 Days        | 30.0%  | > 50.0% (Simple)   | None         |
| Emergency Security Fix | 25,000 SPRX | 48 Hours      | 66.7%  | >= 75.0% (Super)   | 6 Hours      |
+------------------------+-------------+---------------+--------+--------------------+--------------+
```

### 3.1 Parameter Change Proposals
Allows dynamic on-chain mutation of core protocol variables without software redeployment:
- **Consensus Parameters**: `BlockMaxGas`, `MaxBlockSizeBytes`, `TimeoutPropose`, `TimeoutPrevote`.
- **Staking Parameters**: `UnbondingPeriodHours`, `MaxActiveValidators`, `MinValidatorSelfBond`.
- **Slashing Parameters**: `SlashFractionDoubleSign`, `SlashFractionDowntime`, `DowntimeJailDuration`.
- **Fee Parameters**: `BaseFeeBurnPercentage`, `MinGasPrice`.

### 3.2 Software Upgrade Proposals
Signals a mandatory binary upgrade across all network nodes at a specific block height $H_{upgrade}$:
- Contains release tag, binary checksum hashes (SHA-256 for Linux/macOS/Windows x86_64 and ARM64), and execution plan.
- At height $H_{upgrade}$, nodes execute automated state migration handlers and verify consensus state roots before resuming normal block production.

---

## 4. Voting & Tallying Mathematics

Let $V_{total}$ be the total active bonded stake across the entire network at the start height of the voting period $H_{vote\_start}$.

Let the submitted votes on proposal $P$ be:
- $Y$: Total voting weight for `Yes`
- $N$: Total voting weight for `No`
- $V$: Total voting weight for `NoWithVeto`
- $A$: Total voting weight for `Abstain`

Total participating voting power:
$$V_{cast} = Y + N + V + A$$

### 4.1 Quorum Requirement
Proposal $P$ achieves valid quorum if and only if:
$$\frac{V_{cast}}{V_{total}} \ge \text{QuorumThreshold}$$

### 4.2 Anti-Spam / Malicious Proposal Defense (No-With-Veto)
Regardless of the number of `Yes` votes, if:
$$\frac{V}{Y + N + V} \ge 0.334 \quad (33.4\%)$$
Then:
1. Proposal $P$ is immediately **REJECTED**.
2. The entire proposer deposit is **100% BURNED** permanently.

### 4.3 Passing Condition
If Quorum is met and $\text{NoWithVeto} < 33.4\%$:
- For standard proposals: Proposal passes if:
$$\frac{Y}{Y + N + V} > 0.50 \quad (> 50.0\%)$$
- For software upgrades: Proposal passes if:
$$\frac{Y}{Y + N + V} \ge 0.667 \quad (\ge 66.7\%)$$

---

## 5. Emergency Security Council Architecture

To safeguard the protocol against unforeseen zero-day smart contract exploits or catastrophic consensus stalls, SPRX establishes a **Cryptographic Security Council**.

```
+---------------------------------------------------------------------------------------------------+
|                                 SECURITY COUNCIL GOVERNANCE GATES                                 |
+---------------------------------------------------------------------------------------------------+
|                                                                                                   |
|    [Security Council Multi-Sig (7-of-9 Reputable Ecosystem Entities)]                             |
|                                |                                                                  |
|               +----------------+----------------+                                                 |
|               |                                 |                                                 |
|               v                                 v                                                 |
|    [ALLOWED DEFENSIVE POWERS]         [PROHIBITED OFFENSIVE ACTIONS]                              |
|    - Pause compromised smart contract - CANNOT mint or burn tokens arbitrarily                    |
|    - Postpone upgrade height          - CANNOT seize or transfer user account balances            |
|    - Cancel malicious pending proposal- CANNOT bypass regular voting for permanent rule changes   |
|    - Enforce emergency rate-limits    - CANNOT override validator slashing without public vote    |
|                                                                                                   |
+---------------------------------------------------------------------------------------------------+
```

### Security Council Safeguards:
- **Time-Bounded Actions**: Any emergency pause expires automatically after 7 days unless ratified by a standard community governance vote.
- **On-Chain Transparency**: Every Security Council transaction must include an on-chain IPFS cryptographic hash linking to a detailed incident report and vulnerability post-mortem.
