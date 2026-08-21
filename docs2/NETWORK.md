# SPRX Protocol: Peer-to-Peer (P2P) Networking Specification
**Document Version:** 1.0.0-PROPOSAL  
**Status:** ARCHITECTURE_ONLY (Phase 01)  
**Protocol Stack:** libp2p / GossipSub v1.1 / Noise Protocol Framework

---

## 1. Network Layer Overview & Architecture

The SPRX network layer provides a resilient, authenticated, low-latency, and DoS-resistant peer-to-peer (P2P) communication substrate. It connects full nodes, validator sentries, RPC gateways, and archive nodes across global internet topology.

```
+---------------------------------------------------------------------------------------------------+
|                                  SPRX P2P PROTOCOL STACK                                          |
+---------------------------------------------------------------------------------------------------+
| [Application Layer]                                                                               |
|  - Consensus Voting (Prevote/Precommit)  - Block Proposals  - Tx Mempool Gossip  - State-Sync      |
+---------------------------------------------------------------------------------------------------+
| [PubSub Layer]                                                                                    |
|  - GossipSub v1.1 with Peer Scoring, Mesh Routing, and Sybil Throttling                          |
+---------------------------------------------------------------------------------------------------+
| [Discovery & Routing]                                                                             |
|  - Discv5 (Authenticated UDP) + Kademlia DHT (/sprx/kad/1.0.0) + Static Bootstrap Enodes          |
+---------------------------------------------------------------------------------------------------+
| [Security & Stream Multiplexing]                                                                  |
|  - Noise Protocol (XX Handshake) / TLS 1.3  - Yamux Multiplexer  - PeerID (Ed25519)               |
+---------------------------------------------------------------------------------------------------+
| [Transport Layer]                                                                                 |
|  - Multi-Transport: QUIC (UDP) primary, TCP fallback  - IPv4 / IPv6  - AutoNAT / Circuit Relay v2 |
+---------------------------------------------------------------------------------------------------+
```

---

## 2. Transport & Transport Security

### 2.1 Multi-Transport Protocol
SPRX utilizes **libp2p** multi-transports:
1. **QUIC (UDP)** *(Primary Transport)*:
   - Zero round-trip time (0-RTT) connection resumption.
   - Built-in stream multiplexing and native TLS 1.3 encryption.
   - Native resistance to head-of-line blocking under packet loss.
2. **TCP + Yamux** *(Fallback Transport)*:
   - Used in network environments blocking UDP traffic.
   - Multiplexed via Yamux over TLS 1.3 / Noise.

### 2.2 Transport Encryption & Node Identity
- **Node Identity (PeerID)**: Every node generates an Ed25519 keypair specifically for P2P identity. The `PeerID` is the SHA-256 multihash of the public key (e.g., `12D3KooW...`).
- **Noise Protocol Handshake (`Noise_XX_25519_ChaChaPoly_SHA256`)**:
  - Mutual authentication during connection setup.
  - Zero plaintext transmission over the wire.
  - Forward secrecy through ephemeral Diffie-Hellman key exchange.

---

## 3. Peer Discovery & Routing

```
+---------------------------------------------------------------------------------------------------+
|                                 PEER DISCOVERY & TOPOLOGY                                         |
+---------------------------------------------------------------------------------------------------+
|  [Bootstrap Nodes] --------> [Discv5 Discovery] --------> [Kademlia DHT] --------> [Active Mesh] |
|   Hardcoded Seeds             UDP Topic Advertisements     Peer Routing Table       8-12 Gossip   |
|   (DNS Seeds / IPs)           Signed ENR Records           Alpha=3 Lookups          Peers/Topic   |
+---------------------------------------------------------------------------------------------------+
```

