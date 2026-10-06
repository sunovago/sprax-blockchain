use sprax_consensus::{SignedProposal, Validator, ValidatorSet, Vote, VoteType};
use sprax_crypto::Ed25519Keypair;
use sprax_types::{Block, BlockBody, BlockHeader, Hash32};

#[test]
fn proposal_authentication_binds_genesis_round_height_and_header() {
    let signer = Ed25519Keypair::generate();
    let impostor = Ed25519Keypair::generate();
    let validators = ValidatorSet::new(vec![Validator::new(
        signer.address(),
        signer.public_key_bytes().to_vec(),
        1,
    )])
    .unwrap();
    let genesis = Hash32::new([41; 32]);
    let mut header = BlockHeader::genesis("sprax-testnet-1", Hash32::new([42; 32]));
    header.height = 1;
    header.parent_hash = Hash32::new([43; 32]);
    header.proposer = signer.address();
    let mut proposal = SignedProposal {
        genesis,
        signer: signer.address(),
        round: 3,
        valid_round: None,
        valid_round_votes: Vec::new(),
        block: Block {
            header,
            body: BlockBody::default(),
            last_commit: vec![],
        },
        signature: Vec::new(),
    };
    proposal.signature = signer.sign(&proposal.sign_bytes().unwrap());
    proposal
        .verify(genesis, signer.address(), &validators)
        .unwrap();
    assert!(proposal
        .verify(Hash32::new([44; 32]), signer.address(), &validators)
        .is_err());
    assert!(proposal
        .verify(genesis, impostor.address(), &validators)
        .is_err());
    for mutate in 0..4 {
        let mut forged = proposal.clone();
        match mutate {
            0 => forged.round += 1,
            1 => forged.block.header.height += 1,
            2 => forged.block.header.state_root = Hash32::ZERO,
            _ => forged.signature = impostor.sign(&forged.sign_bytes().unwrap()),
        }
        assert!(forged
            .verify(genesis, signer.address(), &validators)
            .is_err());
    }
}

#[test]
fn proposal_carries_a_verified_prior_round_unlock_certificate() {
    let keys: Vec<_> = (71..=74)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();
    let validators = ValidatorSet::new(
        keys.iter()
            .map(|key| Validator::new(key.address(), key.public_key_bytes().to_vec(), 1))
            .collect(),
    )
    .unwrap();
    let genesis = Hash32::new([51; 32]);
    let proposer = &keys[0];
    let mut header = BlockHeader::genesis("sprax-testnet-1", Hash32::new([52; 32]));
    header.height = 9;
    header.parent_hash = Hash32::new([53; 32]);
    header.proposer = proposer.address();
    header.validator_set_hash = validators.commitment().unwrap();
    let block = Block {
        header,
        body: BlockBody::default(),
        last_commit: vec![],
    };
    let block_hash = sprax_crypto::Hasher::block_hash(&block.header).unwrap();
    let valid_round_votes: Vec<_> = keys[1..]
        .iter()
        .map(|key| {
            let mut vote = Vote::new(
                genesis,
                VoteType::Prevote,
                9,
                0,
                Some(block_hash),
                key.address(),
                vec![],
            );
            vote.signature = key.sign(&vote.sign_bytes().unwrap());
            vote
        })
        .collect();
    let mut proposal = SignedProposal {
        genesis,
        signer: proposer.address(),
        round: 1,
        valid_round: Some(0),
        valid_round_votes,
        block,
        signature: vec![],
    };
    proposal.signature = proposer.sign(&proposal.sign_bytes().unwrap());
    proposal
        .verify(genesis, proposer.address(), &validators)
        .unwrap();

    let mut forged = proposal.clone();
    forged.valid_round = None;
    assert!(forged
        .verify(genesis, proposer.address(), &validators)
        .is_err());
    let mut forged = proposal.clone();
    forged.valid_round_votes[0].signature[0] ^= 1;
    assert!(forged
        .verify(genesis, proposer.address(), &validators)
        .is_err());
    let mut forged = proposal;
    forged.valid_round = Some(1);
    assert!(forged
        .verify(genesis, proposer.address(), &validators)
        .is_err());
}
