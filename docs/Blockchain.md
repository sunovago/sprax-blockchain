# Claude Code — Phase-Wise Task File (L1 Blockchain Project)

**Kaise use karein:** Har phase ka prompt copy karo, apne repo root mein Claude Code chalao, prompt paste karo. Ek phase complete + tested hone ke baad hi agla prompt do — sab ek saath mat do, agent scope mein bhatak jayega.

Har prompt ke start mein ye context line zaroor rakhna (already niche included hai):
> "Reference: architecture.md aur phases.md docs is repo ke docs/ folder mein hain — inhe follow karo."

Pehle in dono docs (blockchain-architecture.md, blockchain-project-structure.md, blockchain-phases-detailed.md) ko apne repo ke `docs/` folder mein daal do, taaki Claude Code inhe read kar sake.

---

## PHASE 1 PROMPT — Single-Node Prototype

```
Main ek production-grade L1 blockchain bana raha hoon in Rust. Ye Phase 1 hai: single-node prototype.

Reference: docs/blockchain-architecture.md aur docs/blockchain-phases-detailed.md padho pehle — Phase 1 section follow karo.

Repo structure banao:
chain-core/
  crates/types/       - Block, Transaction, Account structs
  crates/crypto/       - ed25519 keypair gen, signing/verification, blake3 hashing
  crates/storage/      - RocksDB wrapper, simple key-value state (balances)
  crates/node/          - main.rs binary
  config/genesis.json

Requirements:
1. Block struct: header (prev_hash, state_root, tx_root, height, timestamp), transactions[]
2. Transaction struct: nonce, from, to, value, signature — ed25519-dalek use karo signing ke liye, blake3 hashing ke liye
3. Genesis block config file se load ho (initial balances)
4. Node binary: loop mein har N seconds pending txs se naya block banaye, RocksDB mein state persist kare
5. CLI commands: create-account, send-tx, get-balance, get-block
6. Restart pe state RocksDB se wapas load ho

Standard, audited crates hi use karna crypto ke liye (ed25519-dalek, blake3) — koi custom crypto implementation mat likhna.

Unit tests likho: tx signing/verification, block hashing, state persistence.

Production-ready code chahiye — proper error handling (Result types, no unwrap() in non-test code), clear module separation, comments jahan logic non-obvious ho.
```

---

## PHASE 2 PROMPT — PoS Consensus

```
Phase 1 complete hai (single-node prototype working). Ab Phase 2: PoS consensus.

Reference: docs/blockchain-phases-detailed.md ka Phase 2 section follow karo.

chain-core/crates/consensus/ banao:
- validator_set.rs — validator registry with stake amounts
- proposer.rs — stake-weighted round-robin proposer selection
- voting.rs — Tendermint-style BFT voting (pre-vote, pre-commit phases)
- slashing.rs — double-sign detection + stake penalty logic

Requirements:
1. Validator set: struct with validator pubkey + stake amount, add/remove functions
2. Proposer selection: har round mein stake-weighted random ya round-robin se proposer choose ho
3. Voting: proposer block propose kare, validators pre-vote + pre-commit karein, 2/3+ stake weighted votes pe block finalize ho
4. Slashing: agar same validator do different blocks same height pe sign kare (double-sign), detect karo aur stake katne ka logic likho (actual slashing execution Phase 4 ke execution engine se connect hoga, abhi sirf detection + logging)
5. Simulation setup: same machine pe 3-4 validator instances chala sako (separate processes ya async tasks), jo in-memory channel se communicate karein (P2P Phase 3 mein aayega)

Unit + integration tests: normal consensus round, byzantine validator scenario (galat vote bhejna), double-sign detection.

Phase 1 ke types/crypto/storage crates reuse karna, naya nahi likhna unke liye.
```

---

## PHASE 3 PROMPT — P2P Networking

