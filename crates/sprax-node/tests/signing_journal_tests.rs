use sprax_consensus::{Vote, VoteType};
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
    Vote::new(kind, height, round, hash, key.address(), vec![])
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
    let journal =
        SigningJournal::open(&directory.path().join("signer.redb"), Hash32::ZERO, &key).unwrap();
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
