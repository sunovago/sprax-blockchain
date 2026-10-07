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

/// Verifies the +2/3 signed prevote proof required to replace a durable lock with a
/// different block. The caller must also validate that the candidate block commits
/// this validator set before asking the signing journal to accept the unlock.
pub fn verify_prevote_quorum(
    votes: &[Vote],
    validators: &ValidatorSet,
    genesis: Hash32,
    height: u64,
    round: u32,
    block_hash: Hash32,
) -> Result<(), ConsensusError> {
    if votes.len() > validators.len() {
        return Err(ConsensusError::InvalidVote(
            "prevote certificate exceeds active validator count".into(),
        ));
    }
    let mut seen = HashSet::new();
    let mut power = 0u64;
    for vote in votes {
        if vote.genesis != genesis
            || vote.vote_type != VoteType::Prevote
            || vote.height != height
            || vote.round != round
            || vote.block_hash != Some(block_hash)
            || vote.signature.len() != 64
            || !seen.insert(vote.validator_address)
        {
            return Err(ConsensusError::InvalidVote(
                "invalid prevote certificate member".into(),
            ));
        }
        let validator = validators
            .validators()
            .iter()
            .find(|validator| validator.address == vote.validator_address)
            .ok_or_else(|| ConsensusError::InvalidVote("unknown prevote signer".into()))?;
        sprax_crypto::Ed25519Keypair::verify(
            &validator.public_key,
            &vote.sign_bytes()?,
            &vote.signature,
        )
        .map_err(|error| {
            ConsensusError::InvalidVote(format!("invalid prevote signature: {error}"))
        })?;
        power = power
            .checked_add(validator.voting_power)
            .ok_or_else(|| ConsensusError::InvalidVote("prevote power overflow".into()))?;
    }
    if !validators.has_quorum(power) {
        return Err(ConsensusError::InvalidVote(
            "insufficient signed prevote power to unlock".into(),
        ));
    }
    Ok(())
}