### 3.1 Ethereum Node Record (ENR) / SPRX Node Record
Nodes advertise their identity and network capabilities via signed records containing:
- `peer_id`: Ed25519 public key multihash.
- `ip`: Public IPv4 / IPv6 address.
- `tcp_port` & `quic_port`: Transport listening ports.
- `chain_id`: SPRX network identifier (prevents cross-network peer contamination).
- `protocol_version`: Current protocol semantic version (e.g., `v1.0.0`).
- `node_role`: Bitmask flag (`0x01 = FullNode`, `0x02 = Sentry`, `0x04 = Archive`, `0x08 = Validator`).

### 3.2 Discovery Mechanisms
1. **Bootstrap Seeds**: Nodes connect to a geographically distributed set of hardcoded DNS and IP seed nodes on initial launch.
2. **Discv5 (Authenticated UDP Discovery)**: Fast peer discovery using UDP ping/pong and FindNodes queries without opening full TCP/QUIC streams.
3. **Kademlia DHT (`/sprx/kad/1.0.0`)**: Structured routing table with $k$-buckets ($k=20$) for routing queries and finding peers near specific network IDs.
4. **NAT Traversal**:
   - Automated UPnP port mapping.
   - **AutoNAT**: Discovers external reachability status via peer observation.
   - **Circuit Relay v2**: Limited relay fallback for nodes behind symmetric NATs.

---

## 4. GossipSub v1.1 & Topic Specifications

SPRX isolates network traffic across distinct **GossipSub v1.1** topics to optimize latency and prevent mempool spam from degrading consensus messages.

### 4.1 Topic Topography & Prioritization

| Topic Name | Message Content | Priority | Target Latency | Validation Rule |
| :--- | :--- | :--- | :--- | :--- |
| `/sprx/consensus/proposal/v1` | Block Proposals & VRF Seeds | Critical | $< 100\text{ms}$ | Proposer verification & signature check |
| `/sprx/consensus/vote/v1` | Prevote & Precommit Votes | Critical | $< 50\text{ms}$ | Active validator signature & valid height/round |
| `/sprx/consensus/evidence/v1`| Equivocation / Slashing Proofs | High | $< 500\text{ms}$ | Valid double-sign signature cryptographic proof |
| `/sprx/mempool/tx/v1` | Signed User Transactions | Medium | $< 1000\text{ms}$ | Nonce $\ge$ AccountNonce, Balance $\ge$ Fee, Valid Sig |
| `/sprx/sync/snapshot/v1` | State Snapshot Metadata & Chunks | Low | Background | Root hash matches consensus commitment |

---

## 5. Peer Scoring & Anti-Spam Defense

To guarantee resilience against Sybil, eclipse, and spam attacks, every node runs an internal **Peer Scoring Engine** tracking neighbor behavior:

$$S_{total} = w_{topic} \sum_{t} S_t + w_{app} S_{app} + w_{ip} S_{ip}$$

```
+---------------------------------------------------------------------------------------------------+
|                                    PEER SCORING THRESHOLDS                                        |
+---------------------------------------------------------------------------------------------------+
| Score Range                     | Node Action                                                     |
+---------------------------------+-----------------------------------------------------------------+
| Score >= 0                      | Healthy peer: Full mesh participation and gossip forwarding     |
| -50 <= Score < 0                | Warning: Peer placed on probation; excluded from active mesh    |
| -100 <= Score < -50             | Throttled: Incoming gossip rejected; outgoing messages dropped  |
| Score < -100                    | Banned: Connection severed immediately; IP greylisted for 24h   |
+---------------------------------+-----------------------------------------------------------------+
```

### Peer Scoring Factors
1. **Invalid Message Delivery ($P_1$)**: Severe negative score penalty if a peer broadcasts a syntactically invalid block or invalidly signed transaction.
2. **First-Message-Deliveries ($P_2$)**: Positive reward for peers that relay valid blocks/votes first.
3. **Mesh Message Deliveries ($P_3$)**: Reward for consistent delivery of subscribed topic messages.
4. **Behavior Penalty ($P_4$)**: Negative score if a peer sends duplicates or floods bandwidth exceeding $10\text{ MB/s}$.

---

## 6. Compact Block Propagation Protocol

