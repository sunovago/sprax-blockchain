use serde::{Deserialize, Serialize};
use sprax_types::{Address, Amount, Hash32, TxMessage};

/// Paginated query envelope for Explorer tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
    pub has_more: bool,
}

/// Fully indexed block model with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedBlock {
    pub height: u64,
    pub hash: Hash32,
    pub parent_hash: Hash32,
    pub chain_id: String,
    pub timestamp_unix_secs: u64,
    pub proposer: Address,
    pub txs_count: usize,
    pub txs_root: Hash32,
    pub state_root: Hash32,
    pub gas_used: u64,
    pub block_size_bytes: usize,
}

/// Fully indexed transaction model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedTx {
    pub tx_hash: Hash32,
    pub block_height: u64,
    pub block_hash: Hash32,
    pub sender: Address,
    pub recipient: Option<Address>,
    pub message_type: String,
    pub amount: Amount,
    pub fee_amount: Amount,
    pub nonce: u64,
    pub memo: String,
    pub success: bool,
    pub gas_used: u64,
    pub timestamp_unix_secs: u64,
    pub raw_messages: Vec<TxMessage>,
}

/// Indexed account profile and balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedAccount {
    pub address: Address,
    pub address_hex: String,
    pub balance: Amount,
    pub balance_sprx: String,
    pub nonce: u64,
    pub tx_count: usize,
    pub first_seen_height: u64,
    pub last_active_height: u64,
}

/// Indexed validator ranking and performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedValidator {
    pub operator_address: Address,
    pub moniker: String,
    pub voting_power: u64,
    pub voting_power_percentage: f64,
    pub tokens: Amount,
    pub commission_rate: f64,
    pub status: String,
    pub is_tombstoned: bool,
    pub missed_blocks_count: u64,
    pub uptime_percentage: f64,
    pub blocks_proposed_count: usize,
}

/// Real-time high-level network statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub chain_id: String,
    pub latest_height: u64,
    pub latest_block_hash: Hash32,
    pub total_transactions: usize,
    pub total_accounts: usize,
    pub active_validators_count: usize,
    pub total_bonded_tokens: Amount,
    pub avg_block_time_seconds: f64,
    pub current_tps: f64,
    pub latest_state_root: Hash32,
}

/// Search result classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SearchResult {
    Block(IndexedBlock),
    Transaction(IndexedTx),
    Address(IndexedAccount),
    Validator(IndexedValidator),
}
