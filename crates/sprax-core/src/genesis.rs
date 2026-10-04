use crate::{
    error::CoreError,
    state::{AccountState, StateAccessor, SupplyState},
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

/// Default block reward minted to the proposer at height 1, before any halving. Used as the
/// serde default so genesis files persisted before this field existed still deserialize.
fn default_initial_block_reward() -> Amount {
    Amount::from_sprx_whole(2).unwrap()
}

/// Default halving cadence: ~4 years at the standard 1500ms block-time target.
fn default_halving_interval_blocks() -> u64 {
    84_096_000
}

/// Default hard cap on total circulating supply.
fn default_max_supply() -> Amount {
    Amount::from_sprx_whole(1_500_000_000).unwrap()
}

/// Default unbonding period, matching `sprax-consensus`'s `StakingParams::default()` devnet value.
fn default_unbonding_period_blocks() -> u64 {
    10
}

/// Network Consensus and Execution Parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusParams {
    pub block_time_target_ms: u64,
    pub max_block_gas: u64,
    pub max_block_size_bytes: usize,
    pub min_gas_price_atto: u128,
    /// Block reward minted to the proposer at height 1, before any halving.
    #[serde(default = "default_initial_block_reward")]
    pub initial_block_reward: Amount,
    /// Number of blocks between each halving of `initial_block_reward` (Bitcoin-style).
    #[serde(default = "default_halving_interval_blocks")]
    pub halving_interval_blocks: u64,
    /// Hard cap on total circulating supply; minting stops once reached.
    #[serde(default = "default_max_supply")]
    pub max_supply: Amount,

    #[serde(default = "default_unbonding_period_blocks")]
    pub unbonding_period_blocks: u64,
}

impl Default for ConsensusParams {
    fn default() -> Self {
        Self {
            block_time_target_ms: 1500,
            max_block_gas: 20_000_000,
            max_block_size_bytes: 4 * 1024 * 1024,
            min_gas_price_atto: 1_000_000_000, // 1 nano-SPRX
            initial_block_reward: default_initial_block_reward(),
            halving_interval_blocks: default_halving_interval_blocks(),
            max_supply: default_max_supply(),
            unbonding_period_blocks: default_unbonding_period_blocks(),
        }
    }
}

