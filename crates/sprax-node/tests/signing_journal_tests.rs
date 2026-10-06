use sprax_consensus::SignedProposal;
use sprax_consensus::{Validator, ValidatorSet, Vote, VoteType};
use sprax_types::{Block, BlockBody, BlockHeader};

#[test]
fn proposal_signatures_survive_restarts_and_conflicting_retries_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("proposal-signing.redb");
    let signer = Ed25519Keypair::generate();
    let genesis = Hash32::new([61; 32]);
    let validators = ValidatorSet::new(vec![Validator::new(
        signer.address(),
        signer.public_key_bytes().to_vec(),
        1,
    )])
    .unwrap();
    let mut header = BlockHeader::genesis("sprax-testnet-1", Hash32::new([62; 32]));
    header.proposer = signer.address();
    header.height = 1;
    header.parent_hash = Hash32::new([63; 32]);
    let proposal = SignedProposal {
        genesis,
        signer: signer.address(),
        round: 2,
        valid_round: None,
        valid_round_votes: Vec::new(),
        block: Block {
            header,
            body: BlockBody::default(),
            last_commit: vec![],
        },
        signature: Vec::new(),
    };
    let journal = SigningJournal::open(&path, genesis, &signer).unwrap();
    let signed = journal
        .sign_proposal(proposal.clone(), &signer, &validators)
        .unwrap();
    drop(journal);
    let reopened = SigningJournal::open(&path, genesis, &signer).unwrap();
    assert_eq!(
        reopened
            .sign_proposal(proposal.clone(), &signer, &validators)
            .unwrap(),
        signed
    );
    let mut conflict = proposal.clone();
    conflict.block.header.state_root = Hash32::ZERO;
    assert!(reopened
        .sign_proposal(conflict, &signer, &validators)
        .is_err());
    let mut old = proposal.clone();
    old.round = 1;
    assert_eq!(reopened.latest_proposal().unwrap().unwrap().round, 2);
    assert!(reopened.sign_proposal(old, &signer, &validators).is_err());
    let mut wrong_genesis = proposal;
    wrong_genesis.genesis = Hash32::ZERO;
    assert!(reopened
        .sign_proposal(wrong_genesis, &signer, &validators)
        .is_err());
    let old_vote = Vote::new(
        genesis,
        VoteType::Prevote,
        1,
        1,
        None,
        signer.address(),
        Vec::new(),
    );
    assert!(reopened.sign(old_vote, &signer).is_err());
    let current_vote = Vote::new(
        genesis,
        VoteType::Prevote,
        1,
        2,
        None,
        signer.address(),
        Vec::new(),
    );
    reopened.sign(current_vote, &signer).unwrap();
}

