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
