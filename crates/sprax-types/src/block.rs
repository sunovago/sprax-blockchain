use crate::{address::Address, error::TypeError, hash::Hash32, transaction::Transaction};
use serde::{Deserialize, Serialize};

/// Canonical block header in the SPRX Protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub chain_id: String,
    pub height: u64,
    pub timestamp_unix_secs: u64,
    pub parent_hash: Hash32,
    pub proposer: Address,
    pub state_root: Hash32,
    pub txs_root: Hash32,
    pub receipts_root: Hash32,
    pub validator_set_hash: Hash32,
}

impl BlockHeader {
    pub fn genesis(chain_id: impl Into<String>, initial_state_root: Hash32) -> Self {
        Self {
            version: 1,
            chain_id: chain_id.into(),
            height: 0,
            timestamp_unix_secs: 1_700_000_000,
            parent_hash: Hash32::ZERO,
            proposer: Address::ZERO,
            state_root: initial_state_root,
            txs_root: Hash32::ZERO,
            receipts_root: Hash32::ZERO,
            validator_set_hash: Hash32::ZERO,
        }
    }
}

/// Block body containing verified transactions.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BlockBody {
    pub transactions: Vec<Transaction>,
}

/// Commit signature from an active validator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitSignature {
    pub validator_address: Address,
    pub signature: Vec<u8>,
    pub timestamp_unix_secs: u64,
}

/// Complete verified block package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub body: BlockBody,
    pub last_commit: Vec<CommitSignature>,
}

impl Block {
    pub fn new(
        header: BlockHeader,
        body: BlockBody,
        last_commit: Vec<CommitSignature>,
    ) -> Result<Self, TypeError> {
        if header.height > 0 && header.parent_hash == Hash32::ZERO {
            return Err(TypeError::InvalidBlock(
                "non-genesis block cannot have ZERO parent hash".into(),
            ));
        }
        Ok(Self {
            header,
            body,
            last_commit,
        })
    }
}
