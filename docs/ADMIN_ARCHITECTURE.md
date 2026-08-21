# SPRX Ecosystem: Centralized Admin Operations Architecture
**Document Version:** 1.0.0  
**Target:** Infrastructure, SRE, Ops, Security

---

## 1. Unified Architecture Topology

```mermaid
graph TD
    subgraph SpraxChain["Sprax Chain Ledger"]
        Node1["Validator Node 1 (RPC)"]
        Node2["Validator Node 2"]
        Node3["Validator Node 3"]
    end

    subgraph IndexerEngine["SPRX Indexer"]
        SyncWorker["Async Block Catchup Worker"]
        EventStream["Block Event Hub"]
    end

    subgraph StorageLayer["Data Tier"]
        Postgres[(PostgreSQL Primary)]
        RedisCache[(Redis Cache & PubSub)]
    end

    subgraph UnifiedBackend["FastAPI Unified Backend"]
        PublicAPI["/api/v1/explorer, /api/v1/markets"]
        AdminAPI["/api/v1/admin/* (Protected by RBAC & MFA)"]
        WSHub["WebSocket Realtime Engine"]
    end

    subgraph Clients["Ecosystem Frontends"]
        FlutterApp["Flutter Mobile Wallet"]
        ExplorerUI["Sprax Explorer Web UI"]
        AdminPanel["SPRX Admin Operations Panel"]
    end

    Node1 --> SyncWorker
    SyncWorker --> Postgres
    Node1 --> PublicAPI
    Postgres --> PublicAPI
    Postgres --> AdminAPI
    RedisCache --> PublicAPI
    RedisCache --> AdminAPI

    PublicAPI --> FlutterApp
    PublicAPI --> ExplorerUI
    AdminAPI --> AdminPanel
    WSHub --> AdminPanel
```

---

## 2. Invariants & Isolation
1. **Zero Browser Direct Connections**: Browser frontends NEVER connect directly to PostgreSQL, Redis, or raw Tendermint node admin RPC ports.
2. **Single Source of Truth**: Admin Panel queries the unified backend which reflects PostgreSQL indexer tables and RPC node state.
3. **Strict Non-Custodial Isolation**: No private keys or wallet mnemonics are accessible or stored on backend databases.
