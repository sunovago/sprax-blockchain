use crate::{
    error::CoreError,
    state::{AccountState, StateAccessor},
};
use serde::{Deserialize, Serialize};
use sprax_crypto::Hasher;
use sprax_storage::{KVStore, StateCommitment};
use sprax_types::{Address, Amount, BlockHeader, Hash32};

/// Genesis Account allocation specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisAccount {
    pub name: String,
    pub address: Address,
    pub initial_balance: Amount,
}

/// Genesis validator registration: seeded into `StakingKeeper` on first node startup so a
/// devnet has an active BFT validator set from height 0 with zero extra config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisValidator {
    pub operator_address: Address,
    pub consensus_pubkey: Vec<u8>,
    pub self_stake: Amount,
    pub moniker: String,
}

/// Network Consensus and Execution Parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusParams {
    pub block_time_target_ms: u64,
    pub max_block_gas: u64,
    pub max_block_size_bytes: usize,
    pub min_gas_price_atto: u128,
}

impl Default for ConsensusParams {
    fn default() -> Self {
        Self {
            block_time_target_ms: 1500,
            max_block_gas: 20_000_000,
            max_block_size_bytes: 4 * 1024 * 1024,
            min_gas_price_atto: 1_000_000_000, // 1 nano-SPRX
        }
    }
}

/// Master Genesis Specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub chain_id: String,
    pub genesis_time_unix_secs: u64,
    pub initial_height: u64,
    pub consensus_params: ConsensusParams,
    pub accounts: Vec<GenesisAccount>,
    #[serde(default)]
    pub validators: Vec<GenesisValidator>,
}

impl GenesisConfig {
    /// Creates a deterministic local development genesis with pre-funded accounts.
    /// Default pre-funded accounts:
    /// - Alice:   1,000,000.00 SPRX
    /// - Bob:       500,000.00 SPRX
    /// - Charlie:   100,000.00 SPRX
    pub fn default_development() -> Self {
        let alice_addr = Address::new([1u8; 20]);
        let bob_addr = Address::new([2u8; 20]);
        let charlie_addr = Address::new([3u8; 20]);

        Self {
            chain_id: "sprax-devnet-1".to_string(),
            genesis_time_unix_secs: 1_700_000_000,
            initial_height: 0,
            consensus_params: ConsensusParams::default(),
            accounts: vec![
                GenesisAccount {
                    name: "alice".to_string(),
                    address: alice_addr,
                    initial_balance: Amount::from_sprx_whole(1_000_000).unwrap(),
                },
                GenesisAccount {
                    name: "bob".to_string(),
                    address: bob_addr,
                    initial_balance: Amount::from_sprx_whole(500_000).unwrap(),
                },
                GenesisAccount {
                    name: "charlie".to_string(),
                    address: charlie_addr,
                    initial_balance: Amount::from_sprx_whole(100_000).unwrap(),
                },
            ],
            // Empty here deliberately: these test accounts use fixed byte-pattern addresses
            // ([1u8;20] etc.), not addresses derived from a real signing keypair, so they
            // cannot be registered as validators (vote signatures wouldn't verify against
            // them). `sprax-node::NodeService` builds its own dev genesis with real
            // keypair-derived addresses and registers matching `GenesisValidator` entries there.
            validators: vec![],
        }
    }

