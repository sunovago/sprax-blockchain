# SPRX Protocol: Upgradeability & State Migration Architecture
**Document Version:** 1.0.0-PROPOSAL  
**Status:** ARCHITECTURE_ONLY (Phase 01)  
**Scope:** Protocol-Level Upgrades, State Migrations & Smart Contract Proxies

---

## 1. Protocol Upgrade Philosophy

In distributed blockchain networks, software upgrades present severe coordination and state-consistency risks. SPRX implements a **Deterministic In-Band Coordinated Upgrade Architecture** that ensures seamless protocol evolution, zero chain splits, and automated state migrations at deterministic block heights.

```
+---------------------------------------------------------------------------------------------------+
|                                  SPRX UPGRADE ARCHITECTURE                                        |
+---------------------------------------------------------------------------------------------------+
|                                                                                                   |
| [Layer 1: Protocol & Consensus Layer]                                                             |
|  - On-chain height-coordinated software upgrades (e.g., Cosmovisor / SPRX Daemon Manager)       |
|  - Automated state migration handlers executed within consensus execution boundary                |
|  - Zero-downtime binary hot-swapping or coordinated stop/start at height H_upgrade                |
|                                                                                                   |
| [Layer 2: Smart Contract & Application Layer]                                                     |
|  - Standardized Upgradeable Proxies: ERC-1967 Transparent Proxy, UUPS, Diamond Pattern (ERC-2535) |
|  - ERC-7201 Namespaced Storage Layouts to eliminate storage slot clashing                         |
|  - Immutable Core + Pluggable Logic Modules for enterprise dApps                                 |
|                                                                                                   |
| [Layer 3: Network & Interface Layer]                                                              |
|  - P2P Protocol Version Negotiation (Disconnect incompatible wire versions)                       |
|  - Deprecation lifecycle for RPC APIs with guaranteed 12-month backward compatibility             |
|                                                                                                   |
+---------------------------------------------------------------------------------------------------+
```

---

## 2. In-Band Coordinated Protocol Upgrades

```
+---------------------------------------------------------------------------------------------------+
|                                 DETERMINISTIC UPGRADE EXECUTION                                   |
+---------------------------------------------------------------------------------------------------+
| Height H_upgrade - 1                                                                              |
|      |                                                                                            |
|      v                                                                                            |
| [1. Finalize Block H_upgrade - 1] ----> Commit State & Receipts to Disk                           |
|      |                                                                                            |
|      v                                                                                            |
| [2. Trigger Upgrade Plan] ------------> Identify Registered Handler for Plan "v2.0.0-SPRX"        |
|      |                                                                                            |
|      v                                                                                            |
| [3. Execute State Migration Hook] ----> Mutate Storage Schemas / Backfill State Defaults          |
|      |                                                                                            |
|      v                                                                                            |
| [4. Compute Migrated StateRoot] ------> Verify Post-Migration Root Invariant Across Validators   |
|      |                                                                                            |
|      v                                                                                            |
| [5. Resume Consensus at H_upgrade] ---> Block Production Resumes Seamlessly under New Rules       |
+---------------------------------------------------------------------------------------------------+
```

### 2.1 Upgrade Handler Interface & Lifecycle
Every protocol upgrade defines an isolated migration handler in the node runtime:

```rust
pub trait UpgradeHandler {
    /// Unique identifier of the upgrade plan approved via governance
    fn plan_name(&self) -> &'static str;
    
    /// Target block height at which migration executes
    fn upgrade_height(&self) -> u64;
    
    /// Deterministic state migration logic
    fn execute_migration(&self, ctx: &mut MigrationContext) -> Result<StateRoot, MigrationError>;
}
```

### 2.2 Daemon Supervisor (SPRX Node Manager)
To ensure zero manual intervention by node operators during scheduled upgrades:
1. The daemon manager downloads and cryptographically verifies the new binary (`sprx-node-v2.0.0`) matching the governance-approved SHA-256 hash.
2. At height $H_{upgrade}-1$, the old binary halts cleanly.
3. The supervisor hot-swaps the binary symlink and restarts the node with `sprx-node-v2.0.0`.
4. The new binary executes `execute_migration()`, confirms the new `StateRoot`, and joins the consensus round for block $H_{upgrade}$.

