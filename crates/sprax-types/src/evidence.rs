use crate::{address::Address, error::TypeError, hash::Hash32};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
    #[must_use]
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
    pub fn sign_bytes(&self) -> Result<Vec<u8>, TypeError> {
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
        crate::canonical::to_canonical_json_bytes(&signable)
            .map_err(|e| TypeError::InvalidTransaction(format!("serialization error: {e}")))
    }
}

/// Slashing evidence for validator equivocation (double-signing).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquivocationEvidence {
    pub validator_address: Address,
    pub height: u64,
    pub round: u32,
    pub vote_a: Vote,
    pub vote_b: Vote,
}

impl EquivocationEvidence {
    #[must_use]
    pub fn new(
        validator_address: Address,
        height: u64,
        round: u32,
        vote_a: Vote,
        vote_b: Vote,
    ) -> Self {
        Self {
            validator_address,
            height,
            round,
            vote_a,
            vote_b,
        }
    }

    #[must_use]
    pub fn is_valid_equivocation(&self) -> bool {
        self.validator_address == self.vote_a.validator_address
            && self.vote_a.genesis == self.vote_b.genesis
            && self.height == self.vote_a.height
            && self.round == self.vote_a.round
            && self.vote_a.validator_address == self.vote_b.validator_address
            && self.vote_a.height == self.vote_b.height
            && self.vote_a.round == self.vote_b.round
            && self.vote_a.vote_type == self.vote_b.vote_type
            && self.vote_a.block_hash != self.vote_b.block_hash
    }

    pub fn verify_signatures(&self, public_key: &[u8]) -> Result<(), TypeError> {
        if !self.is_valid_equivocation() {
            return Err(TypeError::InvalidTransaction(
                "invalid equivocation metadata".into(),
            ));
        }
        let pk_bytes: [u8; 32] = public_key
            .try_into()
            .map_err(|_| TypeError::InvalidTransaction("public key must be 32 bytes".into()))?;
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&pk_bytes).map_err(|e| {
            TypeError::InvalidTransaction(format!("invalid ed25519 public key: {e}"))
        })?;

        for vote in [&self.vote_a, &self.vote_b] {
            let sign_bytes = vote.sign_bytes()?;
            let sig_bytes: [u8; 64] =
                vote.signature.as_slice().try_into().map_err(|_| {
                    TypeError::InvalidTransaction("signature must be 64 bytes".into())
                })?;
            let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
            verifying_key
                .verify_strict(&sign_bytes, &signature)
                .map_err(|e| {
                    TypeError::InvalidTransaction(format!("forged equivocation signature: {e}"))
                })?;
        }
        Ok(())
    }

    /// Computes canonical 32-byte identifier for deduplicating evidence regardless of vote order.
    #[must_use]
    pub fn canonical_hash(&self) -> Hash32 {
        let (v_a, v_b) = {
            let a_bytes = serde_json::to_vec(&self.vote_a).unwrap_or_default();
            let b_bytes = serde_json::to_vec(&self.vote_b).unwrap_or_default();
            if a_bytes <= b_bytes {
                (&self.vote_a, &self.vote_b)
            } else {
                (&self.vote_b, &self.vote_a)
            }
        };

        #[derive(Serialize)]
        struct NormalizedEvidence<'a> {
            domain: &'static str,
            validator_address: Address,
            height: u64,
            round: u32,
            vote_a: &'a Vote,
            vote_b: &'a Vote,
        }

        let normalized = NormalizedEvidence {
            domain: "sprax/evidence/v1",
            validator_address: self.validator_address,
            height: self.height,
            round: self.round,
            vote_a: v_a,
            vote_b: v_b,
        };

        let bytes = serde_json::to_vec(&normalized).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Hash32::new(hasher.finalize().into())
    }
}
