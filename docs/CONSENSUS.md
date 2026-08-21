# SPRX Protocol — Consensus & Mathematical Specification

## Overview
SPRX Protocol implements a 2-step **Byzantine Fault Tolerant Proof-of-Stake (BFT-PoS)** consensus state machine inspired by CometBFT. The consensus guarantees safety under partial synchrony as long as less than one-third ($< 1/3$) of the total active validator voting power is Byzantine or faulty.

---

## Quorum Mathematics

Let $V = \{v_1, v_2, \dots, v_n\}$ be the active validator set, and $w(v_i) \in \mathbb{N}$ be the voting power of validator $v_i$.
The total voting power is defined as:
$$W = \sum_{i=1}^n w(v_i)$$

The supermajority quorum threshold is strictly greater than two-thirds ($+2/3$) of the total voting power:
$$Q(P) = \left[ P > \frac{2}{3} W \right] \iff P \ge \left\lfloor \frac{2W}{3} \right\rfloor + 1$$

---

## Proposer Election (Deterministic Weighted Round Robin)

For each height $H$ and round $R$, the proposer is deterministically selected using DWRR:
1. Each validator maintains an internal accumulator priority $p_i$.
2. At the start of an election step, each validator's priority is incremented by its voting power:
   $$p_i \leftarrow p_i + w(v_i)$$
3. The validator with the highest priority $v^* = \arg\max_i (p_i)$ is chosen as the proposer.
4. The chosen proposer's priority is decremented by the total voting power:
   $$p^* \leftarrow p^* - W$$

---

## Slashing & Equivocation Prevention

- **Double-Signing (Equivocation)**: Producing two conflicting votes at the same $(H, R)$ with different block hashes results in immediate slashing of $5\%$ of staked tokens and permanent tombstoning.
- **Downtime**: Missing more than $50\%$ of blocks in a 10,000-block rolling window results in a $0.1\%$ downtime penalty and jailing for 24 hours.
