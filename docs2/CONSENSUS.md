# SPRX Protocol: Consensus Architecture & Mathematical Specification
**Document Version:** 1.0.0-PROPOSAL  
**Status:** ARCHITECTURE_ONLY (Phase 01)  
**Consensus Model:** Byzantine Fault Tolerant Proof-of-Stake (BFT-PoS)

---

## 1. Consensus Philosophy & Core Requirements

SPRX employs a high-throughput, low-latency **Byzantine Fault Tolerant Proof-of-Stake (BFT-PoS)** state-machine replication algorithm. The consensus engine guarantees **safety** (no conflicting blocks finalized) and **liveness** (continuous block production) under partially synchronous network conditions.

### Core Consensus Requirements
1. **Deterministic Finality**: Immediate finality upon $\ge \frac{2}{3}W + 1$ validator voting weight commitment. Zero probabilistic rollbacks or chain reorganizations.
2. **Sub-2-Second Block Time**: Target block generation interval of $t_{block} \in [1.0s, 2.0s]$ with sub-second commit latency.
3. **Byzantine Fault Resilience**: Guarantees safety under up to $f < \frac{1}{3}$ malicious or arbitrarily faulty voting weight, and liveness under $f < \frac{1}{3}$ offline weight.
4. **Light Client Verifiability**: Header commitments include Merkle roots and aggregated validator signatures, enabling compact $O(1)$ to $O(K)$ cryptographic verification for mobile clients and cross-chain bridges.
5. **Deterministic BFT-Time**: Time is calculated as the weighted median of validator timestamps, strictly preventing time-jacking or timestamp manipulation attacks.

---

## 2. Mathematical Formalization

### 2.1 Validator Weights & Quorum Bounds

Let the active validator set at height $H$ be defined as:
$$\mathcal{V} = \{v_1, v_2, \dots, v_N\}$$

Each validator $v_i$ possesses an active voting power $w_i \in \mathbb{N}^+$, derived from their bonded self-stake and delegated stake:
$$w_i = \lfloor \text{BondedStake}(v_i) \cdot 10^{-18} \rfloor$$

The total active network voting power $W$ is:
$$W = \sum_{i=1}^N w_i$$

The maximum allowable Byzantine voting power $f$ that the network can tolerate while maintaining absolute safety is strictly bounded by:
$$f < \frac{W}{3}$$

The **Consensus Quorum Threshold** $Q$ required to validate any consensus phase (Prevote, Precommit) is defined as:
$$Q = \left\lfloor \frac{2W}{3} \right\rfloor + 1$$

Any vote aggregation $\mathcal{A} \subseteq \mathcal{V}$ is valid if and only if:
$$\sum_{v_j \in \mathcal{A}} w_j \ge Q$$

---

### 2.2 Proposer Selection Mechanism

To prevent centralized coordination bottlenecks and denial-of-service targeting, SPRX utilizes a **Deterministic Weighted Round-Robin (DWRR)** algorithm augmented by an optional **Verifiable Random Function (VRF)** seed.

#### Algorithm: Deterministic Weighted Round-Robin Proposer Selection

At each consensus round $R$ for height $H$, each validator maintains an internal accumulator priority $p_i \in \mathbb{Z}$, initialized to $0$ at validator set formation.

1. **Priority Accumulation**:
   For every validator $v_i \in \mathcal{V}$:
   $$p_i \leftarrow p_i + w_i$$

2. **Proposer Selection**:
   The designated proposer $v^*$ is the validator with the maximum accumulated priority:
   $$v^* = \arg\max_{v_i \in \mathcal{V}} (p_i)$$
   *(Ties are broken deterministically by sorting validator public keys lexicographically).*

3. **Priority Deduction**:
   The selected proposer's priority is decremented by the total voting power $W$:
   $$p^* \leftarrow p^* - W$$

```
Example (3 Validators with weights w1=50, w2=30, w3=20; Total W=100):
Round 1: Priorities = [50, 30, 20]  -> Max = v1 (50) -> New Priorities = [-50, 30, 20]
Round 2: Priorities = [0, 60, 40]   -> Max = v2 (60) -> New Priorities = [0, -40, 40]
Round 3: Priorities = [50, -10, 60] -> Max = v3 (60) -> New Priorities = [50, -10, -40]
Round 4: Priorities = [100, 20, -20]-> Max = v1 (100)-> New Priorities = [0, 20, -20]
Proposer sequence: v1, v2, v3, v1... Exactly proportional to stake distribution.
```

---

### 2.3 Consensus State Machine (Rounds & Steps)

