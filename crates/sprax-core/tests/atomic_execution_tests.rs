use sprax_core::{executor::TxExecutor, ChainLedger, GenesisConfig, StateAccessor};
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_storage::{ChainMetaStore, MemKVStore, StateCommitment};
use sprax_types::{
    Address, Amount, ChainId, Hash32, KeyType, Transaction, TxBody, TxFee, TxMessage,
};

fn fixture() -> (GenesisConfig, Ed25519Keypair) {
    let key = Ed25519Keypair::from_seed(&[91; 32]);
    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = key.address();
    (genesis, key)
}

fn signed(key: &Ed25519Keypair, messages: Vec<TxMessage>) -> Transaction {
    let body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: key.address(),
        nonce: 0,
        messages,
        fee: TxFee::default(),
        memo: String::new(),
        timeout_height: 100,
    };
    let signature = key.sign(&body.sign_bytes().unwrap());
    Transaction::new(
        body,
        KeyType::Ed25519,
        key.public_key_bytes().to_vec(),
        signature,
    )
    .unwrap()
}

#[test]
fn mempool_capacity_failure_preserves_chain_and_pending_transactions() {
    let mut genesis = GenesisConfig::default_development();
    let keys: Vec<_> = (0..129).map(|_| Ed25519Keypair::generate()).collect();
    for key in &keys {
        genesis.accounts.push(sprax_core::GenesisAccount {
            name: "capacity test".into(),
            address: key.address(),
            initial_balance: Amount::from_sprx_whole(1).unwrap(),
        });
    }
    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();
    let root = ledger.state_root().unwrap();
    for key in &keys[..128] {
        ledger
            .submit_transaction(signed(
                key,
                vec![TxMessage::Transfer {
                    to: Address::ZERO,
                    amount: Amount::from_atto(1),
                }],
            ))
            .unwrap();
    }
    assert_eq!(ledger.mempool_len(), 128);
    let rejected = signed(
        &keys[128],
        vec![TxMessage::Transfer {
            to: Address::ZERO,
            amount: Amount::from_atto(1),
        }],
    );
    assert!(ledger
        .submit_transaction(rejected)
        .unwrap_err()
        .to_string()
        .contains("mempool is full"));
    assert_eq!(ledger.mempool_len(), 128);
    assert_eq!(ledger.height(), 0);
    assert_eq!(ledger.state_root().unwrap(), root);
}

#[test]
fn rejected_block_preserves_state_indexes_height_and_mempool() {
    let (genesis, key) = fixture();
    let store = MemKVStore::new();
    let mut follower =
        ChainLedger::init_from_genesis_with_store(genesis.clone(), store.clone()).unwrap();
    let mut producer = ChainLedger::init_from_genesis(genesis).unwrap();
    let tx = signed(
        &key,
        vec![TxMessage::Transfer {
            to: Address::new([92; 20]),
            amount: Amount::from_atto(10),
        }],
    );
    let hash = Hasher::tx_hash(&tx).unwrap();
    follower.submit_transaction(tx.clone()).unwrap();
    producer.submit_transaction(tx).unwrap();
    let mut block = producer.mine_block(key.address()).unwrap();
    block.header.state_root = Hash32::ZERO;
    let root = follower.state_root().unwrap();
    assert!(follower.apply_block(block).is_err());
    assert_eq!(follower.state_root().unwrap(), root);
    assert_eq!(follower.height(), 0);
    assert_eq!(follower.mempool_len(), 1);
    assert!(follower.get_transaction(&hash).is_none());
    assert!(store.get_tx_index(hash).unwrap().is_none());
    assert!(store.get_block(1).unwrap().is_none());
    assert_eq!(store.get_height().unwrap(), Some(0));
}

#[test]
fn proposal_build_and_validation_do_not_commit() {
    let (genesis, key) = fixture();
    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();
    let tx = signed(
        &key,
        vec![TxMessage::Transfer {
            to: Address::new([92; 20]),
            amount: Amount::from_atto(10),
        }],
    );
    ledger.submit_transaction(tx).unwrap();
    let root = ledger.state_root().unwrap();
    let block = ledger.build_proposal(key.address()).unwrap();
    ledger.validate_proposal(block.clone()).unwrap();
    assert_eq!(ledger.height(), 0);
    assert_eq!(ledger.state_root().unwrap(), root);
    assert_eq!(ledger.mempool_len(), 1);
    ledger.apply_block(block).unwrap();
    assert_eq!(ledger.height(), 1);
    assert_eq!(ledger.mempool_len(), 0);
}

#[test]
fn repeated_unbond_failure_does_not_partially_mutate_state() {
    let (mut genesis, key) = fixture();
    let validator = Address::new([93; 20]);
    genesis.accounts[0].initial_balance = Amount::from_sprx_whole(1000).unwrap();
    let store = MemKVStore::new();
    genesis.initialize_state(&store).unwrap();
    StateAccessor::set_delegation(
        &store,
        &key.address(),
        &validator,
        &sprax_core::state::DelegationState {
            shares: Amount::from_atto(10),
            balance: Amount::from_atto(10),
        },
    )
    .unwrap();
    StateAccessor::set_validator_stake(
        &store,
        &validator,
        &sprax_core::state::ValidatorStakeState {
            tokens: Amount::from_atto(10),
        },
    )
    .unwrap();
    let tx = signed(
        &key,
        vec![
            TxMessage::Unbond {
                validator,
                amount: Amount::from_atto(7)
            };
            2
        ],
    );
    let root = store.compute_root().unwrap();
    assert!(TxExecutor::default()
        .execute_transaction(&store, &tx, 1, "sprax-devnet-1", 10)
        .is_err());
    assert_eq!(store.compute_root().unwrap(), root);
    assert_eq!(
        StateAccessor::get_account(&store, &key.address())
            .unwrap()
            .nonce,
        0
    );
}

#[test]
fn unsupported_messages_are_rejected_without_fee_or_nonce_changes() {
    let (genesis, key) = fixture();
    let store = MemKVStore::new();
    genesis.initialize_state(&store).unwrap();
    let tx = signed(
        &key,
        vec![TxMessage::Generic {
            type_url: "unknown".into(),
            payload: vec![],
        }],
    );
    let root = store.compute_root().unwrap();
    assert!(TxExecutor::default()
        .execute_transaction(&store, &tx, 1, "sprax-devnet-1", 10)
        .is_err());
    assert_eq!(store.compute_root().unwrap(), root);
}
