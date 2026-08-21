# SPRX Protocol: Comprehensive Security Architecture & Threat Matrix
**Document Version:** 1.0.0-PROPOSAL  
**Status:** ARCHITECTURE_ONLY (Phase 01)  
**Methodology:** STRIDE / DREAD Decentralized Threat Modeling

---

## 1. Security Philosophy & Cryptographic Invariants

Security in the SPRX Protocol is enforced by mathematical invariants, formal verification, strict isolation barriers, and defense-in-depth engineering. 

### Fundamental Security Mandates:
1. **Proven Cryptographic Primitives Only**: Zero custom cryptographic algorithms. Strict adherence to audited implementations:
   - `Ed25519` (RFC 8032) for consensus voting and native account transactions.
   - `Secp256k1` (RFC 6979) for EVM compatibility and hardware wallet signing.
   - `BLS12-381` for light-client aggregate attestations.
   - `Blake3`, `SHA-256`, and `Keccak-256` for deterministic hashing.
2. **Zero In-Memory Key Persistence**: Consensus private keys must NEVER reside on publicly accessible network interfaces. Mandatory Hardware Security Module (HSM) or AWS KMS / Vault integration with anti-double-sign persistent high-watermark tracking.
3. **Deterministic Sandbox Isolation**: Smart contracts execute within an isolated WebAssembly (Wasm) or EVM runtime with hard memory bounds, strict instruction gas metering, and zero host filesystem access.
4. **Replay Protection**: Strict inclusion of `ChainID` and monotonic account `Nonce` in every transaction signature payload.

---

## 2. Comprehensive Security Threat Matrix (32 Risk Classifications)

```
+---------------------------------------------------------------------------------------------------+
|                                  THREAT SEVERITY SUMMARY                                          |
+---------------------------------------------------------------------------------------------------+
|  [CRITICAL: 8 Risks]    |  Catastrophic protocol failure, state corruption, loss of funds         |
|  [HIGH: 10 Risks]       |  Consensus stalls, network partition, localized theft, severe DoS       |
|  [MEDIUM: 8 Risks]      |  Degraded network performance, MEV exploitation, state bloat            |
|  [LOW: 6 Risks]         |  RPC degradation, minor oracle delays, cosmetic UI desynchronization     |
+---------------------------------------------------------------------------------------------------+
```

---

### Category A: Consensus & Byzantine Faults (Risks 1 – 7)

#### Risk 1: Validator Equivocation (Double Signing)
- **Severity**: `CRITICAL`
- **Attack Vector**: A malicious or misconfigured validator signs two conflicting block proposals or Precommit votes for the same height $H$ and round $R$ to induce a chain split.
- **Impact**: Potential safety violation and conflicting ledger branches if $> 1/3$ voting power equivocates simultaneously.
- **Mitigation**:
  1. Instant slashing of $10\%$ bonded stake upon submission of dual-signed cryptographic evidence.
  2. Permanent jailing (tombstoning) of the offending validator key.
  3. Anti-double-signing high-water mark database enforced at the HSM driver level (refuses to sign any message with $Height \le H_{lastSigned}$).

#### Risk 2: Long-Range History Rewriting Attack
- **Severity**: `CRITICAL`
- **Attack Vector**: Former validators who unbonded their stake obtain old private keys and create an alternate historical chain branch starting from months ago where they had $> 2/3$ stake.
- **Impact**: New or recovering nodes could theoretically be tricked into accepting a false historical branch.
- **Mitigation**:
  1. Weak Subjectivity rule: Nodes reject any block headers older than the 21-day unbonding period ($T_{unbond}$).
  2. Light-client state-sync seeds must be pinned to trusted recent block hashes within the weak subjectivity horizon.

#### Risk 3: Byzantine 33% Liveness Halt (Cartel Stall)
- **Severity**: `HIGH`
- **Attack Vector**: A cartel of malicious validators controlling $> 33.4\%$ of total voting power goes offline or systematically votes `Prevote(Nil)`.
- **Impact**: The network cannot achieve the $Q = \lfloor \frac{2W}{3} \rfloor + 1$ quorum threshold, causing block production to freeze.
- **Mitigation**:
  1. Liveness downtime tracker immediately initiates slashing ($0.1\%$ per interval) against inactive validators.
  2. Gradual Inactivity Leak protocol: The inactive validators' stake is continuously burned each epoch, reducing their relative voting weight until active online validators recover $> 66.7\%$ voting power.

