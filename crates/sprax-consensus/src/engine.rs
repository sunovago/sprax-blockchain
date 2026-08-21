use crate::{
    error::ConsensusError,
    round::{RoundState, RoundStep},
    validator::{Validator, ValidatorSet},
    vote::{Vote, VoteType},
};
use sprax_types::{Address, CommitSignature, Hash32};
use std::collections::HashMap;

/// CometBFT 2-Step Consensus Engine State Machine.
#[derive(Debug, Clone)]
pub struct BftConsensusEngine {
    state: RoundState,
    val_set: ValidatorSet,
    prevotes: HashMap<(u64, u32, Address), Vote>,
    precommits: HashMap<(u64, u32, Address), Vote>,
    proposed_blocks: HashMap<(u64, u32), Hash32>,
}

impl BftConsensusEngine {
    pub fn new(initial_height: u64, val_set: ValidatorSet) -> Self {
        Self {
            state: RoundState::new(initial_height),
            val_set,
            prevotes: HashMap::new(),
            precommits: HashMap::new(),
            proposed_blocks: HashMap::new(),
        }
    }

    #[must_use]
    pub fn current_height(&self) -> u64 {
        self.state.height
    }

    #[must_use]
    pub fn current_round(&self) -> u32 {
        self.state.round
    }

    #[must_use]
    pub fn current_step(&self) -> RoundStep {
        self.state.step
    }

    #[must_use]
    pub fn locked_round(&self) -> Option<u32> {
        self.state.locked_round
    }

    #[must_use]
    pub fn locked_block(&self) -> Option<Hash32> {
        self.state.locked_block
    }

    #[must_use]
    pub fn valid_round(&self) -> Option<u32> {
        self.state.valid_round
    }

    #[must_use]
    pub fn valid_block(&self) -> Option<Hash32> {
        self.state.valid_block
    }

    #[must_use]
    pub fn validator_set(&self) -> &ValidatorSet {
        &self.val_set
    }

    /// Replaces the active validator set (e.g. reflecting the latest `StakingKeeper` state at
    /// the start of a new height, after delegations/slashing since the last height).
    ///
    /// Carries `proposer_priority` forward for validators present in both the old and new set:
    /// `StakingKeeper::get_active_validator_set` builds fresh `Validator` instances (priority
    /// always 0) on every call, so naively replacing the set here would reset DWRR state every
    /// single height — which breaks fairness (the highest-stake validator would win every
    /// height's initial-priority tiebreak forever) rather than rotating proposers over time.
    /// Only validators new to the set start at priority 0.
    pub fn set_validator_set(&mut self, mut val_set: ValidatorSet) {
        for new_val in val_set.validators_mut() {
            if let Some(old_val) = self
                .val_set
                .validators()
                .iter()
                .find(|v| v.address == new_val.address)
            {
                new_val.proposer_priority = old_val.proposer_priority;
            }
        }
        self.val_set = val_set;
    }

    /// Looks up an already-stored vote from the same validator at the same (height, round) that
    /// conflicts with `vote` (different `block_hash`), without mutating any state. Callers
    /// should check this *before* handing a vote to [`Self::receive_prevote`] /
    /// [`Self::receive_precommit`] in order to detect equivocation while the prior vote is
    /// still intact — the tally itself keeps only the first vote seen per validator per round.
    #[must_use]
    pub fn find_conflicting_vote(&self, vote: &Vote) -> Option<Vote> {
        let key = (vote.height, vote.round, vote.validator_address);
        let existing = match vote.vote_type {
            VoteType::Prevote => self.prevotes.get(&key),
            VoteType::Precommit => self.precommits.get(&key),
        };
        existing
            .filter(|stored| stored.block_hash != vote.block_hash)
            .cloned()
    }

    /// Transitions engine to a new height.
    pub fn start_height(&mut self, height: u64) {
        self.state = RoundState::new(height);
        self.prevotes.clear();
        self.precommits.clear();
        self.proposed_blocks.clear();
        self.state.step = RoundStep::Propose;
    }

    /// Sets the current round within the current height (called after [`Self::start_height`]
    /// when a driving caller retries the same height in a new round after the previous round
    /// failed to reach quorum). Without this, every retry would silently reuse round 0 — a
    /// second, differently-hashed vote from the same validator at the same (height, round) is
    /// exactly what equivocation detection is designed to catch, so honest retries would be
    /// indistinguishable from real double-signing to any observer.
    pub fn set_round(&mut self, round: u32) {
        self.state.round = round;
    }

