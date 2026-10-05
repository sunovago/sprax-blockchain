use sprax_core::{ChainLedger, GenesisAccount, GenesisConfig, GenesisValidator};
use sprax_crypto::Ed25519Keypair;
use sprax_types::{Amount, ChainId, KeyType, Transaction, TxBody, TxFee, TxMessage};

fn genesis() -> (GenesisConfig, Ed25519Keypair, Ed25519Keypair) {
    let alice = Ed25519Keypair::from_seed(&[42; 32]);
    let bob = Ed25519Keypair::from_seed(&[43; 32]);
    let mut g = GenesisConfig::default_development();
    g.accounts = [&alice, &bob]
        .iter()
        .map(|k| GenesisAccount {
            name: "validator".into(),
            address: k.address(),
            initial_balance: Amount::from_sprx_whole(1000).unwrap(),
        })
        .collect();
    g.validators = [(&alice, 100), (&bob, 50)]
        .into_iter()
        .map(|(k, power)| GenesisValidator {
            operator_address: k.address(),
            consensus_pubkey: k.public_key_bytes().to_vec(),
            self_stake: Amount::from_sprx_whole(power).unwrap(),
            moniker: "validator".into(),
        })
        .collect();
    (g, alice, bob)
}

#[test]
fn proposals_commit_preexecution_power_and_reject_wrong_commitments_atomically() {
    let (g, alice, bob) = genesis();
    let mut source = ChainLedger::init_from_genesis(g.clone()).unwrap();
    let mut replica = ChainLedger::init_from_genesis(g).unwrap();
    let initial = source.active_validator_set_hash().unwrap();
    assert_eq!(initial, source.latest_header().validator_set_hash);
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
    let tx = Transaction::new(
        body,
        KeyType::Ed25519,
        bob.public_key_bytes().to_vec(),
        signature,
    )
    .unwrap();
    source.submit_transaction(tx).unwrap();
    let proposed = source.build_proposal(alice.address()).unwrap();
    assert_eq!(proposed.header.validator_set_hash, initial);
    assert_eq!(source.active_validator_set_hash().unwrap(), initial);
    assert_eq!(source.height(), 0);
    let mut tampered = proposed.clone();
    tampered.header.validator_set_hash = sprax_types::Hash32::ZERO;
    let root = replica.state_root().unwrap();
    assert!(replica.apply_block(tampered).is_err());
    assert_eq!(replica.height(), 0);
    assert_eq!(replica.state_root().unwrap(), root);
    replica.apply_block(proposed).unwrap();
    let finalized = source.mine_block(alice.address()).unwrap();
    assert_eq!(finalized.header.validator_set_hash, initial);
    assert_eq!(
        source.active_validator_set_hash().unwrap(),
        replica.active_validator_set_hash().unwrap()
    );
    assert_ne!(source.active_validator_set_hash().unwrap(), initial);
    let next = source.mine_block(alice.address()).unwrap();
    assert_eq!(
        next.header.validator_set_hash,
        replica.active_validator_set_hash().unwrap()
    );
    replica.apply_block(next).unwrap();
    assert_eq!(source.state_root().unwrap(), replica.state_root().unwrap());
}

#[test]
fn restart_rejects_legacy_genesis_commitments_and_unfinalized_state_mutations() {
    use sprax_storage::{ChainMetaStore, KVStore, MemKVStore};
    let (g, alice, _) = genesis();
    let store = MemKVStore::new();
    let mut ledger = ChainLedger::open_or_init(store.clone(), || Ok(g.clone())).unwrap();
    let mut old = ledger.get_block_by_height(0).unwrap().clone();
    old.header.validator_set_hash =
        sprax_crypto::Hasher::sha256(&serde_json::to_vec(&g.validators).unwrap());
    store
        .put_block(0, &serde_json::to_vec(&old).unwrap())
        .unwrap();
    assert!(ChainLedger::open_or_init(store.clone(), || Ok(g.clone())).is_err());
    let original = ledger.get_block_by_height(0).unwrap();
    store
        .put_block(0, &serde_json::to_vec(original).unwrap())
        .unwrap();
    ledger.mine_block(alice.address()).unwrap();
    drop(ledger);
    store
        .set(b"unfinalized/tamper", b"state changed outside a block")
        .unwrap();
    assert!(ChainLedger::open_or_init(store, || Ok(g)).is_err());
}