#### Risk 4: Byzantine 67% State Corruption Takeover
- **Severity**: `CRITICAL`
- **Attack Vector**: An attacker accumulates $> 66.7\%$ of total network stake through market cornering or massive coordinated validator key compromise.
- **Impact**: Full control over state transitions; attacker can finalize arbitrary fraudulent state transitions.
- **Mitigation**:
  1. Social consensus & emergency hard-fork coordination: The honest minority forks away the attacker's stake.
  2. Maximum delegation caps per single validator entity ($\le 10\%$ of total active stake).
  3. Multi-validator staking governance mechanisms.

#### Risk 5: Proposer Block Withholding & Censorship
- **Severity**: `HIGH`
- **Attack Vector**: A designated proposer collects fees but selectively censors specific user transactions or delays publishing proposals until the final milliseconds of $\Delta_{propose}$.
- **Impact**: Transaction delays, unfair MEV extraction, degraded user experience.
- **Mitigation**:
  1. Deterministic round-robin proposer rotation: Proposer only holds proposing authority for a single 1.5s block slot.
  2. Slashing and round incrementation ($R \leftarrow R+1$) if proposal is not received within $\Delta_{propose} = 1000\text{ms}$.

#### Risk 6: BFT-Time Manipulation (Time-Jacking)
- **Severity**: `MEDIUM`
- **Attack Vector**: A subset of colluding validators submit skewed timestamps in their precommit votes to manipulate contract time-locks.
- **Impact**: Premature unlocking of timelocked funds or gaming of time-dependent contracts.
- **Mitigation**:
  1. Block timestamp is calculated strictly as the **Weighted Median** of valid precommit timestamps.
  2. Timestamps deviating by more than $\pm 15\text{ seconds}$ from a node's local clock are rejected during validation.

#### Risk 7: Nothing-at-Stake in Fork Exploration
- **Severity**: `HIGH`
- **Attack Vector**: Validators attempt to vote on multiple competing block proposals simultaneously to maximize chances of earning rewards.
- **Impact**: Consensus instability and delayed finality.
- **Mitigation**:
  1. Strict BFT locking mechanism: A validator that receives a Polka ($\ge 2/3$ Prevotes for block $B$) is locked to $B$ and cannot precommit to any other block in subsequent rounds unless a newer Polka is proven.

---

### Category B: P2P & Network Layer Attacks (Risks 8 – 14)

#### Risk 8: Eclipse Attack on Full Nodes & Validators
- **Severity**: `HIGH`
- **Attack Vector**: An attacker inundates a target node with hundreds of fake peer connections from botnet IPs, isolating it from the honest P2P network.
- **Impact**: Node is fed stale or manipulated blockchain views, leading to double-spend vulnerability for localized merchants.
- **Mitigation**:
  1. Peer table diversification: Enforce connection limits per `/16` IPv4 subnet and `/32` IPv6 subnet.
  2. Persistent outbound connections to verified bootstrap peers.
  3. GossipSub peer scoring that rewards long-standing honest peers.

#### Risk 9: Sybil Attack on P2P Mesh
- **Severity**: `HIGH`
- **Attack Vector**: Attacker spins up 5,000 lightweight fake nodes to dominate GossipSub topics and intercept message flow.
- **Impact**: Propagation delays, memory consumption on honest nodes.
- **Mitigation**:
  1. Cryptographic Node Identity: PeerID bound to Ed25519 public key.
  2. Strict peer scoring penalty for redundant/useless gossip streams.
  3. Dynamic mesh pruning (GossipSub v1.1 limits active mesh to 8–12 high-scoring peers).

#### Risk 10: GossipSub Amplification & Memory Flooding DoS
- **Severity**: `HIGH`
- **Attack Vector**: Attacker broadcasts millions of invalid or oversized transaction messages across P2P topics.
- **Impact**: Network bandwidth saturation and out-of-memory crashes on full nodes.
- **Mitigation**:
  1. Strict message size limits ($128\text{ KB}$ per transaction, $4\text{ MB}$ per block).
  2. Stateless pre-validation: Discard messages with invalid signatures before placing in mempool or relaying to peers.
  3. Immediate peer disconnect and IP ban if invalid message rate exceeds 5 msgs/sec.

#### Risk 11: Validator IP Exposure & Direct DDoS
- **Severity**: `CRITICAL`
- **Attack Vector**: Attacker discovers the public IP of active consensus validators and launches a multi-terabit volumetric DDoS attack.
- **Impact**: Validators drop offline, leading to missed blocks and potential consensus halts.
- **Mitigation**:
  1. Mandatory Sentry Node Architecture: Validators have ZERO public IPs; communicate exclusively via private WireGuard tunnels to distributed sentry clusters.
  2. BGP Anycast and DDoS-scrubbing upstream filtering on sentry nodes.