To scale throughput to thousands of transactions per second without incurring massive network bandwidth spikes, SPRX utilizes **SPRX Compact Blocks** (derived from BIP-152 and Erlay).

```
+---------------------------------------------------------------------------------------------------+
|                                 COMPACT BLOCK RELAY FLOW                                          |
+---------------------------------------------------------------------------------------------------+
| Proposer Node                                                      Peer Node                      |
|      |                                                                 |                          |
|      |  1. CompactBlockHeader + Short Tx IDs (6 bytes each)            |                          |
|      |---------------------------------------------------------------->|                          |
|      |                                                                 |                          |
|      |                                                    2. Match Short IDs against local Mempool|
|      |                                                       Found: 98% of transactions in block  |
|      |                                                       Missing: 2 transactions              |
|      |                                                                 |                          |
|      |  3. GetBlockTxns(MissingIndices = [42, 107])                    |                          |
|      |<----------------------------------------------------------------|                          |
|      |                                                                 |                          |
|      |  4. BlockTxns(FullTxs = [Tx_42, Tx_107])                        |                          |
|      |---------------------------------------------------------------->|                          |
|      |                                                                 |                          |
|      |                                                    5. Assemble full block & execute state  |
+---------------------------------------------------------------------------------------------------+
```

### Bandwidth Savings Analysis:
- **Standard Block Relay (10,000 Txs)**: $\sim 2.5\text{ MB}$ raw data per peer.
- **Compact Block Relay (10,000 Txs)**: Header ($500\text{ bytes}$) + Short IDs ($60\text{ KB}$) + 2 missing txs ($500\text{ bytes}$) $\approx 61\text{ KB}$.
- **Net Bandwidth Reduction**: $> 97.5\%$ bandwidth savings; propagation latency dropped from $450\text{ms}$ to $< 25\text{ms}$.

---

## 7. Sentry Node Architecture for Validators

Validator nodes are high-value targets for Distributed Denial of Service (DDoS) and targeted network attacks. SPRX enforces a **Sentry Node Architecture**:

```
+---------------------------------------------------------------------------------------------------+
|                                 VALIDATOR SENTRY TOPOLOGY                                         |
+---------------------------------------------------------------------------------------------------+
|                                                                                                   |
|    [Public Internet / Peer Network]                                                               |
|        |                  |                  |                                                    |
|        v                  v                  v                                                    |
|  +--------------+   +--------------+   +--------------+                                           |
|  | Public       |   | Public       |   | Public       |  <--- Frontline Sentry Nodes              |
|  | Sentry 01    |   | Sentry 02    |   | Sentry 03    |       (Public IPs, DDoS-scrubbed)         |
|  +--------------+   +--------------+   +--------------+                                           |
|        |                  |                  |                                                    |
|        +------------------+------------------+                                                    |
|                           | (Encrypted WireGuard / Private Subnet)                                |
|                           v                                                                       |
|                 +-------------------+                                                             |
|                 | Isolated Core     |  <--- Validator Node                                        |
|                 | Validator Node    |       (NO Public IP, Zero Direct Inbound)                   |
|                 +-------------------+                                                             |
|                           |                                                                       |
|                           v (PKCS#11 / USB-IP)                                                    |
|                 +-------------------+                                                             |
|                 | Hardware Security |  <--- Private Signing Key Encapsulation                     |
|                 | Module (HSM)      |                                                             |
|                 +-------------------+                                                             |
+---------------------------------------------------------------------------------------------------+
```

### Sentry Topology Requirements:
1. **Zero Direct Exposure**: The validator node binds consensus ports strictly to private loopback/internal interfaces (`10.0.0.x` or WireGuard VPN `10.8.0.x`).
2. **Dedicated Sentry Scrubbing**: Sentries inspect, rate-limit, and validate all GossipSub packets before relaying them to the internal validator over private connections.
3. **Dynamic Sentry Cycling**: Validators can dynamically rotate sentries without interrupting consensus signing or exposing their core IP address.
