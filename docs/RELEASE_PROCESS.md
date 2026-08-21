# SPRX Protocol: Release Management & Mainnet Launch Sequence
**Document Version:** 1.0.0  
**Target:** Release Engineers, Core Developers, Genesis Validators

---

## 1. 10-Step Mainnet Launch Sequence

```mermaid
sequenceDiagram
    autonumber
    participant Rel as Release Engineer
    participant Gen as Genesis Coordinator
    participant Val as Genesis Validators
    participant Net as SPRX Mainnet
    participant Pub as Public (RPC / Explorer)

    Rel->>Rel: 1. Tag v1.0.0 & Generate Reproducible Docker Binaries
    Rel->>Gen: 2. Distribute SHA-256 Checksums & Genesis Config
    Gen->>Gen: 3. Genesis Hash Verification Ceremony (Blake3)
    Gen->>Val: 4. Coordinated Genesis Broadcast (T-24h)
    Val->>Net: 5. Connect P2P Sentry Tunnels & Bootstrap Peers
    Val->>Net: 6. Block #0 State Commit at Genesis Timestamp
    Net->>Net: 7. BFT Round #1 Propose/Prevote/Precommit (+2/3 Quorum)
    Net->>Rel: 8. Block #1 Finality Verified
    Net->>Pub: 9. Open Public RPC (:26657) & Start Indexer
    Pub->>Pub: 10. Web Explorer & Wallet Endpoints Live
```

---

## 2. Reproducible Build & Verification Process

### 2.1 Generating Signed Release Checksums
```bash
# 1. Deterministic Container Build
docker build --no-cache -t sprax:v1.0.0 -f docker/Dockerfile .

# 2. Extract Binary and Compute SHA-256
sha256sum target/release/sprax > SHA256SUMS.txt
gpg --armor --detach-sign SHA256SUMS.txt
```

### 2.2 Genesis File Hash Verification
Genesis validators must independently compute and verify the Blake3 genesis hash prior to node startup:
```bash
sprax genesis verify --file genesis.mainnet.json
# Expected Blake3 Hash: sha256 / blake3 verified commitment
```