#### Risk 12: Slowloris & TCP Connection Starvation
- **Severity**: `MEDIUM`
- **Attack Vector**: Attacker establishes maximum allowable TCP connections to a node and sends data at 1 byte per minute, exhausting node file descriptors.
- **Impact**: Legitimate peers and RPC clients are blocked from connecting.
- **Mitigation**:
  1. Aggressive read/write timeouts ($5\text{ seconds}$ inactivity disconnect).
  2. File descriptor limits monitored via systemd and cgroups.
  3. Primary transport over QUIC (UDP) which is immune to TCP SYN floods and connection starvation.

#### Risk 13: Transaction Malleability Attack
- **Severity**: `MEDIUM`
- **Attack Vector**: Attacker alters cryptographic signature encoding bytes ($S$ vs $-S \pmod N$) without invalidating the signature, changing the transaction hash before inclusion.
- **Impact**: Breaks transaction tracking in unconfirmed wallet pipelines and indexers.
- **Mitigation**:
  1. Enforce Low-$S$ signature normalization for Secp256k1.
  2. Canonical Protobuf/SSZ binary serialization with strict field ordering.
  3. Ed25519 native signatures which are mathematically non-malleable.

#### Risk 14: Cross-Chain / Cross-Network Replay Attack
- **Severity**: `HIGH`
- **Attack Vector**: A transaction signed on SPRX Testnet or Devnet is captured and replayed on SPRX Mainnet.
- **Impact**: Unauthorized asset transfer on Mainnet.
- **Mitigation**:
  1. Mandatory inclusion of a 64-bit `ChainID` (e.g., `SPRX-MAINNET-1`) in the hashed preimage of every transaction signature.

---

### Category C: Smart Contract & Virtual Machine Attacks (Risks 15 – 21)

#### Risk 15: Reentrancy Exploitation
- **Severity**: `CRITICAL`
- **Attack Vector**: Malicious contract calls back into a victim contract before the victim contract updates internal state balances (e.g., DAO-style exploit).
- **Impact**: Complete drainage of liquidity pools or contract token reserves.
- **Mitigation**:
  1. Checks-Effects-Interactions pattern enforced at compiler level.
  2. Native reentrancy lock built into the VM host execution environment (prevents recursive external invocations by default unless explicitly decorated).

#### Risk 16: Gas Metering Discrepancy & Non-Deterministic Execution
- **Severity**: `CRITICAL`
- **Attack Vector**: A crafted Wasm bytecode sequence consumes differing CPU cycle counts on x86_64 vs ARM64 CPU architectures, leading to differing out-of-gas results across validators.
- **Impact**: Consensus split (state fork) between node operators running different hardware architectures.
- **Mitigation**:
  1. Deterministic instruction-level gas metering (Wasm instrumentation injects fixed gas deduction counters before every basic block).
  2. Floating-point operations strictly banned or soft-floated with deterministic bitwise emulation.
  3. Comprehensive continuous fuzzing across multi-architecture CI pipelines.

#### Risk 17: Infinite Memory Allocation (Host OOM Crash)
- **Severity**: `HIGH`
- **Attack Vector**: Smart contract attempts to allocate gigabytes of virtual memory in a loop to trigger an operating system Out-Of-Memory (OOM) killer on validator nodes.
- **Impact**: Node crash and loss of consensus participation.
- **Mitigation**:
  1. Hard VM memory limit capped at $32\text{ MB}$ per execution context.
  2. Quadratic gas scaling cost for memory expansion:
     $$\text{Cost}_{mem} = C_{base} \cdot \text{pages} + \frac{\text{pages}^2}{512}$$

#### Risk 18: Front-Running & Toxic MEV Sandwich Attacks
- **Severity**: `MEDIUM`
- **Attack Vector**: Searchers monitor the public mempool, spot large user DEX swaps, and insert front-run buy and back-run sell transactions to extract value.
- **Impact**: Slippage losses and economic harm to end users.
- **Mitigation**:
  1. Encrypted Mempool / Threshold Decryption support: Transaction payloads remain encrypted until the block proposal is committed.
  2. Strict priority gas fee caps and fair-ordering transaction inclusion rules.

