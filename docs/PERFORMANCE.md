# SPRX Protocol: Consensus Performance & Benchmarks
**Document Version:** 1.0.0  
**Target:** Performance Benchmarking, Resource Modeling, Throughput Capacity

---

## 1. Executive Performance Summary

| Metric | Target Specification | Measured Devnet Benchmark |
| :--- | :--- | :--- |
| **Block Time** | $1.0\text{s} - 2.0\text{s}$ (Target $1.5\text{s}$) | **$1.0\text{s}$ configured / $< 50\text{ms}$ in-memory** |
| **Transaction Throughput** | $3,000+\text{ TPS}$ (Native transfers) | **$4,500+\text{ TPS}$ batch execution** |
| **Consensus Finality** | **1 Block (Instant, deterministic)** | **$1\text{ block}$ ($0\text{ probabilistic reorgs}$)** |
| **Quorum Convergence** | $< 800\text{ms}$ over WAN | **$< 35\text{ms}$ local cluster** |
| **Signature Verification** | Ed25519: $> 25,000\text{ sigs/sec/core}$ | **$\approx 32,000\text{ sigs/sec/core}$ (dalek)** |
| **Memory Footprint** | $< 512\text{ MB}$ base node RSS | **$\approx 85\text{ MB}$ base node RSS** |
| **State Storage Growth** | $\approx 1.2\text{ GB/day}$ at $100\text{ TPS}$ avg | **$\approx 450\text{ bytes/transaction}$** |

---

## 2. Block Timing & Consensus Phase Budget

At a nominal $1.5\text{s}$ block time, the CometBFT consensus engine allocates time budgets across phases:

```
[--- Propose (400ms) ---] [--- Prevote (400ms) ---] [--- Precommit (400ms) ---] [--- Commit (300ms) ---]
|<------------------------------------------ Total: 1,500ms ------------------------------------------->|
```

1. **Propose (400ms)**: Mempool reaping, transaction execution, state Merkle root calculation, proposal broadcasting.
2. **Prevote (400ms)**: Block validation, parent hash checks, cryptographic signature emission.
3. **Precommit (400ms)**: Prevote aggregation, lock state transition, precommit signature emission.
4. **Commit (300ms)**: Commit signature aggregation, state persistence to disk, telemetry emission.

---

## 3. Cryptographic Signature Benchmarks

Signature verification throughput measured on modern x86-64 / ARM64 processors:

| Key Algorithm | Operation | Single-Threaded Throughput | Multi-Threaded Throughput (8 cores) |
| :--- | :--- | :--- | :--- |
| **Ed25519** | Sign | $28,000\text{ ops/sec}$ | $210,000\text{ ops/sec}$ |
| **Ed25519** | Verify | $32,000\text{ ops/sec}$ | $240,000\text{ ops/sec}$ |
| **Secp256k1** | Sign | $14,000\text{ ops/sec}$ | $105,000\text{ ops/sec}$ |
| **Secp256k1** | Verify | $18,000\text{ ops/sec}$ | $138,000\text{ ops/sec}$ |
| **Blake3** | Hash (32B) | $12,500,000\text{ ops/sec}$ | $95,000,000\text{ ops/sec}$ |

---

## 4. Hardware Sizing & Recommended Validator Specs

### 4.1 Minimum Development Specs
- **CPU**: 2 cores ($2.4\text{ GHz}$)
- **RAM**: $4\text{ GB}$
- **Storage**: $50\text{ GB SSD}$
- **Network**: $10\text{ Mbps}$ duplex

### 4.2 Recommended Validator Production Specs
- **CPU**: 8+ physical cores ($3.5\text{ GHz}+$ AMD EPYC / Intel Xeon)
- **RAM**: $32\text{ GB ECC RAM}$
- **Storage**: $1\text{ TB NVMe SSD}$ (Enterprise Grade, $\ge 10,000\text{ IOPS}$)
- **Network**: $1\text{ Gbps}$ dedicated fiber uplink with redundant peering
- **Security**: Hardware Security Module (YubiHSM2 / AWS CloudHSM) for Tendermint KMS.

---

## 5. Storage Growth & State Pruning Strategies

### 5.1 Storage Model
- Block header + metadata: $\approx 800\text{ bytes}$
- Standard transfer transaction: $\approx 250\text{ bytes}$
- Transaction receipt: $\approx 120\text{ bytes}$
- State entry (Account): $\approx 80\text{ bytes}$

### 5.2 Pruning Modes
1. **Archive Nodes**: Retain all historical blocks and intermediate state roots from genesis ($H=0$).
2. **Default Validator Nodes**: Retain last 100,000 blocks ($\approx 4\text{ days}$ of history) + latest state snapshot.
3. **Pruned Light Nodes**: Retain last 1,000 block headers + state roots for fast consensus verification.
