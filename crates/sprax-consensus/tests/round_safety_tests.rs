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
    Vote::new(kind, 1, round, hash, Address::new([validator; 20]), vec![])
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
fn a_conflicting_round_cannot_replace_a_precommit_lock_without_unlock_proof() {
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
    assert!(engine.record_precommit_lock(1, 1, conflicting).is_err());
    assert_eq!(engine.locked_round(), Some(0));
    engine.set_round(0);
    assert_eq!(engine.current_round(), 1);
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