    /// Selects the deterministic proposer for the current round using DWRR.
    pub fn select_proposer(&mut self) -> Validator {
        self.val_set.select_proposer()
    }

    /// Submits a block proposal for the current (height, round).
    pub fn propose_block(
        &mut self,
        block_hash: Hash32,
        proposer: Address,
    ) -> Result<(), ConsensusError> {
        let expected_proposer = self
            .val_set
            .validators()
            .iter()
            .find(|v| v.address == proposer);
        if expected_proposer.is_none() {
            return Err(ConsensusError::InvalidProposer(
                "proposer is not in active validator set".into(),
            ));
        }

        self.proposed_blocks
            .insert((self.state.height, self.state.round), block_hash);
        self.state.step = RoundStep::Prevote;
        Ok(())
    }

    /// Casts or receives a Prevote attestation.
    pub fn receive_prevote(&mut self, vote: Vote) -> Result<bool, ConsensusError> {
        if vote.vote_type != VoteType::Prevote {
            return Err(ConsensusError::InvalidVote("expected Prevote vote".into()));
        }
        if vote.height != self.state.height || vote.round != self.state.round {
            return Err(ConsensusError::InvalidVote(
                "vote height/round mismatch".into(),
            ));
        }

        // Verify voter is in validator set
        if !self
            .val_set
            .validators()
            .iter()
            .any(|v| v.address == vote.validator_address)
        {
            return Err(ConsensusError::InvalidVote(
                "voter is not an active validator".into(),
            ));
        }

        // Keep the first vote seen per validator per round: a Byzantine validator flooding
        // conflicting votes must not be able to overwrite/confuse the quorum tally. Callers
        // detect the conflict via `find_conflicting_vote` before reaching this point.
        self.prevotes
            .entry((vote.height, vote.round, vote.validator_address))
            .or_insert(vote);

        // Check if +2/3 prevote quorum has been reached for any block hash
        let has_quorum = self.has_prevote_quorum(self.state.height, self.state.round);
        if has_quorum {
            self.state.step = RoundStep::Precommit;
            if let Some(bh) =
                self.get_prevoted_block_with_quorum(self.state.height, self.state.round)
            {
                self.state.locked_round = Some(self.state.round);
                self.state.locked_block = Some(bh);
                self.state.valid_round = Some(self.state.round);
                self.state.valid_block = Some(bh);
            }
        }

        Ok(has_quorum)
    }

    /// Casts or receives a Precommit attestation. Returns aggregated commit signatures if +2/3 precommits reached.
    pub fn receive_precommit(
        &mut self,
        vote: Vote,
    ) -> Result<Option<Vec<CommitSignature>>, ConsensusError> {
        if vote.vote_type != VoteType::Precommit {
            return Err(ConsensusError::InvalidVote(
                "expected Precommit vote".into(),
            ));
        }
        if vote.height != self.state.height || vote.round != self.state.round {
            return Err(ConsensusError::InvalidVote(
                "vote height/round mismatch".into(),
            ));
        }

        if !self
            .val_set
            .validators()
            .iter()
            .any(|v| v.address == vote.validator_address)
        {
            return Err(ConsensusError::InvalidVote(
                "voter is not an active validator".into(),
            ));
        }

        self.precommits
            .entry((vote.height, vote.round, vote.validator_address))
            .or_insert(vote);

        if let Some(target_hash) =
            self.get_precommitted_block_with_quorum(self.state.height, self.state.round)
        {
            let commit_sigs =
                self.aggregate_commit_signatures(self.state.height, self.state.round, target_hash);
            self.state.step = RoundStep::Commit;
            return Ok(Some(commit_sigs));
        }

        Ok(None)
    }

    /// Checks if a +2/3 prevote quorum exists.
    pub fn has_prevote_quorum(&self, height: u64, round: u32) -> bool {
        let mut power_by_hash: HashMap<Option<Hash32>, u64> = HashMap::new();

        for ((h, r, addr), vote) in &self.prevotes {
            if *h == height && *r == round {
                if let Some(val) = self
                    .val_set
                    .validators()
                    .iter()
                    .find(|v| v.address == *addr)
                {
                    *power_by_hash.entry(vote.block_hash).or_insert(0) += val.voting_power;
                }
            }
        }

        power_by_hash.values().any(|&p| self.val_set.has_quorum(p))
    }

