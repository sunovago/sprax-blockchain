use crate::error::ConsensusError;
use serde::{Deserialize, Serialize};
use sprax_types::{Address, Hash32};

/// Type of consensus attestation vote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoteType {
    Prevote,
    Precommit,
}

/// Signed validator vote message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vote {
    pub genesis: Hash32,
    pub vote_type: VoteType,
    pub height: u64,
    pub round: u32,
    pub block_hash: Option<Hash32>, // None indicates a Prevote(Nil) or Precommit(Nil)
    pub validator_address: Address,
    pub signature: Vec<u8>,
}

impl Vote {
    pub fn new(
        genesis: Hash32,
        vote_type: VoteType,
        height: u64,
        round: u32,
        block_hash: Option<Hash32>,
        validator_address: Address,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            genesis,
            vote_type,
            height,
            round,
            block_hash,
            validator_address,
            signature,
        }
    }

    /// Versioned, genesis-bound bytes including signer identity. Nil votes are bound
    /// to the same network as block votes. Legacy unbound encodings are not accepted.
    pub fn sign_bytes(&self) -> Result<Vec<u8>, ConsensusError> {
        #[derive(Serialize)]
        struct SignableVote {
            domain: &'static str,
            genesis: Hash32,
            validator_address: Address,
            vote_type: VoteType,
            height: u64,
            round: u32,
            block_hash: Option<Hash32>,
        }
        let signable = SignableVote {
            domain: "sprax/vote/v2",
            genesis: self.genesis,
            validator_address: self.validator_address,
            vote_type: self.vote_type,
            height: self.height,
            round: self.round,
            block_hash: self.block_hash,
        };
        serde_json::to_vec(&signable)
            .map_err(|e| ConsensusError::InvalidVote(format!("sign_bytes encode error: {e}")))
    }
}
