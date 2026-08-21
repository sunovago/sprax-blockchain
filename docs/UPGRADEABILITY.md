# SPRX Protocol — Upgradeability & State Migration

## In-Band Consensus Upgrades
Network software upgrades are coordinated via on-chain governance proposals specifying an exact target block height $H_{upgrade}$.

### Upgrade Flow
1. **Governance Approval**: A software upgrade proposal passes voting with target height $H_{upgrade}$ and binary release hash.
2. **Height Halt**: When the node reaches $H_{upgrade}$, the consensus driver halts block production and writes an upgrade marker to disk.
3. **Binary Replacement**: Node operators update their node binary to the approved version.
4. **State Migration**: On startup, the new binary runs automated schema and state migration hooks before resuming BFT consensus.

---

## Smart Contract Upgrades
- Smart contracts deployed to `sprax-wasm` can specify an optional `admin` address upon instantiation.
- The `admin` can migrate the contract to a new `code_id` via a migration message, preserving existing contract storage while updating execution logic.
