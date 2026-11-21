use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use sprax_storage::{BatchOperation, KVStore, ReadonlyKVStore};
use sprax_types::{Address, Amount, Hash32};

pub use sprax_types::{AccountState, SupplyState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalValidatorStatus {
    Active,
    Jailed,
    Tombstoned,
}

/// Authoritative persisted validator record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorRecordState {
    pub operator_address: Address,
    pub consensus_pubkey: Vec<u8>,
    pub moniker: String,
    pub status: CanonicalValidatorStatus,
    pub jailed_until: Option<u64>,
    pub is_tombstoned: bool,
    pub registered_height: u64,
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

    #[must_use]
    pub fn validator_record_key(operator: &Address) -> Vec<u8> {
        let mut k = Vec::with_capacity(1 + 20);
        k.push(b'r'); // 'r' prefix for validator record
        k.extend_from_slice(operator.as_bytes());
        k
    }

    #[must_use]
    pub fn consensus_key_index(pubkey: &[u8]) -> Vec<u8> {
        let mut k = Vec::with_capacity(1 + pubkey.len());
        k.push(b'k'); // 'k' prefix for consensus pubkey uniqueness index
        k.extend_from_slice(pubkey);
        k
    }

    #[must_use]
    pub fn evidence_processed_key(hash: &Hash32) -> Vec<u8> {
        let mut k = Vec::with_capacity(1 + 32);
        k.push(b'e'); // 'e' prefix for finalized evidence processed marker
        k.extend_from_slice(hash.as_bytes());
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

    pub fn get_validator_record(
        store: &impl ReadonlyKVStore,
        operator: &Address,
    ) -> Result<Option<ValidatorRecordState>, CoreError> {
        let key = Self::validator_record_key(operator);
        match store
            .get(&key)
            .map_err(|e| CoreError::StateError(e.to_string()))?
        {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|e| CoreError::StateError(format!("deserialization error: {e}"))),
            None => Ok(None),
        }
    }

    pub fn set_validator_record(
        store: &impl KVStore,
        record: &ValidatorRecordState,
    ) -> Result<(), CoreError> {
        let key = Self::validator_record_key(&record.operator_address);
        let bytes = serde_json::to_vec(record)
            .map_err(|e| CoreError::StateError(format!("serialization error: {e}")))?;
        store
            .set(&key, &bytes)
            .map_err(|e| CoreError::StateError(e.to_string()))
    }

    pub fn get_operator_by_consensus_key(
        store: &impl ReadonlyKVStore,
        pubkey: &[u8],
    ) -> Result<Option<Address>, CoreError> {
        let key = Self::consensus_key_index(pubkey);
        match store
            .get(&key)
            .map_err(|e| CoreError::StateError(e.to_string()))?
        {
            Some(bytes) => {
                if bytes.len() == 20 {
                    let mut arr = [0u8; 20];
                    arr.copy_from_slice(&bytes);
                    Ok(Some(Address::new(arr)))
                } else {
                    Err(CoreError::StateError(
                        "invalid operator address in consensus key index".into(),
                    ))
                }
            }
            None => Ok(None),
        }
    }

    pub fn set_consensus_key_index(
        store: &impl KVStore,
        pubkey: &[u8],
        operator: &Address,
    ) -> Result<(), CoreError> {
        let key = Self::consensus_key_index(pubkey);
        store
            .set(&key, operator.as_bytes())
            .map_err(|e| CoreError::StateError(e.to_string()))
    }

    pub fn has_evidence(store: &impl ReadonlyKVStore, hash: &Hash32) -> Result<bool, CoreError> {
        let key = Self::evidence_processed_key(hash);
        Ok(store
            .get(&key)
            .map_err(|e| CoreError::StateError(e.to_string()))?
            .is_some())
    }

    pub fn record_evidence(
        store: &impl KVStore,
        hash: &Hash32,
        height: u64,
    ) -> Result<(), CoreError> {
        let key = Self::evidence_processed_key(hash);
        store
            .set(&key, &height.to_be_bytes())
            .map_err(|e| CoreError::StateError(e.to_string()))
    }

    pub fn scan_all_validators(
        store: &impl ReadonlyKVStore,
    ) -> Result<Vec<ValidatorRecordState>, CoreError> {
        let raw = store
            .scan_prefix(b"r")
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        raw.into_iter()
            .map(|(_, bytes)| {
                serde_json::from_slice(&bytes)
                    .map_err(|e| CoreError::StateError(format!("deserialization error: {e}")))
            })
            .collect()
    }

    pub fn scan_unbondings_for_validator(
        store: &impl ReadonlyKVStore,
        validator: &Address,
    ) -> Result<Vec<(Vec<u8>, UnbondingRecord)>, CoreError> {
        let raw = store
            .scan_prefix(b"u")
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        let mut matching = Vec::new();
        for (key, bytes) in raw {
            let record: UnbondingRecord = serde_json::from_slice(&bytes)
                .map_err(|e| CoreError::StateError(format!("deserialization error: {e}")))?;
            if record.validator == *validator {
                matching.push((key, record));
            }
        }
        Ok(matching)
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