Each block height $H$ proceeds through sequential rounds $R \ge 0$. Each round comprises five distinct phases:

```
+---------------------------------------------------------------------------------------------------+
|                                  BFT-PoS CONSENSUS ROUND STEPS                                    |
+---------------------------------------------------------------------------------------------------+
|                                                                                                   |
|   +--------------+      +--------------+      +--------------+      +--------------+              |
|   | 1. Propose   | ---> | 2. Prevote   | ---> | 3. Precommit | ---> | 4. Commit    | ---> Height+1|
|   +--------------+      +--------------+      +--------------+      +--------------+              |
|          |                     |                     |                                            |
|          | Timeout             | Timeout / Nil       | Timeout / Nil                              |
|          v                     v                     v                                            |
|   +----------------------------------------------------------+                                    |
|   |                     Round R <- R + 1                     |                                    |
|   +----------------------------------------------------------+                                    |
+---------------------------------------------------------------------------------------------------+
```

#### Step 1: Propose
- The designated proposer $v^*$ gathers pending transactions from the mempool, constructs a block proposal $B = \langle H, R, Txs, StateRoot, LastCommitInfo \rangle$, signs the proposal with their consensus key, and broadcasts `Proposal(H, R, B)` over GossipSub.

#### Step 2: Prevote
- Each validator $v_i$ receives $B$, verifies:
  1. $v^*$ is the legitimate proposer for $\langle H, R \rangle$.
  2. Block header adheres to structural invariants and parent hash links.
  3. Transactions are valid and state transitions execute deterministically.
- If valid, $v_i$ broadcasts $\text{Prevote}(H, R, \text{Hash}(B))$.
- If invalid or timed out after $\Delta_{propose}$, $v_i$ broadcasts $\text{Prevote}(H, R, \text{nil})$.

#### Step 3: Precommit
- If a validator receives $\ge Q$ prevotes for $\text{Hash}(B)$ (termed a **Polka**):
  - The validator locks on $B$ and sets $\text{lockedRound} \leftarrow R, \text{lockedBlock} \leftarrow B$.
  - The validator broadcasts $\text{Precommit}(H, R, \text{Hash}(B))$.
- If the validator receives $\ge Q$ prevotes for $\text{nil}$ or times out after $\Delta_{prevote}$:
  - The validator broadcasts $\text{Precommit}(H, R, \text{nil})$.

#### Step 4: Commit & Finalize
- Upon receiving $\ge Q$ precommits for $\text{Hash}(B)$:
  - The block $B$ is irrevocably committed to the canonical ledger at height $H$.
  - State transitions are committed to disk storage.
  - Consensus advances to $H + 1$, resetting $R \leftarrow 0$.
- If $\ge Q$ precommits are not received within $\Delta_{precommit}$, round $R$ times out, incrementing $R \leftarrow R + 1$.

---

### 2.4 BFT-Time Calculation

To ensure absolute time determinism without relying on centralized time servers (NTP), block timestamps are computed as the **Weighted Median** of the timestamps submitted in validator precommits from the previous block commit:

Let the commit signatures for block $H-1$ be $\mathcal{C} = \{(v_1, t_1), (v_2, t_2), \dots, (v_m, t_m)\}$ where $t_1 \le t_2 \le \dots \le t_m$.

The deterministic block timestamp $T_{block}(H)$ is the timestamp $t_k$ satisfying:
$$\sum_{j=1}^{k-1} w_j < \frac{W_{commit}}{2} \quad \text{and} \quad \sum_{j=1}^k w_j \ge \frac{W_{commit}}{2}$$

**Monotonicity Invariant:**
$$T_{block}(H) > T_{block}(H-1)$$

---

## 3. Dynamic Validator Set & Staking Dynamics

### 3.1 Active Validator Set Election
- **Maximum Active Set Size**: $K_{max} = 125$ validators.
- **Minimum Self-Bond**: $10,000 \text{ SPRX}$ required to register a candidate validator.
- **Dynamic Rebalancing**: At every epoch boundary ($E = 10,000 \text{ blocks} \approx 4.5 \text{ hours}$), all bonded candidate validators are ranked by total stake (self-stake + delegations). The top $K_{max}$ constitute the active validator set $\mathcal{V}_{E+1}$.

