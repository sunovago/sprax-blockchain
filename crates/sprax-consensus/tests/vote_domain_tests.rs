use sprax_consensus::{Vote, VoteType};
use sprax_crypto::Ed25519Keypair;
use sprax_types::{Address, Hash32};

#[test]
fn nil_and_block_votes_cannot_be_replayed_under_another_genesis_or_signer() {
    let key = Ed25519Keypair::from_seed(&[91; 32]);
    for kind in [VoteType::Prevote, VoteType::Precommit] {
        for hash in [None, Some(Hash32::new([92; 32]))] {
            let mut vote = Vote::new(
                Hash32::new([93; 32]),
                kind,
                5,
                2,
                hash,
                key.address(),
                vec![],
            );
            vote.signature = key.sign(&vote.sign_bytes().unwrap());
            let valid = |v: &Vote| {
                Ed25519Keypair::verify(
                    &key.public_key_bytes(),
                    &v.sign_bytes().unwrap(),
                    &v.signature,
                )
                .is_ok()
            };
            assert!(valid(&vote));
            let mut replay = vote.clone();
            replay.genesis = Hash32::new([94; 32]);
            assert!(!valid(&replay));
            replay = vote.clone();
            replay.validator_address = Address::ZERO;
            assert!(!valid(&replay));
            replay = vote;
            replay.vote_type = if kind == VoteType::Prevote {
                VoteType::Precommit
            } else {
                VoteType::Prevote
            };
            assert!(!valid(&replay));
        }
    }
}

#[test]
fn legacy_vote_messages_without_genesis_fail_deserialization() {
    let vote = Vote::new(
        Hash32::ZERO,
        VoteType::Prevote,
        1,
        0,
        None,
        Address::ZERO,
        vec![],
    );
    let mut encoded = serde_json::to_value(vote).unwrap();
    encoded.as_object_mut().unwrap().remove("genesis");
    assert!(serde_json::from_value::<Vote>(encoded).is_err());
}
