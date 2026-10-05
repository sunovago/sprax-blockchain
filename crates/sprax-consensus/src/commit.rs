use crate::{ConsensusError, ValidatorSet, Vote, VoteType};
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_types::{Block, Hash32};
use std::collections::HashSet;

/// Verifies a unique, same-round, signed >2/3 certificate for this exact block.
pub fn verify_block_commit(
    block: &Block,
    validators: &ValidatorSet,
    genesis: Hash32,
) -> Result<(), ConsensusError> {
    // Bound certificate work before hashing or verifying any attacker-supplied signature.
    if block.last_commit.len() > validators.validators().len()
        || block
            .last_commit
            .iter()
            .any(|commit| commit.signature.len() != 64)
    {
        return Err(ConsensusError::InvalidVote(
            "oversized certificate or invalid signature length".into(),
        ));
    }
    if block.header.validator_set_hash != validators.commitment()? {
        return Err(ConsensusError::InvalidVote(
            "commit validator-set commitment mismatch".into(),
        ));
    }
    let hash = Hasher::block_hash(&block.header)
        .map_err(|e| ConsensusError::InvalidVote(e.to_string()))?;
    let round = block
        .last_commit
        .first()
        .ok_or_else(|| ConsensusError::InvalidVote("missing block commit".into()))?
        .round;
    let mut seen = HashSet::new();
    let mut power = 0u64;
    for commit in &block.last_commit {
        if commit.round != round || !seen.insert(commit.validator_address) {
            return Err(ConsensusError::InvalidVote(
                "duplicate signer or mixed commit rounds".into(),
            ));
        }
        let validator = validators
            .validators()
            .iter()
            .find(|v| v.address == commit.validator_address)
            .ok_or_else(|| ConsensusError::InvalidVote("unknown commit signer".into()))?;
        let vote = Vote::new(
            genesis,
            VoteType::Precommit,
            block.header.height,
            round,
            Some(hash),
            commit.validator_address,
            commit.signature.clone(),
        );
        Ed25519Keypair::verify(
            &validator.public_key,
            &vote.sign_bytes()?,
            &commit.signature,
        )
        .map_err(|e| ConsensusError::InvalidVote(format!("invalid commit signature: {e}")))?;
        power = power
            .checked_add(validator.voting_power)
            .ok_or_else(|| ConsensusError::InvalidVote("commit voting power overflow".into()))?;
    }
    if !validators.has_quorum(power) {
        return Err(ConsensusError::InvalidVote(
            "insufficient commit voting power".into(),
        ));
    }
    Ok(())
}