### 3.2 Delegation & Commission Structure
- Any SPRX token holder may delegate any amount of SPRX to active or candidate validators.
- Validators define a commission rate $c \in [0.00, 0.20]$ (max $20\%$), with a maximum daily change rate bounded by $1.0\%$ per 24 hours to protect delegators.
- Delegators earn block rewards proportional to their delegated stake, minus validator commission:
$$R_{delegator} = R_{gross} \cdot (1 - c) \cdot \frac{\text{DelegatorStake}}{\text{TotalValidatorStake}}$$
$$R_{validator} = (R_{gross} \cdot c) + \left(R_{gross} \cdot (1 - c) \cdot \frac{\text{ValidatorSelfStake}}{\text{TotalValidatorStake}}\right)$$

### 3.3 Unbonding & Slashing Period (Cooling-Off)
- **Unbonding Duration**: $T_{unbond} = 21 \text{ days}$ ($1,209,600 \text{ blocks}$ at $1.5s$ block time).
- During unbonding:
  - Tokens cease earning block rewards immediately.
  - Tokens cannot be transferred or re-delegated.
  - Tokens remain fully susceptible to slashing if Byzantine evidence is submitted for offenses committed during the validator's active tenure prior to unbonding.

---

## 4. Slashing Protocol & Byzantine Fault Evidence

```
+---------------------------------------------------------------------------------------------------+
|                                  VALIDATOR SLASHING MATRIX                                        |
+---------------------------------------------------------------------------------------------------+
| Offense Type           | Detection Mechanism        | Slash Penalty | Jailing Duration | Status   |
+------------------------+----------------------------+---------------+------------------+----------+
| Double Signing (Equiv) | 2 Conflicting Signed Msgs  | 10.0% Stake   | Permanent (Tomb) | Banned   |
| Liveness Downtime      | Missed >50% in 10k window  | 0.1% Stake    | 3,600 Blocks     | Jailed   |
| Malicious Forking      | Conflicting Commit Proofs  | 20.0% Stake   | Permanent (Tomb) | Banned   |
| Invalid State Proposal | Bad StateRoot Computation  | 5.0% Stake    | 86,400 Blocks    | Jailed   |
+---------------------------------------------------------------------------------------------------+
```

### 4.1 Double-Signing (Equivocation)
- **Definition**: A validator signs two distinct block proposals or two conflicting Prevote/Precommit votes for the same height $H$ and round $R$.
- **Evidence Verification**: Any full node can submit an `EquivocationEvidence` transaction containing the two conflicting signatures signed by the same validator key $\text{PubKey}_{val}$.
- **Penalty**:
  1. Instant slashing of $10\%$ of all bonded and delegated tokens.
  2. Immediate and permanent jailing (tombstoning) — the validator is permanently barred from ever participating in consensus.
  3. $1\%$ of the slashed amount is awarded to the evidence submitter as a whistleblower bounty; the remaining $99\%$ is burned permanently, reducing total SPRX supply.

### 4.2 Liveness Downtime (Non-Responsiveness)
- **Tracking Window**: Sliding window of $W_{live} = 10,000 \text{ blocks}$.
- **Threshold**: A validator must sign at least $5,000$ blocks ($50\%$) within the window.
- **Penalty**:
  1. Minor slashing of $0.1\%$ of bonded stake.
  2. Temporary jailing for $T_{jail} = 3,600 \text{ blocks}$ ($\sim 1.5 \text{ hours}$).
  3. Validator must submit a signed `MsgUnjail` after the jail duration expires to re-enter the active set.

---

## 5. Light Client Verification & Fast Sync Protocol

### 5.1 Light Client Verification
A light client verifies state authenticity without downloading transaction bodies or executing state transitions.

Given trusted header $H_{trusted}$ with validator set $\mathcal{V}_{trusted}$:
1. Light client requests header $H_{target}$ and commit proof $\mathcal{C}_{target}$.
2. Light client verifies that $\sum_{v_j \in \mathcal{C}_{target} \cap \mathcal{V}_{trusted}} w_j \ge \frac{1}{3}W_{trusted}$ (Bisection Verification).
3. If validator set changed, intermediate epoch validator set transition proofs are verified sequentially.
4. Light client verifies inclusion of account balance or contract state via Merkle-proof against `StateRoot` in $H_{target}$.

### 5.2 Fast State-Sync Protocol
New or recovering full nodes can bypass replaying millions of historical blocks via **State-Sync**:
1. Node requests the latest consensus-finalized snapshot metadata at height $H_{snap}$ (signed by $> \frac{2}{3}W$).
2. Node downloads state trie chunks in parallel across peer nodes.
3. Each chunk is verified against the Merkle subtree root hashes.
4. Once all chunks are assembled, the computed `StateRoot` is verified against $H_{snap}$.
5. Node begins participating in consensus immediately at height $H_{snap} + 1$.