---

## 3. Fork Classification & Upgrade Taxonomy

```
+---------------------------------------------------------------------------------------------------+
|                                  UPGRADE TAXONOMY MATRIX                                          |
+---------------------------------------------------------------------------------------------------+
| Upgrade Type        | Trigger Mechanism           | State Migration Required | Node Action        |
+---------------------+-----------------------------+--------------------------+--------------------+
| Dynamic Parameter   | Governance Proposal Vote    | No (In-memory update)    | Zero Restart       |
| In-Band Coordinate  | Height-Triggered Governance | Yes (Automated Handler)  | Auto Hot-Swap      |
| Emergency Soft-Fork | Validator Consensus Patch   | No (Rule restriction)    | Optional Restart   |
| Emergency Hard-Fork | Social Consensus Directive  | Manual Snapshot Restore  | Manual Bin Replace |
+---------------------+-----------------------------+--------------------------+--------------------+
```

---

## 4. Smart Contract Upgrade Patterns

For on-chain smart contracts (DeFi protocols, Real-World Asset registries, Token vaults), SPRX provides native tooling and standards for safe upgradeability:

```
+---------------------------------------------------------------------------------------------------+
|                                  SMART CONTRACT UPGRADE PATTERNS                                  |
+---------------------------------------------------------------------------------------------------+
|                                                                                                   |
|  [Pattern 1: UUPS (Universal Upgradeable Proxy)]                                                  |
|   Caller ---> [Proxy Contract (Holds State & Balance)] ---delegatecall---> [Logic Implementation V1]|
|                                                                         | (Upgrade to V2)         |
|                                                                         +--> [Logic Implementation V2]|
|                                                                                                   |
|  [Pattern 2: Diamond Multi-Facet Proxy (ERC-2535)]                                                |
|   Caller ---> [Diamond Proxy (Router)] ---delegatecall---> [Facet A: Staking Logic]               |
|                                        ---delegatecall---> [Facet B: Governance Logic]            |
|                                        ---delegatecall---> [Facet C: Token Economics]             |
|                                                                                                   |
+---------------------------------------------------------------------------------------------------+
```

### 4.1 Storage Collision Prevention: Namespaced Storage (ERC-7201)
To eliminate catastrophic storage slot clashing across proxy upgrades, all smart contracts on SPRX must utilize hash-derived storage namespaces:

```solidity
// Deterministic Diamond Storage Slot formula:
// bytes32(uint256(keccak256("sprx.storage.asset_registry.v1")) - 1)
bytes32 private constant ASSET_STORAGE_SLOT = 
    0x8b3a7b8e1f582c0b48c084e930f576e27a944634f19bfa7b8e1f582c0b48c084;

struct AssetStorage {
    mapping(uint256 => AssetRecord) assets;
    uint256 totalAssetsRegistered;
}

function _getAssetStorage() internal pure returns (AssetStorage storage s) {
    bytes32 slot = ASSET_STORAGE_SLOT;
    assembly { s.slot := slot }
}
```

---

## 5. API & Wire Versioning and Deprecation Policy

### 5.1 P2P Protocol Versioning
- Nodes exchange a 32-bit protocol handshake mask during connection setup.
- If a connecting peer's protocol version does not match the network's minimum active compatibility version, the connection is immediately terminated with `ErrIncompatibleProtocolVersion`.

### 5.2 JSON-RPC & gRPC API Deprecation Lifecycle
1. **Introduction**: New API version (`/v2`) introduced alongside active version (`/v1`).
2. **Deprecation Notice**: `/v1` marked deprecated; header response includes `Warning: 299 - "SPRX RPC v1 is deprecated and will sunset on block H_sunset"`.
3. **12-Month Grace Period**: `/v1` endpoints maintained with full backward compatibility for a minimum of 12 calendar months.
4. **Sunset**: `/v1` routes return `410 Gone` with redirection payload to `/v2`.
