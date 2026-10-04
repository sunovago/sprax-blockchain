use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use sprax_storage::{BatchOperation, KVStore, ReadonlyKVStore};
use sprax_types::{Address, Amount, Hash32};

/// On-chain account state record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountState {
    pub nonce: u64,
    pub balance: Amount,
    pub code_hash: Hash32,
    pub storage_root: Hash32,
}

impl Default for AccountState {
    fn default() -> Self {
        Self {
            nonce: 0,
            balance: Amount::ZERO,
            code_hash: Hash32::ZERO,
            storage_root: Hash32::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplyState {
    /// Total issued supply, including liquid, bonded and unbonding balances (genesis + minted - burned).
    pub circulating_supply: Amount,
    /// Cumulative transaction fees burned since genesis (informational/analytics only).
    pub total_burned: Amount,
}

impl Default for SupplyState {
    fn default() -> Self {
        Self {
            circulating_supply: Amount::ZERO,
            total_burned: Amount::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorStakeState {
    pub tokens: Amount,
}

impl Default for ValidatorStakeState {
    fn default() -> Self {
        Self {
            tokens: Amount::ZERO,
        }
    }
}

/// A single delegator's bonded stake with one validator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationState {
    pub shares: Amount,
    pub balance: Amount,
}

impl Default for DelegationState {
    fn default() -> Self {
        Self {
            shares: Amount::ZERO,
            balance: Amount::ZERO,
        }
    }
}

/// A pending unbonding entry: `amount` returns to `delegator`'s account balance once
/// `completion_height` is reached (see `ChainLedger`'s matured-unbonding sweep).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnbondingRecord {
    pub delegator: Address,
    pub validator: Address,
    pub completion_height: u64,
    pub amount: Amount,
}

/// Helper for encoding and accessing account state in key-value store.
#[derive(Debug)]
pub struct StateAccessor;

impl StateAccessor {
    #[must_use]
    pub fn account_key(addr: &Address) -> Vec<u8> {
        let mut k = Vec::with_capacity(1 + 20);
        k.push(b'a'); // 'a' prefix for account
        k.extend_from_slice(addr.as_bytes());
        k
    }

    #[must_use]
    pub fn supply_key() -> Vec<u8> {
        vec![b's'] // 's' prefix for chain-wide supply state (single well-known key)
    }

    #[must_use]
    pub fn validator_stake_key(validator: &Address) -> Vec<u8> {
        let mut k = Vec::with_capacity(1 + 20);
        k.push(b'v'); // 'v' prefix for validator stake
        k.extend_from_slice(validator.as_bytes());
        k
    }

    #[must_use]
    pub fn delegation_key(delegator: &Address, validator: &Address) -> Vec<u8> {
        let mut k = Vec::with_capacity(1 + 20 + 20);
        k.push(b'd'); // 'd' prefix for delegation
        k.extend_from_slice(delegator.as_bytes());
        k.extend_from_slice(validator.as_bytes());
        k
    }

    #[must_use]
    pub fn unbonding_key(
        completion_height: u64,
        delegator: &Address,
        validator: &Address,
        tx_nonce: u64,
    ) -> Vec<u8> {
        let mut k = Vec::with_capacity(1 + 8 + 20 + 20 + 8);
        k.push(b'u'); // 'u' prefix for unbonding queue
        k.extend_from_slice(&completion_height.to_be_bytes());
        k.extend_from_slice(delegator.as_bytes());
        k.extend_from_slice(validator.as_bytes());
        k.extend_from_slice(&tx_nonce.to_be_bytes());
        k
    }

    pub fn get_account(
        store: &impl ReadonlyKVStore,
        addr: &Address,
    ) -> Result<AccountState, CoreError> {
        let key = Self::account_key(addr);
        match store
            .get(&key)
            .map_err(|e| CoreError::StateError(e.to_string()))?
        {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| CoreError::StateError(format!("deserialization error: {e}"))),
            None => Ok(AccountState::default()),
        }
    }

    pub fn set_account(
        store: &impl KVStore,
        addr: &Address,
        account: &AccountState,
    ) -> Result<(), CoreError> {
        let key = Self::account_key(addr);
        let bytes = serde_json::to_vec(account)
            .map_err(|e| CoreError::StateError(format!("serialization error: {e}")))?;
        store
            .set(&key, &bytes)
            .map_err(|e| CoreError::StateError(e.to_string()))
    }

    pub fn get_supply_state(store: &impl ReadonlyKVStore) -> Result<SupplyState, CoreError> {
        let key = Self::supply_key();
        match store
            .get(&key)
            .map_err(|e| CoreError::StateError(e.to_string()))?
        {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| CoreError::StateError(format!("deserialization error: {e}"))),
            None => Ok(SupplyState::default()),
        }
    }

    pub fn set_supply_state(store: &impl KVStore, supply: &SupplyState) -> Result<(), CoreError> {
        let key = Self::supply_key();
        let bytes = serde_json::to_vec(supply)
            .map_err(|e| CoreError::StateError(format!("serialization error: {e}")))?;
        store
            .set(&key, &bytes)
            .map_err(|e| CoreError::StateError(e.to_string()))
    }

    pub fn get_validator_stake(
        store: &impl ReadonlyKVStore,
        validator: &Address,
    ) -> Result<ValidatorStakeState, CoreError> {
        let key = Self::validator_stake_key(validator);
        match store
            .get(&key)
            .map_err(|e| CoreError::StateError(e.to_string()))?
        {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| CoreError::StateError(format!("deserialization error: {e}"))),
            None => Ok(ValidatorStakeState::default()),
        }
    }

    pub fn set_validator_stake(
        store: &impl KVStore,
        validator: &Address,
        stake: &ValidatorStakeState,
    ) -> Result<(), CoreError> {
        let key = Self::validator_stake_key(validator);
        let bytes = serde_json::to_vec(stake)
            .map_err(|e| CoreError::StateError(format!("serialization error: {e}")))?;
        store
            .set(&key, &bytes)
            .map_err(|e| CoreError::StateError(e.to_string()))
    }

    pub fn get_delegation(
        store: &impl ReadonlyKVStore,
        delegator: &Address,
        validator: &Address,
    ) -> Result<DelegationState, CoreError> {
        let key = Self::delegation_key(delegator, validator);
        match store
            .get(&key)
            .map_err(|e| CoreError::StateError(e.to_string()))?
        {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| CoreError::StateError(format!("deserialization error: {e}"))),
            None => Ok(DelegationState::default()),
        }
    }