```
Phase 2 complete hai (simulated multi-validator consensus working). Ab Phase 3: real P2P networking.

Reference: docs/blockchain-phases-detailed.md ka Phase 3 section follow karo.

chain-core/crates/network/ banao:
- gossip.rs — libp2p gossipsub setup for block/tx broadcast
- discovery.rs — Kademlia DHT peer discovery
- messages.rs — message enum: NewBlock, NewTx, VoteMsg, SyncRequest, SyncResponse

Requirements:
1. libp2p crate integrate karo, gossipsub protocol setup
2. Node startup pe config se bootstrap peers list load ho aur connect ho
3. Naya tx submit hone pe gossip se poore network mein broadcast ho
4. Block propose/vote messages bhi gossip se propagate hon
5. Naya node join kare toh SyncRequest bhej ke missing blocks le sake (basic chain sync)
6. Rate limiting: per-peer message rate limit, malformed message reject karo (basic DoS protection)
7. Phase 2 ka in-memory consensus simulation ko is real P2P layer se replace karo

Docker setup bhi banao (docker-compose.yml) jisse local machine pe 3-5 containerized nodes chala ke real testnet simulate kar sakoon.

Tests: 2-node message exchange, node sync from genesis, malformed message rejection.
```

---

## PHASE 4 PROMPT — EVM Compatibility

```
Phase 3 complete hai (P2P testnet working). Ab Phase 4: EVM compatibility layer.

Reference: docs/blockchain-phases-detailed.md ka Phase 4 section follow karo.

chain-core/crates/execution/ banao:
- evm.rs — revm crate integrate karo
- state_transition.rs — EVM execution ko apni state (Phase 1 ke storage crate) ke saath connect karo
- gas.rs — gas metering aur gas price logic

Requirements:
1. revm crate add karo, EVM bytecode execute karne ke liye
2. Contract deploy transactions handle karo (data field mein bytecode)
3. Contract call transactions handle karo (read aur write dono)
4. Gas calculation: insufficient gas pe transaction revert ho, gas fee proposer/validators ko credit ho
5. chain-core/crates/storage/trie.rs banao — Phase 1 ka simple KV store ko proper Merkle Patricia Trie mein upgrade karo, taaki state_root sahi se compute ho
6. RPC handlers add karo jo Ethereum-compatible JSON-RPC methods expose karein (eth_sendRawTransaction, eth_call, eth_getBalance, eth_blockNumber) taaki MetaMask/standard tooling kaam kare

Contracts folder bhi setup karo:
contracts/ — Foundry project, ek simple NativeToken.sol (ERC-20 style) likho test karne ke liye

Tests: contract deploy + call end-to-end, gas exhaustion revert, state root verification after execution.
```

---

## PHASE 5 PROMPT — Indexer + RPC Gateway + Explorer

```
Phase 4 complete hai (EVM-compatible chain working, contracts deploy ho rahe hain). Ab Phase 5: backend indexer aur explorer.

Reference: docs/blockchain-phases-detailed.md ka Phase 5 section follow karo. Backend ke liye humara existing pattern use karo: FastAPI + async SQLAlchemy + Alembic migrations (Sprax-Love backend jaisa structure).

backend-api/ banao:
- app/api/rpc_gateway.py — node ke JSON-RPC ko wrap karke FastAPI REST endpoints expose karo
- app/indexer/listener.py — node ke naye finalized blocks ko subscribe/poll karo
- app/indexer/parser.py — transactions aur contract events decode karo
- app/indexer/writer.py — parsed data Postgres mein async likho
- app/models/ — SQLAlchemy models: Block, Transaction, Account, ContractEvent
- app/websocket/live_feed.py — naye blocks/txs ka real-time WebSocket feed
- app/db/ — async session setup, Alembic migrations

explorer-frontend/ banao (React + Vite + TypeScript, Tailwind):
- pages: Home (latest blocks/txs), BlockDetail, TxDetail, AddressDetail, ContractVerify
- search bar: block number / tx hash / address se search
- live updates WebSocket se (naya block aaye toh Home page real-time update ho)

Requirements:
1. Indexer resilient ho — agar backend restart ho toh last indexed block se resume kare (missed blocks na chhoote)
2. Postgres schema mein proper indexes (tx hash, address, block number pe)
3. API pagination support kare (large result sets ke liye)
4. Explorer dark mode support kare (humare pattern ke hisaab se Tailwind dark: variants)

Tests: indexer resume-after-crash, API pagination, WebSocket live update delivery.
```

