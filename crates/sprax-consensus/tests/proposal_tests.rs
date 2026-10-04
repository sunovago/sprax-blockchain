use sprax_consensus::{SignedProposal, Validator, ValidatorSet};
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
        round: 3,
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
