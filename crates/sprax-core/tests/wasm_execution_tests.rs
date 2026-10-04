use sprax_core::{ChainLedger, GenesisConfig};
use sprax_crypto::Ed25519Keypair;
use sprax_storage::MemKVStore;
use sprax_types::{Address, ChainId, Hash32, KeyType, Transaction, TxBody, TxFee, TxMessage};

fn fixture() -> Vec<u8> {
    let path = std::env::var("SPRX_TEST_CONTRACT_WASM").expect("build contracts/examples/counter for wasm32-unknown-unknown and set SPRX_TEST_CONTRACT_WASM to sprax_counter.wasm");
    std::fs::read(path).expect("read compiled contract fixture")
}
fn transaction(key: &Ed25519Keypair, nonce: u64, message: TxMessage) -> Transaction {
    let body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: key.address(),
        nonce,
        messages: vec![message],
        fee: TxFee {
            gas_limit: 5_000_000,
            ..TxFee::default()
        },
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
fn real_wasm_executes_replicates_rolls_back_and_survives_restart() {
    let key = Ed25519Keypair::from_seed(&[88; 32]);
    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = key.address();
    let store = MemKVStore::new();
    let mut producer =
        ChainLedger::init_from_genesis_with_store(genesis.clone(), store.clone()).unwrap();
    let mut follower = ChainLedger::init_from_genesis(genesis.clone()).unwrap();
    let mut nonce = 0;
    let mut send = |ledger: &mut ChainLedger, follower: &mut ChainLedger, message: TxMessage| {
        let tx = transaction(&key, nonce, message);
        nonce += 1;
        let hash = ledger.submit_transaction(tx).unwrap();
        let block = ledger.mine_block(key.address()).unwrap();
        assert_eq!(
            block.body.transactions.len(),
            1,
            "contract transaction must actually execute"
        );
        follower.apply_block(block).unwrap();
        assert_eq!(ledger.state_root().unwrap(), follower.state_root().unwrap());
        let receipt = ledger.get_transaction(&hash).unwrap().1;
        assert!(receipt.gas_used > 41_000, "WASM execution must consume gas");
        receipt.return_data.clone()
    };
    let code: Hash32 = serde_json::from_slice(&send(
        &mut producer,
        &mut follower,
        TxMessage::StoreCode {
            wasm_bytecode: fixture(),
        },
    ))
    .unwrap();
    let address: Address = serde_json::from_slice(&send(
        &mut producer,
        &mut follower,
        TxMessage::InstantiateContract {
            code_id: code,
            msg: br#"{"count":7}"#.to_vec(),
            funds: sprax_types::Amount::ZERO,
            label: "counter".into(),
        },
    ))
    .unwrap();
    send(
        &mut producer,
        &mut follower,
        TxMessage::ContractCall {
            contract: address,
            data: br#"{"increment":{}}"#.to_vec(),
            funds: sprax_types::Amount::ZERO,
        },
    );
    let result = producer
        .query_contract(address, br#"{"count":{}}"#, 200_000)
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&result.data).unwrap()["count"],
        8
    );
    assert!(result.gas_used > 0);
    let root = producer.state_root().unwrap();
    let failed = transaction(
        &key,
        3,
        TxMessage::ContractCall {
            contract: address,
            data: br#"{"write_then_fail":{}}"#.to_vec(),
            funds: sprax_types::Amount::ZERO,
        },
    );
    producer.submit_transaction(failed).unwrap();
    let proposed = producer.build_proposal(key.address()).unwrap();
    assert!(proposed.body.transactions.is_empty());
    assert_eq!(producer.state_root().unwrap(), root);
    assert_eq!(producer.get_account(&key.address()).unwrap().nonce, 3);
    let restarted = ChainLedger::open_or_init(store, || Ok(genesis)).unwrap();
    let result = restarted
        .query_contract(address, br#"{"count":{}}"#, 200_000)
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&result.data).unwrap()["count"],
        8
    );
}