impl ConsensusParams {
    /// Computes the block reward at a given height, halving every `halving_interval_blocks`.
    /// Returns zero once enough halvings have elapsed to exhaust the reward (u128 shift-safe).
    #[must_use]
    pub fn block_reward_at_height(&self, height: u64) -> Amount {
        let halvings = height / self.halving_interval_blocks;
        let halvings_u32 = u32::try_from(halvings).unwrap_or(u32::MAX);
        Amount::from_atto(
            self.initial_block_reward
                .as_atto()
                .checked_shr(halvings_u32)
                .unwrap_or(0),
        )
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
    pub const IDENTITY_KEY: &'static [u8] = b"chain/genesis-identity/v1";

    pub fn fingerprint(&self) -> Result<Hash32, CoreError> {
        let bytes = serde_json::to_vec(self).map_err(|e| CoreError::StateError(e.to_string()))?;
        Ok(Hasher::sha256(&bytes))
    }

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

            validators: vec![],
        }
    }

    pub fn mainnet_genesis() -> Self {
        Self {
            chain_id: "sprax-mainnet-1".to_string(),
            genesis_time_unix_secs: 1_735_689_600, // 2025-01-01T00:00:00Z
            initial_height: 0,
            consensus_params: ConsensusParams::default(),
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
        sprax_types::ChainId::new(&self.chain_id)
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        if self.initial_height != 0
            || self.consensus_params.max_block_gas == 0
            || self.consensus_params.max_block_size_bytes == 0
            || self.consensus_params.halving_interval_blocks == 0
            || self.consensus_params.block_time_target_ms == 0
        {
            return Err(CoreError::StateError(
                "unsupported genesis height or zero block limits".into(),
            ));
        }
        let mut addresses = std::collections::HashSet::new();
        for account in &self.accounts {
            if !addresses.insert(account.address) {
                return Err(CoreError::StateError(
                    "duplicate genesis account address".into(),
                ));
            }
        }
        let supply = self
            .accounts
            .iter()
            .try_fold(Amount::ZERO, |sum, account| {
                sum.checked_add(account.initial_balance)
            })
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        if supply > self.consensus_params.max_supply {
            return Err(CoreError::StateError(
                "genesis allocations exceed maximum supply".into(),
            ));
        }
        let mut operators = std::collections::HashSet::new();
        let mut consensus_keys = std::collections::HashSet::new();
        for validator in &self.validators {
            if !operators.insert(validator.operator_address)
                || !consensus_keys.insert(validator.consensus_pubkey.clone())
                || validator.consensus_pubkey.len() != 32
                || validator.self_stake == Amount::ZERO
            {
                return Err(CoreError::StateError(
                    "invalid or duplicate genesis validator".into(),
                ));
            }
            let allocation = self
                .accounts
                .iter()
                .find(|account| account.address == validator.operator_address);
            if allocation.is_none_or(|account| account.initial_balance < validator.self_stake) {
                return Err(CoreError::StateError(
                    "genesis self-stake must be funded by the operator allocation".into(),
                ));
            }
        }
        store
            .set(Self::IDENTITY_KEY, self.fingerprint()?.as_bytes())
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        let mut circulating_supply = Amount::ZERO;
        for acc in &self.accounts {
            let state = AccountState {
                nonce: 0,
                balance: acc.initial_balance,
                code_hash: Hash32::ZERO,
                storage_root: Hash32::ZERO,
            };
            StateAccessor::set_account(store, &acc.address, &state)?;
            circulating_supply = circulating_supply
                .checked_add(acc.initial_balance)
                .map_err(|e| CoreError::StateError(e.to_string()))?;
        }
        StateAccessor::set_supply_state(
            store,
            &SupplyState {
                circulating_supply,
                total_burned: Amount::ZERO,
            },
        )?;

        // Seed each genesis validator's canonical stake so `sprax-node`'s per-height sync
        // (ConsensusDriver reading this back into `StakingKeeper`'s BFT cache) starts from the
        // same self-stake the validator was registered with, instead of zeroing it out at the
        // first height.
        for val in &self.validators {
            let mut operator = StateAccessor::get_account(store, &val.operator_address)?;
            operator.balance = operator
                .balance
                .checked_sub(val.self_stake)
                .map_err(|e| CoreError::StateError(e.to_string()))?;
            StateAccessor::set_account(store, &val.operator_address, &operator)?;
            StateAccessor::set_delegation(
                store,
                &val.operator_address,
                &val.operator_address,
                &crate::state::DelegationState {
                    shares: val.self_stake,
                    balance: val.self_stake,
                },
            )?;
            StateAccessor::set_validator_stake(
                store,
                &val.operator_address,
                &crate::state::ValidatorStakeState {
                    tokens: val.self_stake,
                },
            )?;
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
            validator_set_hash: Hasher::sha256(
                &serde_json::to_vec(&self.validators)
                    .map_err(|e| CoreError::StateError(e.to_string()))?,
            ),
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

    #[test]
    fn test_initialize_state_seeds_circulating_supply() {
        let genesis = GenesisConfig::default_development();
        let store = MemKVStore::new();
        genesis.initialize_state(&store).unwrap();

        let expected: Amount = genesis.accounts.iter().fold(Amount::ZERO, |acc, a| {
            acc.checked_add(a.initial_balance).unwrap()
        });

        let supply = StateAccessor::get_supply_state(&store).unwrap();
        assert_eq!(supply.circulating_supply, expected);
        assert_eq!(supply.total_burned, Amount::ZERO);
    }

    #[test]
    fn test_block_reward_halving_schedule() {
        let params = ConsensusParams {
            initial_block_reward: Amount::from_sprx_whole(8).unwrap(),
            halving_interval_blocks: 100,
            ..ConsensusParams::default()
        };

        assert_eq!(
            params.block_reward_at_height(1),
            Amount::from_sprx_whole(8).unwrap()
        );
        assert_eq!(
            params.block_reward_at_height(99),
            Amount::from_sprx_whole(8).unwrap()
        );
        assert_eq!(
            params.block_reward_at_height(100),
            Amount::from_sprx_whole(4).unwrap()
        );
        assert_eq!(
            params.block_reward_at_height(200),
            Amount::from_sprx_whole(2).unwrap()
        );
        // Far enough out that the reward has shifted past zero entirely.
        assert_eq!(params.block_reward_at_height(100 * 200), Amount::ZERO);
    }
}
