use sprax_consensus::{verify_block_commit, Validator, ValidatorSet, Vote, VoteType};
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_types::{Block, BlockBody, BlockHeader, CommitSignature, Hash32};

fn fixture() -> (Block, ValidatorSet) {
    let keys: Vec<_> = (80..84)
        .map(|n| Ed25519Keypair::from_seed(&[n; 32]))
        .collect();
    let validators = ValidatorSet::new(
        keys.iter()
            .map(|k| Validator::new(k.address(), k.public_key_bytes().to_vec(), 10))
            .collect(),
    )
    .unwrap();
    let mut block = Block {
        header: BlockHeader::genesis("test", Hash32::ZERO),
        body: BlockBody::default(),
        last_commit: vec![],
    };
    block.header.height = 1;
    let hash = Hasher::block_hash(&block.header).unwrap();
    for key in keys.iter().take(3) {
        let vote = Vote::new(VoteType::Precommit, 1, 2, Some(hash), key.address(), vec![]);
        block.last_commit.push(CommitSignature {
            round: 2,
            validator_address: key.address(),
            signature: key.sign(&vote.sign_bytes().unwrap()),
            timestamp_unix_secs: 0,
        });
    }
    (block, validators)
}

#[test]
fn valid_certificate_at_nonzero_round_verifies() {
    let (block, validators) = fixture();
    verify_block_commit(&block, &validators).unwrap();
}

#[test]
fn forged_duplicate_insufficient_or_replayed_certificates_fail() {
    let (block, validators) = fixture();
    let mut bad = block.clone();
    bad.last_commit[0].signature[0] ^= 1;
    assert!(verify_block_commit(&bad, &validators).is_err());
    let mut bad = block.clone();
    bad.last_commit[1] = bad.last_commit[0].clone();
    assert!(verify_block_commit(&bad, &validators).is_err());
    let mut bad = block.clone();
    bad.last_commit.pop();
    assert!(verify_block_commit(&bad, &validators).is_err());
    let mut bad = block.clone();
    bad.header.chain_id = "other-chain".into();
    assert!(verify_block_commit(&bad, &validators).is_err());
    let mut bad = block.clone();
    bad.last_commit[1].round = 3;
    assert!(verify_block_commit(&bad, &validators).is_err());
    let mut bad = block;
    bad.last_commit.clear();
    assert!(verify_block_commit(&bad, &validators).is_err());
}

#[test]
fn validator_set_rejects_duplicates_and_unsafe_power() {
    let key = Ed25519Keypair::from_seed(&[9; 32]);
    let validator = Validator::new(key.address(), key.public_key_bytes().to_vec(), 10);
    assert!(ValidatorSet::new(vec![validator.clone(), validator]).is_err());
    assert!(ValidatorSet::new(vec![Validator::new(
        key.address(),
        key.public_key_bytes().to_vec(),
        u64::MAX
    )])
    .is_err());
}