    pub fn set_delegation(
        store: &impl KVStore,
        delegator: &Address,
        validator: &Address,
        delegation: &DelegationState,
    ) -> Result<(), CoreError> {
        let key = Self::delegation_key(delegator, validator);
        let bytes = serde_json::to_vec(delegation)
            .map_err(|e| CoreError::StateError(format!("serialization error: {e}")))?;
        store
            .set(&key, &bytes)
            .map_err(|e| CoreError::StateError(e.to_string()))
    }

    /// Queues a new unbonding entry.
    pub fn set_unbonding(
        store: &impl KVStore,
        entry: &UnbondingRecord,
        tx_nonce: u64,
    ) -> Result<(), CoreError> {
        let key = Self::unbonding_key(
            entry.completion_height,
            &entry.delegator,
            &entry.validator,
            tx_nonce,
        );
        let bytes = serde_json::to_vec(entry)
            .map_err(|e| CoreError::StateError(format!("serialization error: {e}")))?;
        store
            .set(&key, &bytes)
            .map_err(|e| CoreError::StateError(e.to_string()))
    }

    /// Returns every unbonding entry with `completion_height <= max_height_inclusive`, each
    /// paired with its raw storage key (so the caller can `KVStore::delete` it once processed).
    pub fn scan_matured_unbondings(
        store: &impl ReadonlyKVStore,
        max_height_inclusive: u64,
    ) -> Result<Vec<(Vec<u8>, UnbondingRecord)>, CoreError> {
        let start = vec![b'u'];
        let mut end = vec![b'u'];
        end.extend_from_slice(&max_height_inclusive.saturating_add(1).to_be_bytes());

        let raw = store
            .scan_range(&start, Some(&end))
            .map_err(|e| CoreError::StateError(e.to_string()))?;

        raw.into_iter()
            .map(|(key, bytes)| {
                let record: UnbondingRecord = serde_json::from_slice(&bytes)
                    .map_err(|e| CoreError::StateError(format!("deserialization error: {e}")))?;
                Ok((key, record))
            })
            .collect()
    }
}

/// Execution context for state transitions.
#[derive(Debug)]
pub struct StateTransitionContext<'a, S: KVStore> {
    pub store: &'a S,
    pub height: u64,
    pub timestamp_unix_secs: u64,
    pub chain_id: String,
    pub batch: BatchOperation,
}

impl<'a, S: KVStore> StateTransitionContext<'a, S> {
    pub fn new(
        store: &'a S,
        height: u64,
        timestamp_unix_secs: u64,
        chain_id: impl Into<String>,
    ) -> Self {
        Self {
            store,
            height,
            timestamp_unix_secs,
            chain_id: chain_id.into(),
            batch: BatchOperation::new(),
        }
    }
}