---

## PHASE 6 PROMPT — Wallet App + Faucet + Public Testnet

```
Phase 5 complete hai (explorer + backend working). Ab Phase 6: wallet app, faucet, public testnet launch.

Reference: docs/blockchain-phases-detailed.md ka Phase 6 section follow karo. Flutter app ke liye humara existing mobile app pattern follow karo.

wallet-app/ banao (Flutter):
- lib/core/keygen.dart — ed25519 keypair generation, secure device keychain mein store
- lib/core/signer.dart — transaction signing
- lib/screens/ — home (balance), send, receive (QR code), transaction history
- lib/services/rpc_client.dart — backend-api se communication (REST + WebSocket)

backend-api/app/api/faucet.py banao:
- Per-address rate limiting (e.g. 24 hours mein ek baar)
- Testnet token distribution endpoint
- Simple abuse prevention (captcha ya IP rate limit)

Deployment prep:
- Validator nodes ke liye Docker images finalize karo (Phase 3 ke docker-compose ko production-ready banao)
- Cloud deployment scripts (systemd service files ya k8s manifests — jo bhi hum use karna decide karein)
- Public RPC endpoint ke liye nginx reverse proxy config with rate limiting

Requirements:
1. Wallet app private keys kabhi bhi plaintext store na kare — Flutter secure storage use karo
2. Faucet abuse-resistant ho
3. Basic user documentation likho (README) — wallet setup, faucet use, testnet connect karne ka process

Tests: wallet key generation + signing correctness, faucet rate limit enforcement.
```

---

## PHASE 7 — Security Audit + Bug Bounty + Mainnet Prep 

```
Ye phase code-generation task nahi hai — ye external process hai:

1. External audit firm hire karo (Trail of Bits, OpenZeppelin, Halborn, ya similar) — poore chain-core/, contracts/ ka professional audit
2. Fuzz testing setup karo (cargo-fuzz) — Claude Code se fuzz test harnesses likhwa sakte ho, lekin actual fuzzing run + result analysis manual hai
3. Bug bounty program launch karo (Immunefi ya similar platform) testnet pe
4. Validator key management ko HSM/secure enclave pe migrate karo
5. Load testing + chaos testing (network partition, validator downtime simulate karo)
6. Genesis ceremony plan document karo
7. Incident response plan likho (chain halt mechanism, emergency upgrade path)

Claude Code se help le sakte ho:
- Fuzz test harness code likhne mein
- Load testing scripts likhne mein
- Documentation (incident response plan, genesis ceremony doc) draft karne mein

Lekin audit khud aur bug bounty triage insaan (security expert) hi kare — ye critical hai.
```

---

## General Rules for Every Phase (Gemini Code ko yaad dilana)

- Har phase start karne se pehle purane phase ka test suite pass hona chahiye
- Consensus aur crypto code ko manually line-by-line review karo — sirf AI-generated hone ki wajah se trust mat karo
- Koi custom cryptography implementation nahi — sirf audited crates
- Har phase ke end mein: `docs/` folder mein us phase ka actual implementation notes add karwao (jo build hua, kya deviate hua plan se, kyun)
- Zip-based delivery chahiye toh Claude Code se bolo: "is phase ka complete code ek zip mein package karo /mnt/user-data/outputs mein" (agar Claude Code environment mein ye supported ho)