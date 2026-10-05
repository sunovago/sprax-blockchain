use sprax_consensus::SignedProposal;
use sprax_consensus::{Vote, VoteType};
use sprax_types::{Block, BlockBody, BlockHeader};

#[test]
fn proposal_signatures_survive_restarts_and_conflicting_retries_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("proposal-signing.redb");
    let signer = Ed25519Keypair::generate();
    let genesis = Hash32::new([61; 32]);
    let mut header = BlockHeader::genesis("sprax-testnet-1", Hash32::new([62; 32]));
    header.proposer = signer.address();
    header.height = 1;
    header.parent_hash = Hash32::new([63; 32]);
    let proposal = SignedProposal {
        genesis,
        signer: signer.address(),
        round: 2,
        block: Block {
            header,
            body: BlockBody::default(),
            last_commit: vec![],
        },
        signature: Vec::new(),
    };
    let journal = SigningJournal::open(&path, genesis, &signer).unwrap();
    let signed = journal.sign_proposal(proposal.clone(), &signer).unwrap();
    drop(journal);
    let reopened = SigningJournal::open(&path, genesis, &signer).unwrap();
    assert_eq!(
        reopened.sign_proposal(proposal.clone(), &signer).unwrap(),
        signed
    );
    let mut conflict = proposal.clone();
    conflict.block.header.state_root = Hash32::ZERO;
    assert!(reopened.sign_proposal(conflict, &signer).is_err());
    let mut old = proposal.clone();
    old.round = 1;
    assert_eq!(reopened.latest_proposal().unwrap().unwrap().round, 2);
    assert!(reopened.sign_proposal(old, &signer).is_err());
    let mut wrong_genesis = proposal;
    wrong_genesis.genesis = Hash32::ZERO;
    assert!(reopened.sign_proposal(wrong_genesis, &signer).is_err());
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
use sprax_crypto::Ed25519Keypair;
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
    let block = Hash32::new([12; 32]);
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
        .sign(vote(&key, 7, 2, VoteType::Precommit, Some(block)), &key)
        .unwrap();
    drop(journal);
    let journal = SigningJournal::open(&path, identity, &key).unwrap();
    assert_eq!(journal.state().unwrap().unwrap().locked_block, Some(block));
    assert!(journal
        .sign(
            vote(&key, 7, 3, VoteType::Prevote, Some(Hash32::ZERO)),
            &key
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