#### Risk 19: Storage Slot Clashing in Upgradeable Proxies
- **Severity**: `HIGH`
- **Attack Vector**: An upgrade to a proxy contract declares a new variable in a storage slot already occupied by a previous implementation variable.
- **Impact**: Silent state corruption, balance overwrites, loss of contract ownership.
- **Mitigation**:
  1. Mandatory adoption of ERC-1967 Storage Slots (unstructured storage at randomized hash offsets) or ERC-7201 Diamond Storage namespacing.
  2. Static analysis checks in CI during contract deployment verification.

#### Risk 20: Integer Overflow / Underflow Vulnerabilities
- **Severity**: `MEDIUM`
- **Attack Vector**: Arithmetic operations on token balances wrap around $2^{256}-1 \rightarrow 0$ or $0 - 1 \rightarrow 2^{256}-1$.
- **Impact**: Unauthorized minting of billions of tokens.
- **Mitigation**:
  1. SafeMath checked arithmetic by default in both Wasm (Rust `checked_add`) and EVM (Solidity $\ge 0.8.0$).
  2. VM halts with execution error upon arithmetic overflow.

#### Risk 21: Malicious Contract Deployment Resource Exhaustion
- **Severity**: `MEDIUM`
- **Attack Vector**: Attacker deploys contracts with deeply nested call trees or recursive contract deployments to exhaust call stack limits.
- **Impact**: Validator CPU spikes and delayed block execution.
- **Mitigation**:
  1. Call stack depth hard-capped at 64 frames.
  2. Bytecode size limit capped at $24.576\text{ KB}$ (EIP-170) or $128\text{ KB}$ for Wasm.
  3. High upfront deployment base gas fee.

---

### Category D: Storage, Node & Infrastructure Attacks (Risks 22 – 27)

#### Risk 22: State Bloat & Storage Exhaustion
- **Severity**: `HIGH`
- **Attack Vector**: Attacker sends millions of tiny dust transactions creating empty accounts and populating storage slots at negligible cost.
- **Impact**: State database explodes to multi-terabytes, pricing out honest validator operators and centralizing node operations.
- **Mitigation**:
  1. Non-zero minimum balance requirement for active account creation.
  2. High gas cost for new storage allocation ($20,000\text{ gas}$ for `SSTORE` from 0 to non-zero).
  3. State rent / storage deposit refund mechanism when storage slots are cleared.

#### Risk 23: LSM-Tree Write Amplification Exhaustion
- **Severity**: `MEDIUM`
- **Attack Vector**: Attacker queries and mutates randomly scattered storage keys to force continuous LSM compaction and maximize disk I/O.
- **Impact**: Disk I/O bottlenecks causing validator to miss block proposal deadlines.
- **Mitigation**:
  1. Jellyfish Merkle Tree layout with in-memory write buffer caching.
  2. NVMe SSD requirement ($> 5,000\text{ IOPS}$) with tuned compaction threads in RocksDB/Pebble.

#### Risk 24: Unindexed RPC Query Exhaustion DoS
- **Severity**: `MEDIUM`
- **Attack Vector**: External attackers send heavy JSON-RPC calls requesting 100,000 historical blocks or unindexed event logs simultaneously.
- **Impact**: RPC node CPU reaches 100%, starving normal transaction ingestion.
- **Mitigation**:
  1. RPC query limits: Maximum range of 2,000 blocks for `eth_getLogs`.
  2. Tiered API rate-limiting and query depth limits at the gateway load balancer.
  3. RPC read queries isolated from validator consensus nodes.

#### Risk 25: WAL Crash Corruption
- **Severity**: `HIGH`
- **Attack Vector**: Abrupt host power failure or kernel panic occurs during a heavy state commit cycle.
- **Impact**: Inconsistent database state preventing node reboot.
- **Mitigation**:
  1. Synchronous fsync write-ahead logging (WAL) prior to mutating LSM SST files.
  2. Automatic WAL replay and state rollback checksum verification on node startup.

#### Risk 26: Privileged Node Operator Key Extraction
- **Severity**: `CRITICAL`
- **Attack Vector**: Malicious insider or compromised server root access attempts to extract validator private signing keys from memory or disk.
- **Impact**: Full compromise of validator identity and stake slashing.
- **Mitigation**:
  1. Keys generated inside and never exported from Hardware Security Modules (YubiHSM2 / AWS CloudHSM).
  2. Node processes run under unprivileged systemd user (`sprx-daemon`) with read-only root filesystems and disabled core dumps (`ulimit -c 0`).

