//! Account and supply state records.
use crate::{Address, Amount, Hash32};
use serde::{Deserialize, Serialize};

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

#[must_use]
pub fn account_key(addr: &Address) -> Vec<u8> {
    let mut k = Vec::with_capacity(1 + 20);
    k.push(b'a'); // 'a' prefix for account
    k.extend_from_slice(addr.as_bytes());
    k
}

#[must_use]
pub fn supply_key() -> Vec<u8> {
    vec![b's'] // 's' prefix for chain-wide supply state
}