#[test]
fn durable_lock_allows_only_a_certified_conflicting_proposal() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("locked-proposal.redb");
    let keys: Vec<_> = (111..=114)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();
    let validators = ValidatorSet::new(
        keys.iter()
            .map(|key| Validator::new(key.address(), key.public_key_bytes().to_vec(), 1))
            .collect(),
    )
    .unwrap();
    let genesis = Hash32::new([11; 32]);
    let signer = &keys[0];
    let journal = SigningJournal::open(&path, genesis, signer).unwrap();

    let mut locked_header = BlockHeader::genesis("sprax-testnet-1", Hash32::new([21; 32]));
    locked_header.height = 4;
    locked_header.parent_hash = Hash32::new([22; 32]);
    locked_header.proposer = signer.address();
    locked_header.validator_set_hash = validators.commitment().unwrap();
    let locked_block = Block {
        header: locked_header,
        body: BlockBody::default(),
        last_commit: vec![],
    };
    let locked_hash = Hasher::block_hash(&locked_block.header).unwrap();
    journal
        .sign_with_block(
            vote(signer, 4, 0, VoteType::Precommit, Some(locked_hash)),
            signer,
            Some(locked_block),
            None,
        )
        .unwrap();

    let mut candidate_header = BlockHeader::genesis("sprax-testnet-1", Hash32::new([23; 32]));
    candidate_header.height = 4;
    candidate_header.parent_hash = Hash32::new([22; 32]);
    candidate_header.proposer = signer.address();
    candidate_header.validator_set_hash = validators.commitment().unwrap();
    let candidate = Block {
        header: candidate_header,
        body: BlockBody::default(),
        last_commit: vec![],
    };
    let candidate_hash = Hasher::block_hash(&candidate.header).unwrap();
    let bare_proposal = SignedProposal {
        genesis,
        signer: signer.address(),
        round: 2,
        valid_round: None,
        valid_round_votes: vec![],
        block: candidate.clone(),
        signature: vec![],
    };
    assert!(journal
        .sign_proposal(bare_proposal, signer, &validators)
        .is_err());

    let votes: Vec<_> = keys[1..]
        .iter()
        .map(|key| {
            let mut vote = vote(key, 4, 1, VoteType::Prevote, Some(candidate_hash));
            vote.signature = key.sign(&vote.sign_bytes().unwrap());
            vote
        })
        .collect();
    let proposal = SignedProposal {
        genesis,
        signer: signer.address(),
        round: 2,
        valid_round: Some(1),
        valid_round_votes: votes,
        block: candidate,
        signature: vec![],
    };
    let signed = journal
        .sign_proposal(proposal, signer, &validators)
        .unwrap();
    signed
        .verify(genesis, signer.address(), &validators)
        .unwrap();
    assert_eq!(journal.latest_proposal().unwrap(), Some(signed));
    assert_eq!(
        journal.state().unwrap().unwrap().locked_block,
        Some(locked_hash)
    );
}
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_node::signing_journal::SigningJournal;
use sprax_types::Hash32;

fn vote(
    key: &Ed25519Keypair,
    height: u64,
    round: u32,
    kind: VoteType,
    hash: Option<Hash32>,
) -> Vote {
    Vote::new(
        Hash32::new([11; 32]),
        kind,
        height,
        round,
        hash,
        key.address(),
        vec![],
    )
}

#[test]
fn durable_signer_replays_identical_votes_and_refuses_conflicts_after_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("signer.redb");
    let key = Ed25519Keypair::generate();
    let identity = Hash32::new([11; 32]);
    let journal = SigningJournal::open(&path, identity, &key).unwrap();
    let mut header = BlockHeader::genesis("sprax-testnet-1", Hash32::new([14; 32]));
    header.height = 7;
    header.parent_hash = Hash32::new([15; 32]);
    let block_data = Block {
        header,
        body: BlockBody::default(),
        last_commit: vec![],
    };
    let block = Hasher::block_hash(&block_data.header).unwrap();
    let first = vote(&key, 7, 2, VoteType::Prevote, Some(block));
    let signed = journal.sign(first.clone(), &key).unwrap();
    Ed25519Keypair::verify(
        &key.public_key_bytes(),
        &signed.sign_bytes().unwrap(),
        &signed.signature,
    )
    .unwrap();
    drop(journal);
    let journal = SigningJournal::open(&path, identity, &key).unwrap();
    assert_eq!(journal.sign(first, &key).unwrap(), signed);
    assert!(journal
        .sign(vote(&key, 7, 2, VoteType::Prevote, None), &key)
        .is_err());
    assert!(journal
        .sign(vote(&key, 7, 1, VoteType::Precommit, Some(block)), &key)
        .is_err());
    journal
        .sign_with_block(
            vote(&key, 7, 2, VoteType::Precommit, Some(block)),
            &key,
            Some(block_data.clone()),
            None,
        )
        .unwrap();
    drop(journal);
    let journal = SigningJournal::open(&path, identity, &key).unwrap();
    assert_eq!(journal.state().unwrap().unwrap().locked_block, Some(block));
    assert_eq!(
        journal.state().unwrap().unwrap().locked_block_data,
        Some(block_data.clone())
    );
    let mut conflicting_block = block_data.clone();
    conflicting_block.header.state_root = Hash32::ZERO;
    let conflicting_hash = Hasher::block_hash(&conflicting_block.header).unwrap();
    assert!(journal
        .sign_with_block(
            vote(&key, 7, 3, VoteType::Precommit, Some(conflicting_hash)),
            &key,
            Some(conflicting_block),
            None,
        )
        .is_err());
    journal
        .sign(vote(&key, 7, 3, VoteType::Prevote, None), &key)
        .unwrap();
    journal
        .sign(
            vote(&key, 8, 0, VoteType::Prevote, Some(Hash32::ZERO)),
            &key,
        )
        .unwrap();
    assert!(journal.state().unwrap().unwrap().locked_block.is_none());
    drop(journal);
    assert!(SigningJournal::open(&path, Hash32::ZERO, &key).is_err());
    assert!(SigningJournal::open(&path, identity, &Ed25519Keypair::generate()).is_err());
    std::fs::remove_file(&path).unwrap();
    assert!(SigningJournal::open(&path, identity, &key).is_err());
}