#### Risk 27: Snapshot Poisoning during State Sync
- **Severity**: `HIGH`
- **Attack Vector**: Malicious peers serve corrupted state snapshot chunks to a syncing node.
- **Impact**: Syncing node initializes with an invalid state and rejects honest blocks.
- **Mitigation**:
  1. State snapshot chunk hashes are cryptographically verified against the Merkle tree root of the snapshot manifest before ingestion.
  2. Final state root must match the consensus-signed header commit from trusted validators.

---

### Category E: Governance, Staking & Oracle Attacks (Risks 28 – 32)

#### Risk 28: Flash Loan Governance Attack
- **Severity**: `CRITICAL`
- **Attack Vector**: Attacker borrows 50,000,000 SPRX via a flash loan or lending pool, creates a malicious governance proposal, and passes it within a single block.
- **Impact**: Protocol parameters hijacked, treasury drained, or malicious code activated.
- **Mitigation**:
  1. Voting power snapshots taken at the proposal creation height $H_{prop}$ prior to vote start.
  2. 7-day minimum voting period combined with a mandatory 48-hour timelock execution delay.
  3. Flash-borrowed tokens cannot vote if acquired in the same block/epoch as the proposal.

#### Risk 29: Low Quorum Proposal Hijacking
- **Severity**: `HIGH`
- **Attack Vector**: Attacker submits a sneaky proposal during a holiday period and passes it with minimal votes due to voter apathy.
- **Impact**: Malicious parameter updates enacted without community consensus.
- **Mitigation**:
  1. Mandatory Quorum of $\ge 40\%$ of total circulating staked SPRX.
  2. Supermajority requirement ($\ge 66.7\%$) for critical protocol changes.
  3. No-With-Veto threshold ($\ge 33.4\%$) enables honest minority to veto and burn the malicious proposer's deposit.

#### Risk 30: Oracle Spot-Price Manipulation Exploit
- **Severity**: `CRITICAL`
- **Attack Vector**: Attacker manipulates a low-liquidity on-chain DEX pool within a single transaction to distort the SPRX/USD or SPRX/INR price feed consumed by lending protocols.
- **Impact**: Massive undercollateralized borrowing and insolvency of DeFi protocols.
- **Mitigation**:
  1. Multi-source decentralized oracle aggregation with median filtering.
  2. Mandatory Time-Weighted Average Price (TWAP) calculation over a minimum 30-minute window.
  3. Circuit breaker halts liquidations if price moves $> 15\%$ within a single block.

#### Risk 31: Stale Price Feed Delay Attack
- **Severity**: `MEDIUM`
- **Attack Vector**: Oracle nodes experience network latency or halt updates during high volatility, causing dApps to consume stale exchange rates.
- **Impact**: Arbitrageurs exploit stale rates against automated market makers.
- **Mitigation**:
  1. Maximum heartbeat timeout ($T_{heartbeat} = 60\text{ seconds}$).
  2. Transactions consuming oracle data revert automatically if `block.timestamp - oracle.lastUpdatedAt > 120s`.

#### Risk 32: Unbonding Queue Front-Running
- **Severity**: `LOW`
- **Attack Vector**: A validator anticipates impending slashing and initiates unbonding to escape penalties before evidence is submitted.
- **Impact**: Evasion of consensus slashing penalties.
- **Mitigation**:
  1. Slashing protocol retroactively penalizes all tokens that were bonded at the height of the infraction, even if currently in the 21-day unbonding queue.

---

## 3. Cryptographic Standards & Key Hygiene Summary

| Primitive | Approved Algorithm | Standard / RFC | Primary Usage |
| :--- | :--- | :--- | :--- |
| **Consensus Signatures** | Ed25519 | RFC 8032 | Block proposals, Prevote/Precommit voting |
| **Account Signatures** | Ed25519 & Secp256k1 | RFC 6979 / FIPS 186-4 | User transactions, smart contract invocations |
| **Attestation Aggregation**| BLS12-381 | draft-irtf-cfrg-bls-sig | Light client proofs, multi-sig commitments |
| **State Hashing** | Blake3 | Official 2020 Spec | JMT Trie nodes, block headers, chunk hashes |
| **EVM Compatibility Hashing**| Keccak-256 | FIPS 202 | EVM storage keys, Solidity ABI hashing |
| **Key Derivation** | Argon2id / PBKDF2 | RFC 9106 | Keystore file encryption (AES-256-GCM) |
| **HD Wallet Derivation**| BIP-32 / BIP-39 / BIP-44 | Bitcoin Standards | Hierarchical deterministic mnemonic wallets |
