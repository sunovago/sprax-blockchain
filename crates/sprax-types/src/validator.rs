use crate::Address;
use serde::{Deserialize, Serialize};

/// Consensus identity and power committed by a block header. Local proposer priorities
/// and operational staking caches are deliberately excluded from this representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalValidator {
    pub address: Address,
    pub public_key: Vec<u8>,
    pub voting_power: u64,
}