    /// Finds the block hash that has received +2/3 prevote quorum, if any (mirrors
    /// [`Self::get_precommitted_block_with_quorum`] for the prevote phase) — used by a driving
    /// caller to know which hash to cast its own reactive precommit for.
    pub fn get_prevoted_block_with_quorum(&self, height: u64, round: u32) -> Option<Hash32> {
        let mut power_by_hash: HashMap<Hash32, u64> = HashMap::new();

        for ((h, r, addr), vote) in &self.prevotes {
            if *h == height && *r == round {
                if let Some(hash) = vote.block_hash {
                    if let Some(val) = self
                        .val_set
                        .validators()
                        .iter()
                        .find(|v| v.address == *addr)
                    {
                        *power_by_hash.entry(hash).or_insert(0) += val.voting_power;
                    }
                }
            }
        }

        for (hash, power) in power_by_hash {
            if self.val_set.has_quorum(power) {
                return Some(hash);
            }
        }
        None
    }

    /// Finds the block hash that has received +2/3 precommit quorum.
    pub fn get_precommitted_block_with_quorum(&self, height: u64, round: u32) -> Option<Hash32> {
        let mut power_by_hash: HashMap<Hash32, u64> = HashMap::new();

        for ((h, r, addr), vote) in &self.precommits {
            if *h == height && *r == round {
                if let Some(hash) = vote.block_hash {
                    if let Some(val) = self
                        .val_set
                        .validators()
                        .iter()
                        .find(|v| v.address == *addr)
                    {
                        *power_by_hash.entry(hash).or_insert(0) += val.voting_power;
                    }
                }
            }
        }

        for (hash, power) in power_by_hash {
            if self.val_set.has_quorum(power) {
                return Some(hash);
            }
        }
        None
    }

