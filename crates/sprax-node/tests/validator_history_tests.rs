use sprax_consensus::{ValidatorSet, Vote, VoteType};
use sprax_core::{ChainLedger, GenesisAccount, GenesisConfig, GenesisValidator};
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_node::NodeService;
use sprax_types::{
    Amount, Block, ChainId, CommitSignature, KeyType, Transaction, TxBody, TxFee, TxMessage,
};

fn certify(block: &mut Block, keys: &[&Ed25519Keypair], genesis: sprax_types::Hash32) {
    let hash = Hasher::block_hash(&block.header).unwrap();
    block.last_commit = keys
        .iter()
        .map(|key| {
            let vote = Vote::new(
                genesis,
                VoteType::Precommit,
                block.header.height,
                0,
                Some(hash),
                key.address(),
                vec![],
            );
            CommitSignature {
                round: 0,
                validator_address: key.address(),
                signature: key.sign(&vote.sign_bytes().unwrap()),
                timestamp_unix_secs: 0,
            }
        })
        .collect();
}

#[test]
fn historical_catchup_uses_each_committed_stake_transition_and_ignores_local_cache() {
    let alice = Ed25519Keypair::from_seed(&[42; 32]);
    let bob = Ed25519Keypair::from_seed(&[43; 32]);
    let mut genesis = GenesisConfig::default_development();
    genesis.accounts = [&alice, &bob]
        .iter()
        .map(|key| GenesisAccount {
            name: "validator".into(),
            address: key.address(),
            initial_balance: Amount::from_sprx_whole(1000).unwrap(),
        })
        .collect();
    genesis.validators = [(&alice, 100), (&bob, 50)]
        .into_iter()
        .map(|(key, stake)| GenesisValidator {
            operator_address: key.address(),
            consensus_pubkey: key.public_key_bytes().to_vec(),
            self_stake: Amount::from_sprx_whole(stake).unwrap(),
            moniker: "validator".into(),
        })
        .collect();
    let dir = tempfile::tempdir().unwrap();
    genesis
        .save_to_file(&dir.path().join("genesis.json"))
        .unwrap();
    std::fs::create_dir_all(dir.path().join("data")).unwrap();
    std::fs::write(
        dir.path().join("data/staking.json"),
        b"corrupt untrusted cache",
    )
    .unwrap();
    let node = NodeService::new_or_load(dir.path().to_path_buf()).unwrap();
    let identity = genesis.fingerprint().unwrap();
    let mut producer = ChainLedger::init_from_genesis(genesis).unwrap();
    let old = ValidatorSet::from_canonical(producer.active_validators().unwrap()).unwrap();
    let body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: bob.address(),
        nonce: 0,
        messages: vec![TxMessage::Delegate {
            validator: alice.address(),
            amount: Amount::from_sprx_whole(100).unwrap(),
        }],
        fee: TxFee::default(),
        memo: String::new(),
        timeout_height: 10,
    };
    let signature = bob.sign(&body.sign_bytes().unwrap());
    producer
        .submit_transaction(
            Transaction::new(
                body,
                KeyType::Ed25519,
                bob.public_key_bytes().to_vec(),
                signature,
            )
            .unwrap(),
        )
        .unwrap();
    let mut first = producer.mine_block(alice.address()).unwrap();
    certify(&mut first, &[&alice, &bob], identity);
    node.apply_block(first.clone()).unwrap();
    assert!(node.apply_block(first.clone()).unwrap().is_empty());
    let mut conflict = first;
    conflict.header.timestamp_unix_secs += 1;
    assert!(node.apply_block(conflict).is_err());
    // A stale/mutated display cache cannot change the next certificate's power.
    node.staking()
        .write()
        .sync_validator_tokens(&alice.address(), Amount::from_sprx_whole(1).unwrap());
    let mut second = producer.mine_block(alice.address()).unwrap();
    certify(&mut second, &[&alice], identity);
    assert!(sprax_consensus::verify_block_commit(&second, &old, identity).is_err());
    node.apply_block(second).unwrap();
    assert_eq!(node.height(), 2);
    assert_eq!(
        node.latest_header().state_root,
        producer.state_root().unwrap()
    );
    assert_eq!(
        node.canonical_validator_set().unwrap().total_voting_power(),
        250
    );
    drop(node);
    let reopened = NodeService::new_or_load(dir.path().to_path_buf()).unwrap();
    assert_eq!(reopened.height(), 2);
    assert_eq!(
        reopened
            .canonical_validator_set()
            .unwrap()
            .total_voting_power(),
        250
    );
    assert_eq!(
        std::fs::read(dir.path().join("data/staking.json")).unwrap(),
        b"corrupt untrusted cache"
    );
}