#[test]
fn durable_lock_changes_only_with_later_signed_prevote_quorum() {
    use sprax_consensus::{Validator, ValidatorSet};
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("unlock.redb");
    let keys: Vec<_> = (101..=104)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();
    let validators = ValidatorSet::new(
        keys.iter()
            .map(|key| Validator::new(key.address(), key.public_key_bytes().to_vec(), 1))
            .collect(),
    )
    .unwrap();
    let genesis = Hash32::new([11; 32]);
    let signer = &keys[0];
    let journal = SigningJournal::open(&path, genesis, signer).unwrap();
    let mut old_header = BlockHeader::genesis("sprax-testnet-1", Hash32::new([16; 32]));
    old_header.height = 7;
    old_header.parent_hash = Hash32::new([15; 32]);
    old_header.validator_set_hash = validators.commitment().unwrap();
    let old_block = Block {
        header: old_header,
        body: BlockBody::default(),
        last_commit: vec![],
    };
    let old_hash = Hasher::block_hash(&old_block.header).unwrap();
    journal
        .sign_with_block(
            vote(signer, 7, 0, VoteType::Precommit, Some(old_hash)),
            signer,
            Some(old_block),
            None,
        )
        .unwrap();

    let mut candidate_header = BlockHeader::genesis("sprax-testnet-1", Hash32::new([17; 32]));
    candidate_header.height = 7;
    candidate_header.parent_hash = Hash32::new([15; 32]);
    candidate_header.validator_set_hash = validators.commitment().unwrap();
    let candidate = Block {
        header: candidate_header,
        body: BlockBody::default(),
        last_commit: vec![],
    };
    let candidate_hash = Hasher::block_hash(&candidate.header).unwrap();
    let sign_prevote = |key: &Ed25519Keypair| {
        let mut proof_vote = vote(key, 7, 1, VoteType::Prevote, Some(candidate_hash));
        proof_vote.signature = key.sign(&proof_vote.sign_bytes().unwrap());
        proof_vote
    };
    let insufficient = [sign_prevote(&keys[1]), sign_prevote(&keys[2])];
    assert!(journal
        .sign_with_block(
            vote(signer, 7, 1, VoteType::Precommit, Some(candidate_hash)),
            signer,
            Some(candidate.clone()),
            Some((&validators, &insufficient)),
        )
        .is_err());
    assert_eq!(
        journal.state().unwrap().unwrap().locked_block,
        Some(old_hash)
    );

    let certificate = [
        {
            let mut vote = vote(&keys[1], 7, 1, VoteType::Prevote, Some(candidate_hash));
            vote.signature = keys[1].sign(&vote.sign_bytes().unwrap());
            vote
        },
        {
            let mut vote = vote(&keys[2], 7, 1, VoteType::Prevote, Some(candidate_hash));
            vote.signature = keys[2].sign(&vote.sign_bytes().unwrap());
            vote
        },
        {
            let mut vote = vote(&keys[3], 7, 1, VoteType::Prevote, Some(candidate_hash));
            vote.signature = keys[3].sign(&vote.sign_bytes().unwrap());
            vote
        },
    ];
    journal
        .sign_with_block(
            vote(signer, 7, 2, VoteType::Precommit, Some(candidate_hash)),
            signer,
            Some(candidate.clone()),
            Some((&validators, &certificate)),
        )
        .unwrap();
    let recovered = journal.state().unwrap().unwrap();
    assert_eq!(recovered.locked_block, Some(candidate_hash));
    assert_eq!(recovered.locked_round, Some(2));
    assert_eq!(recovered.locked_block_data, Some(candidate));
}