    /// Aggregates all commit signatures from validators that voted for the finalized block.
    pub fn aggregate_commit_signatures(
        &self,
        height: u64,
        round: u32,
        block_hash: Hash32,
    ) -> Vec<CommitSignature> {
        let mut sigs = Vec::new();
        for ((h, r, addr), vote) in &self.precommits {
            if *h == height && *r == round && vote.block_hash == Some(block_hash) {
                sigs.push(CommitSignature {
                    validator_address: *addr,
                    signature: vote.signature.clone(),
                    timestamp_unix_secs: 1_700_000_000,
                });
            }
        }
        sigs.sort_by_key(|s| s.validator_address);
        sigs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sprax_crypto::Ed25519Keypair;

    #[test]
    fn test_full_bft_consensus_round_convergence() {
        let val1_kp = Ed25519Keypair::from_seed(&[1u8; 32]);
        let val2_kp = Ed25519Keypair::from_seed(&[2u8; 32]);
        let val3_kp = Ed25519Keypair::from_seed(&[3u8; 32]);

        let v1 = Validator::new(val1_kp.address(), val1_kp.public_key_bytes().to_vec(), 40);
        let v2 = Validator::new(val2_kp.address(), val2_kp.public_key_bytes().to_vec(), 35);
        let v3 = Validator::new(val3_kp.address(), val3_kp.public_key_bytes().to_vec(), 25);

        let val_set = ValidatorSet::new(vec![v1.clone(), v2.clone(), v3.clone()]).unwrap();
        assert_eq!(val_set.total_voting_power(), 100);
        assert_eq!(val_set.quorum_threshold(), 67);

        let mut engine = BftConsensusEngine::new(1, val_set);
        engine.start_height(1);
        assert_eq!(engine.current_step(), RoundStep::Propose);

        let proposer = engine.select_proposer();
        let proposal_hash = Hash32::new([0xaa; 32]);
        engine
            .propose_block(proposal_hash, proposer.address)
            .unwrap();
        assert_eq!(engine.current_step(), RoundStep::Prevote);

        // Validators 1 & 2 cast prevote (40 + 35 = 75 >= 67 quorum)
        let pv1 = signed_vote(VoteType::Prevote, 1, 0, Some(proposal_hash), &val1_kp);
        let pv2 = signed_vote(VoteType::Prevote, 1, 0, Some(proposal_hash), &val2_kp);

        assert!(!engine.receive_prevote(pv1).unwrap()); // 40 power < 67
        assert!(engine.receive_prevote(pv2).unwrap()); // 75 power >= 67 -> step becomes Precommit
        assert_eq!(engine.current_step(), RoundStep::Precommit);

        // Validators 1 & 2 cast precommit
        let pc1 = signed_vote(VoteType::Precommit, 1, 0, Some(proposal_hash), &val1_kp);
        let pc2 = signed_vote(VoteType::Precommit, 1, 0, Some(proposal_hash), &val2_kp);

        assert!(engine.receive_precommit(pc1).unwrap().is_none());
        let commit_sigs = engine.receive_precommit(pc2).unwrap().unwrap();

        assert_eq!(engine.current_step(), RoundStep::Commit);
        assert_eq!(commit_sigs.len(), 2);
    }

    /// Builds a `Vote` signed over its own canonical `sign_bytes()` (not a raw block hash),
    /// matching how `ConsensusDriver` will construct and verify votes in production.
    fn signed_vote(
        vote_type: VoteType,
        height: u64,
        round: u32,
        block_hash: Option<Hash32>,
        signer: &Ed25519Keypair,
    ) -> Vote {
        let address = {
            // Derived from the signer's own address so `validator_address` matches the signer.
            signer.address()
        };
        let mut vote = Vote::new(vote_type, height, round, block_hash, address, vec![]);
        let sign_bytes = vote.sign_bytes().unwrap();
        vote.signature = signer.sign(&sign_bytes);
        vote
    }

    #[test]
    fn test_find_conflicting_vote_detects_equivocation_without_mutating_tally() {
        let val1_kp = Ed25519Keypair::from_seed(&[1u8; 32]);
        let v1 = Validator::new(val1_kp.address(), val1_kp.public_key_bytes().to_vec(), 100);
        let val_set = ValidatorSet::new(vec![v1.clone()]).unwrap();
        let mut engine = BftConsensusEngine::new(1, val_set);
        engine.start_height(1);

        let hash_a = Hash32::new([0xaa; 32]);
        let hash_b = Hash32::new([0xbb; 32]);

        let vote_a = signed_vote(VoteType::Prevote, 1, 0, Some(hash_a), &val1_kp);
        engine.receive_prevote(vote_a.clone()).unwrap();

        let vote_b = signed_vote(VoteType::Prevote, 1, 0, Some(hash_b), &val1_kp);
        let conflict = engine.find_conflicting_vote(&vote_b);
        assert_eq!(conflict, Some(vote_a));
    }

    #[test]
    fn test_first_vote_wins_on_conflicting_second_vote() {
        let val1_kp = Ed25519Keypair::from_seed(&[1u8; 32]);
        let val2_kp = Ed25519Keypair::from_seed(&[2u8; 32]);
        let v1 = Validator::new(val1_kp.address(), val1_kp.public_key_bytes().to_vec(), 60);
        let v2 = Validator::new(val2_kp.address(), val2_kp.public_key_bytes().to_vec(), 40);
        let val_set = ValidatorSet::new(vec![v1, v2]).unwrap();
        let mut engine = BftConsensusEngine::new(1, val_set);
        engine.start_height(1);

        let hash_a = Hash32::new([0xaa; 32]);
        let hash_b = Hash32::new([0xbb; 32]);

        // Validator 1 double-votes: first for hash_a, then (maliciously) for hash_b.
        let first = signed_vote(VoteType::Prevote, 1, 0, Some(hash_a), &val1_kp);
        engine.receive_prevote(first).unwrap();
        let second = signed_vote(VoteType::Prevote, 1, 0, Some(hash_b), &val1_kp);
        engine.receive_prevote(second).unwrap();

        // Only the first vote (for hash_a, 60 power) counts toward quorum for hash_a; the
        // conflicting second vote must not have overwritten it or split/duplicated power.
        assert!(!engine.has_prevote_quorum(1, 0));

        // Validator 2 also votes for hash_a: 60 + 40 = 100 >= 67 quorum for hash_a specifically,
        // proving validator 1's tally stayed pinned to hash_a despite the second vote.
        let v2_vote = signed_vote(VoteType::Prevote, 1, 0, Some(hash_a), &val2_kp);
        assert!(engine.receive_prevote(v2_vote).unwrap());
    }
}
