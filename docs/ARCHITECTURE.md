# SPRX Protocol — Master System Architecture

```mermaid
flowchart TB
    subgraph Clients["Presentation & Client Layer"]
        WW["Web Wallet (React + Vite)"]
        MW["Mobile Wallet (Flutter / Dart)"]
        EXP["Explorer UI (React + Tailwind)"]
        ADM["Admin Panel (React)"]
        CLI["SPRX CLI (Rust)"]
    end

    subgraph Backend["API & Ingestion Layer"]
        API["FastAPI Core (Port 8000)"]
        IDX["Indexer Engine (Python Celery)"]
        PG[(PostgreSQL Primary)]
        RD[(Redis Cache & Queue)]
    end

    subgraph Node["Chain Core Runtime (crates/sprax-node)"]
        RPC["JSON-RPC 2.0 & REST (Port 26657)"]
        MEM["Transaction Mempool"]
        LEDGER["ChainLedger & State Executor"]
        CONS["BftConsensusEngine & Driver"]
        P2P["P2P Gossip Network (Port 26656)"]
        STORE[(Embedded redb / State Store)]
    end

    WW -->|REST / JSON-RPC| RPC
    MW -->|JSON-RPC| RPC
    CLI -->|CLI Commands| LEDGER
    EXP -->|REST API| API
    ADM -->|JWT API| API
    API -->|JSON-RPC 2.0| RPC
    IDX -->|Poll Blocks / Events| RPC
    API --> PG
    API --> RD
    IDX --> PG
    RPC --> MEM
    RPC --> LEDGER
    MEM --> LEDGER
    CONS --> LEDGER
    P2P --> CONS
    LEDGER --> STORE
```

## System Pillars

1. **Modular Crate Separation**: Clear boundaries between types, crypto, storage, consensus, ledger execution, and network transport.
2. **Deterministic BFT-PoS Consensus**: 2-step Prevote & Precommit voting with Deterministic Weighted Round Robin (DWRR) proposer election.
3. **Dual Cryptographic Suite**: Ed25519 for validators & high-speed accounts; Secp256k1 for hardware wallets and cross-chain bridging.
4. **Decoupled Client & Presentation Layer**: Wallets sign transactions offline; backend and RPC nodes never receive private keys.
5. **Real-time Fiat Abstraction**: Native balances are strictly maintained in 18-decimal atto-SPRX, with decoupled client-side currency valuation (USD, INR, EUR, JPY).