    /// Creates the formal production Mainnet Genesis Specification.
    /// Total Initial Supply: 1,000,000,000.00 SPRX (1 Billion SPRX).
    /// Allocation breakdown:
    /// - Community Pool: 400,000,000 SPRX (40%)
    /// - Ecosystem & Grants: 250,000,000 SPRX (25%)
    /// - Treasury Reserve: 150,000,000 SPRX (15%)
    /// - Validator Genesis Incentives: 100,000,000 SPRX (10%)
    /// - Core Contributors (Vesting): 100,000,000 SPRX (10%)
    pub fn mainnet_genesis() -> Self {
        Self {
            chain_id: "sprax-mainnet-1".to_string(),
            genesis_time_unix_secs: 1_735_689_600, // 2025-01-01T00:00:00Z
            initial_height: 0,
            consensus_params: ConsensusParams {
                block_time_target_ms: 1500,
                max_block_gas: 20_000_000,
                max_block_size_bytes: 4 * 1024 * 1024,
                min_gas_price_atto: 1_000_000_000, // 1 nano-SPRX
            },
            accounts: vec![
                GenesisAccount {
                    name: "community_pool".to_string(),
                    address: Address::new([0x10; 20]),
                    initial_balance: Amount::from_sprx_whole(400_000_000).unwrap(),
                },
                GenesisAccount {
                    name: "ecosystem_grants".to_string(),
                    address: Address::new([0x20; 20]),
                    initial_balance: Amount::from_sprx_whole(250_000_000).unwrap(),
                },
                GenesisAccount {
                    name: "treasury_reserve".to_string(),
                    address: Address::new([0x30; 20]),
                    initial_balance: Amount::from_sprx_whole(150_000_000).unwrap(),
                },
                GenesisAccount {
                    name: "validator_incentives".to_string(),
                    address: Address::new([0x40; 20]),
                    initial_balance: Amount::from_sprx_whole(100_000_000).unwrap(),
                },
                GenesisAccount {
                    name: "core_contributors_vesting".to_string(),
                    address: Address::new([0x50; 20]),
                    initial_balance: Amount::from_sprx_whole(100_000_000).unwrap(),
                },
            ],
            // Mainnet validator set is not yet defined — tracked as a later milestone (genesis
            // ceremony, per docs/Blockchain.md Phase 7).
            validators: vec![],
        }
    }

    /// Initializes state store with genesis account balances and returns initial state root and BlockHeader.
    pub fn initialize_state<S: KVStore + StateCommitment>(
        &self,
        store: &S,
    ) -> Result<BlockHeader, CoreError> {
        for acc in &self.accounts {
            let state = AccountState {
                nonce: 0,
                balance: acc.initial_balance,
                code_hash: Hash32::ZERO,
                storage_root: Hash32::ZERO,
            };
            StateAccessor::set_account(store, &acc.address, &state)?;
        }

        let state_root = store
            .compute_root()
            .map_err(|e| CoreError::StateError(e.to_string()))?;

        let header = BlockHeader {
            version: 1,
            chain_id: self.chain_id.clone(),
            height: 0,
            timestamp_unix_secs: self.genesis_time_unix_secs,
            parent_hash: Hash32::ZERO,
            proposer: Address::ZERO,
            state_root,
            txs_root: Hash32::ZERO,
            receipts_root: Hash32::ZERO,
            validator_set_hash: Hasher::blake3(b"genesis-validators"),
        };

        Ok(header)
    }

    /// Saves genesis JSON to file.
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), CoreError> {
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| CoreError::StateError(format!("json encode error: {e}")))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CoreError::StateError(format!("fs error: {e}")))?;
        }
        std::fs::write(path, json_str)
            .map_err(|e| CoreError::StateError(format!("fs write error: {e}")))?;
        Ok(())
    }

    /// Loads genesis configuration from file.
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, CoreError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| CoreError::StateError(format!("fs read error: {e}")))?;
        serde_json::from_str(&content)
            .map_err(|e| CoreError::StateError(format!("json parse error: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sprax_storage::MemKVStore;

    #[test]
    fn test_genesis_state_initialization() {
        let genesis = GenesisConfig::default_development();
        let store = MemKVStore::new();
        let header = genesis.initialize_state(&store).unwrap();

        assert_eq!(header.height, 0);
        assert_eq!(header.chain_id, "sprax-devnet-1");
        assert_ne!(header.state_root, Hash32::ZERO);

        let alice_addr = Address::new([1u8; 20]);
        let alice_state = StateAccessor::get_account(&store, &alice_addr).unwrap();
        assert_eq!(
            alice_state.balance,
            Amount::from_sprx_whole(1_000_000).unwrap()
        );
        assert_eq!(alice_state.nonce, 0);
    }

    #[test]
    fn test_mainnet_genesis_1_billion_supply_conservation() {
        let genesis = GenesisConfig::mainnet_genesis();
        assert_eq!(genesis.chain_id, "sprax-mainnet-1");

        let mut total_atto: u128 = 0;
        for acc in &genesis.accounts {
            total_atto += acc.initial_balance.as_atto();
        }

        let expected_1_billion_atto = 1_000_000_000 * 1_000_000_000_000_000_000u128;
        assert_eq!(total_atto, expected_1_billion_atto);
    }
}