#[test]
fn competing_signers_cannot_release_two_conflicting_votes() {
    let directory = tempfile::tempdir().unwrap();
    let key = Ed25519Keypair::generate();
    let journal = SigningJournal::open(
        &directory.path().join("signer.redb"),
        Hash32::new([11; 32]),
        &key,
    )
    .unwrap();
    let results = std::thread::scope(|scope| {
        let a = scope.spawn(|| {
            journal.sign(
                vote(&key, 1, 0, VoteType::Prevote, Some(Hash32::ZERO)),
                &key,
            )
        });
        let b = scope.spawn(|| {
            journal.sign(
                vote(&key, 1, 0, VoteType::Prevote, Some(Hash32::new([1; 32]))),
                &key,
            )
        });
        [a.join().unwrap(), b.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
}

#[test]
fn wrong_genesis_votes_do_not_advance_durable_signing_state() {
    let directory = tempfile::tempdir().unwrap();
    let key = Ed25519Keypair::generate();
    let identity = Hash32::new([11; 32]);
    let journal =
        SigningJournal::open(&directory.path().join("signer.redb"), identity, &key).unwrap();
    let mut wrong = vote(&key, 10, 0, VoteType::Prevote, None);
    wrong.genesis = Hash32::new([99; 32]);
    assert!(journal.sign(wrong, &key).is_err());
    assert!(journal.state().unwrap().is_none());
    journal
        .sign(vote(&key, 1, 0, VoteType::Prevote, None), &key)
        .unwrap();
    assert_eq!(journal.state().unwrap().unwrap().vote.genesis, identity);
}

#[test]
fn legacy_signing_identity_is_rejected_without_resetting_history() {
    use redb::{Database, TableDefinition};
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("legacy.redb");
    let key = Ed25519Keypair::generate();
    let genesis = Hash32::new([11; 32]);
    let encoded = serde_json::to_vec(&serde_json::json!({
        "genesis": genesis,
        "public_key": key.public_key_bytes().to_vec(),
    }))
    .unwrap();
    {
        let db = Database::create(&path).unwrap();
        let write = db.begin_write().unwrap();
        {
            let mut table = write
                .open_table(TableDefinition::<&str, &[u8]>::new("signing"))
                .unwrap();
            table.insert("identity", encoded.as_slice()).unwrap();
            table
                .insert("state", b"legacy history must remain intact".as_slice())
                .unwrap();
        }
        write.commit().unwrap();
    }
    let marker = path.with_extension("initialized");
    std::fs::write(&marker, &encoded).unwrap();
    assert!(SigningJournal::open(&path, genesis, &key).is_err());
    assert_eq!(std::fs::read(marker).unwrap(), encoded);
    let db = Database::open(&path).unwrap();
    let read = db.begin_read().unwrap();
    let table = read
        .open_table(TableDefinition::<&str, &[u8]>::new("signing"))
        .unwrap();
    assert_eq!(
        table.get("state").unwrap().unwrap().value(),
        b"legacy history must remain intact"
    );
}
