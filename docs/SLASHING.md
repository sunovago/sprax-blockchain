# SPRX Protocol: Slashing & Byzantine Fault Penalties
**Document Version:** 1.0.0  
**Target:** Consensus Security, Slashing Mechanics, Evidence Verification

---

## 1. Objective & Security Mandate

Slashing provides economic finality and incentive alignment in SPRX Proof-of-Stake consensus. Malicious or negligent validator behaviors incur deterministic penalties on staked capital.

---

## 2. Slashing Conditions & Penalty Matrix

SPRX enforces two distinct slashing classes:

| Offense | Trigger Condition | Slash Fraction | Jail Duration | Tombstoned |
| :--- | :--- | :--- | :--- | :--- |
| **Equivocation (Double-Sign)** | Conflicting votes at same `(height, round, type)` | **5.00%** | $\infty$ (Permanent) | **YES** |
| **Liveness (Downtime)** | Missed $> 50$ blocks in sliding window | **0.01%** | 600 blocks ($15\text{ mins}$) | **NO** |

---

## 3. Detailed Slashing Mechanics

### 3.1 Equivocation (Double-Signing)
An equivocation occurs when a validator's consensus private key signs two distinct votes $\text{Vote}_A$ and $\text{Vote}_B$ such that:
$$\text{Vote}_A.\text{val} = \text{Vote}_B.\text{val} \land \text{Vote}_A.H = \text{Vote}_B.H \land \text{Vote}_A.R = \text{Vote}_B.R \land \text{Vote}_A.\text{type} = \text{Vote}_B.\text{type} \land \text{Vote}_A.\text{hash} \ne \text{Vote}_B.\text{hash}$$

#### Enforcement Procedure:
1. An `EquivocationEvidence` message containing both signed votes is submitted to the chain mempool.
2. `TxExecutor` verifies both signatures against the validator's registered `consensus_pubkey`.
3. If verified:
   - **5% of total bonded tokens** is permanently burned from the validator pool.
   - Validator status is immediately set to `Jailed`.
   - `is_tombstoned` flag is set to `true`.
   - Tombstoned validators can **never** rejoin the active set or unjail.

### 3.2 Liveness & Downtime Slashing
To guarantee continuous network liveness, validators must participate in block signing.

#### Enforcement Procedure:
1. For each finalized block, the consensus engine tracks signatures in `BlockHeader.signatures`.
2. If an active validator fails to provide a valid signature, its `missed_blocks_count` increments.
3. If `missed_blocks_count \ge 50`:
   - **0.01% of bonded tokens** is slashed.
   - Validator status becomes `Jailed`.
   - `jailed_until` is set to $H + 600\text{ blocks}$.
   - After block $H + 600$, the validator operator may submit `MsgUnjail` to resume participation.

---

## 4. Slashing Mathematical Impact on Delegators

Because delegations represent fractional shares in a validator pool:
$$\text{SharePrice} = \frac{\text{Validator Bonded Tokens}}{\text{Total Delegator Shares}}$$

When a validator is slashed:
1. `Validator.tokens` is decremented by the slash amount:
   $$\text{Tokens}_{\text{new}} = \text{Tokens}_{\text{old}} \times (1 - \text{SlashFraction})$$
2. Total shares remain unchanged, thereby immediately reducing the redeemable token value of all delegator shares proportionally.
3. This creates a strong incentive for delegators to conduct due diligence on validator infrastructure reliability.

---

## 5. Defense Against Common Byzantine Attacks

1. **Nothing-at-Stake Attack**: Completely prevented by the 5% double-signing slash penalty.
2. **Grinding Attacks**: Proposer selection is strictly deterministic (DWRR) and independent of previous block entropy.
3. **Long-Range Attacks**: Prevented by the 21-day unbonding lockup and weak subjectivity checkpoints.
4. **Censorship / Bribery Attacks**: If a minority validator coalition attempts censorship, governance slashing can burn the malicious stake.
