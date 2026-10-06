use sprax_consensus::{BftConsensusEngine, Validator, ValidatorSet, Vote, VoteType};
use sprax_types::{Address, Hash32};

fn engine() -> BftConsensusEngine {
    BftConsensusEngine::new(
        1,
        ValidatorSet::new(
            (1..=3)
                .map(|i| Validator::new(Address::new([i; 20]), vec![i; 32], 1))
                .collect(),
        )
        .unwrap(),
    )
}
fn vote(kind: VoteType, round: u32, hash: Option<Hash32>, validator: u8) -> Vote {
    Vote::new(
        sprax_types::Hash32::ZERO,
        kind,
        1,
        round,
        hash,
        Address::new([validator; 20]),
        vec![],
    )
}
#[test]
fn quorum_for_unknown_data_does_not_create_a_lock() {
    let mut engine = engine();
    let hash = Hash32::new([7; 32]);
    for i in 1..=3 {
        engine
            .receive_prevote(vote(VoteType::Prevote, 0, Some(hash), i))
            .unwrap();
    }
    assert!(engine.has_prevote_quorum(1, 0));
    assert_eq!(engine.locked_block(), None);
    assert_eq!(engine.valid_block(), None);
    assert!(engine.record_precommit_lock(1, 0, hash).is_err());
}
#[test]
fn a_later_round_quorum_certificate_allows_safe_lock_replacement() {
    let mut engine = engine();
    let original = Hash32::new([7; 32]);
    engine
        .propose_block(original, Address::new([1; 20]))
        .unwrap();
    for i in 1..=3 {
        engine
            .receive_prevote(vote(VoteType::Prevote, 0, Some(original), i))
            .unwrap();
    }
    assert_eq!(engine.locked_block(), None);
    engine.record_precommit_lock(1, 0, original).unwrap();
    engine.set_round(1);
    let conflicting = Hash32::new([8; 32]);
    engine
        .propose_block(conflicting, Address::new([2; 20]))
        .unwrap();
    for i in 1..=3 {
        engine
            .receive_prevote(vote(VoteType::Prevote, 1, Some(conflicting), i))
            .unwrap();
    }
    assert_eq!(engine.locked_block(), Some(original));
    assert!(engine.can_record_precommit_lock(1, 1, conflicting));
    engine.record_precommit_lock(1, 1, conflicting).unwrap();
    assert_eq!(engine.locked_block(), Some(conflicting));
    assert_eq!(engine.locked_round(), Some(1));
    engine.set_round(0);
    assert_eq!(engine.current_round(), 1);
}

#[test]
fn a_verified_proposal_valid_round_allows_locked_validator_to_prevote_conflict() {
    let mut engine = engine();
    let locked = Hash32::new([17; 32]);
    engine.propose_block(locked, Address::new([1; 20])).unwrap();
    for i in 1..=3 {
        engine
            .receive_prevote(vote(VoteType::Prevote, 0, Some(locked), i))
            .unwrap();
    }
    engine.record_precommit_lock(1, 0, locked).unwrap();

    let replacement = Hash32::new([18; 32]);
    engine.set_round(2);
    engine
        .propose_block(replacement, Address::new([2; 20]))
        .unwrap();
    assert!(!engine.can_prevote_block(replacement));

    let certificate: Vec<_> = (1..=3)
        .map(|i| vote(VoteType::Prevote, 1, Some(replacement), i))
        .collect();
    engine.install_valid_round_certificate(1, 1, replacement, &certificate);
    assert!(engine.can_prevote_block(replacement));
    assert!(!engine.can_prevote_block(locked));
}

#[test]
fn nil_precommits_finish_a_round_without_finalizing_a_block() {
    let mut engine = engine();
    for i in 1..=3 {
        assert!(engine
            .receive_precommit(vote(VoteType::Precommit, 0, None, i))
            .unwrap()
            .is_none());
    }
    assert!(engine.has_nil_precommit_quorum(1, 0));
    assert_eq!(engine.get_precommitted_block_with_quorum(1, 0), None);
    assert_eq!(engine.locked_block(), None);
}
#[test]
fn stalled_heights_keep_a_bounded_vote_history() {
    let mut engine = engine();
    for round in 0..1000 {
        engine.set_round(round);
        for i in 1..=3 {
            engine
                .receive_prevote(vote(VoteType::Prevote, round, None, i))
                .unwrap();
            engine
                .receive_precommit(vote(VoteType::Precommit, round, None, i))
                .unwrap();
        }
        assert!(engine.retained_vote_count() <= 18);
    }
    assert_eq!(engine.retained_vote_count(), 6);
}
