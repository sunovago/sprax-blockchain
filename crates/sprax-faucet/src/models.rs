use serde::{Deserialize, Serialize};
use sprax_types::{Address, Amount, Hash32};

/// Faucet claim request payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimRequest {
    pub recipient: String,
    pub amount_sprx: Option<u64>,
    pub client_ip: Option<String>,
}

/// Faucet claim response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimResponse {
    pub success: bool,
    pub tx_hash: Hash32,
    pub recipient: Address,
    pub amount: Amount,
    pub amount_sprx: String,
    pub message: String,
    pub network_notice: String,
}

/// Public faucet status and operational metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaucetStats {
    pub network: String,
    pub faucet_address: Address,
    pub available_balance: Amount,
    pub total_disbursed: Amount,
    pub total_claims_count: usize,
    pub max_payout_per_request: Amount,
    pub rate_limit_window_secs: u64,
    pub disclaimer: String,
}

/// Persistent audit record for compliance and abuse tracking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp_unix: u64,
    pub recipient: Address,
    pub amount: Amount,
    pub tx_hash: Hash32,
    pub client_ip: String,
}
