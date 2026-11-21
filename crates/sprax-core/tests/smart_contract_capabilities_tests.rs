use sprax_core::{ChainLedger, GenesisConfig};
use sprax_crypto::Ed25519Keypair;
use sprax_storage::RedbStore;
use sprax_types::{
    Address, Amount, ChainId, Hash32, KeyType, Transaction, TxBody, TxFee, TxMessage, TxReceipt,
};
use sprax_wasm::cosmwasm_std::Binary;

fn fixture() -> Vec<u8> {
    let path = std::env::var_os("SPRX_TEST_CONTRACT_WASM")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
                "../../contracts/examples/counter/target/wasm32-unknown-unknown/release/sprax_counter.wasm",
            )
        });
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("cannot read WASM fixture {}: {error}", path.display()))
}

fn make_tx(key: &Ed25519Keypair, nonce: u64, message: TxMessage) -> Transaction {
    let body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: key.address(),
        nonce,
        messages: vec![message],
        fee: TxFee {
            gas_limit: 10_000_000,
            ..TxFee::default()
        },
        memo: String::new(),
        timeout_height: 1000,
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

fn apply_tx(
    producer: &mut ChainLedger<RedbStore>,
    follower: &mut ChainLedger<RedbStore>,
    key: &Ed25519Keypair,
    nonce: &mut u64,
    message: TxMessage,
) -> (TxReceipt, bool) {
    let msg_desc = format!("{:?}", message);
    println!(
        "Applying tx nonce={} msg={}",
        *nonce,
        &msg_desc[..msg_desc.len().min(60)]
    );
    let tx = make_tx(key, *nonce, message);
    *nonce += 1;
    let hash = producer.submit_transaction(tx).unwrap();
    let block = producer.mine_block(key.address()).unwrap();
    assert_eq!(
        block.body.transactions.len(),
        1,
        "tx must be included in mined block"
    );
    follower.apply_block(block).unwrap();
    assert_eq!(
        producer.state_root().unwrap(),
        follower.state_root().unwrap(),
        "producer and follower state roots must match after block application"
    );
    let receipt = producer.get_transaction(&hash).unwrap().1;
    (receipt.clone(), true)
}

#[test]
fn test_comprehensive_smart_contract_capabilities() {
    let wasm = fixture();
    let alice = Ed25519Keypair::from_seed(&[11; 32]);
    let bob = Ed25519Keypair::from_seed(&[22; 32]);
    let mut alice_nonce = 0;
    let mut bob_nonce = 0;

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = alice.address();
    genesis.accounts.push(sprax_core::GenesisAccount {
        address: bob.address(),
        initial_balance: Amount::from_sprx_whole(10_000).unwrap(),
        name: "bob".into(),
    });

    let directory = tempfile::tempdir().unwrap();
    let producer_store = RedbStore::open(&directory.path().join("producer.redb")).unwrap();
    let follower_store = RedbStore::open(&directory.path().join("follower.redb")).unwrap();

    let mut producer =
        ChainLedger::init_from_genesis_with_store(genesis.clone(), producer_store).unwrap();
    let mut follower =
        ChainLedger::init_from_genesis_with_store(genesis.clone(), follower_store).unwrap();

    // 1. Upload Code
    let (receipt, executed) = apply_tx(
        &mut producer,
        &mut follower,
        &alice,
        &mut alice_nonce,
        TxMessage::StoreCode {
            wasm_bytecode: wasm.clone(),
        },
    );
    assert!(executed);
    let code_id: Hash32 = serde_json::from_slice(&receipt.return_data).unwrap();

    // 2. Instantiate Contract A with Admin = Alice and Attached Funds = 50,000 atto
    let (receipt_a, executed) = apply_tx(
        &mut producer,
        &mut follower,
        &alice,
        &mut alice_nonce,
        TxMessage::InstantiateContract {
            code_id,
            msg: serde_json::to_vec(&serde_json::json!({ "count": 10 })).unwrap(),
            funds: Amount::from_atto(50_000),
            label: "counter-a".into(),
            admin: Some(alice.address()),
        },
    );
    assert!(executed);
    let contract_a: Address = serde_json::from_slice(&receipt_a.return_data).unwrap();
    assert_eq!(
        producer.get_account(&contract_a).unwrap().balance,
        Amount::from_atto(50_000)
    );

    // 3. Instantiate Contract B with Admin = Alice and Attached Funds = 20,000 atto
    let (receipt_b, executed) = apply_tx(
        &mut producer,
        &mut follower,
        &alice,
        &mut alice_nonce,
        TxMessage::InstantiateContract {
            code_id,
            msg: serde_json::to_vec(&serde_json::json!({ "count": 100 })).unwrap(),
            funds: Amount::from_atto(20_000),
            label: "counter-b".into(),
            admin: Some(alice.address()),
        },
    );
    assert!(executed);
    let contract_b: Address = serde_json::from_slice(&receipt_b.return_data).unwrap();
    assert_eq!(
        producer.get_account(&contract_b).unwrap().balance,
        Amount::from_atto(20_000)
    );

    // 4. Contract Composition & Submessage with ReplyOn::Success
    // Contract A calls Contract B's Increment with reply_on: "success", reply_id: 42
    let submsg_call = serde_json::json!({
        "call_other": {
            "contract": contract_b.to_bech32().unwrap(),
            "msg": Binary::new(serde_json::to_vec(&serde_json::json!({ "increment": {} })).unwrap()),
            "funds": [],
            "reply_on": "success",
            "reply_id": 42
        }
    });
    let (call_receipt, executed) = apply_tx(
        &mut producer,
        &mut follower,
        &alice,
        &mut alice_nonce,
        TxMessage::ContractCall {
            contract: contract_a,
            data: serde_json::to_vec(&submsg_call).unwrap(),
            funds: Amount::ZERO,
        },
    );
    assert!(executed);
    assert!(call_receipt.success);

    // Verify Contract B count incremented from 100 to 101
    let b_query = producer
        .query_contract(contract_b, br#"{"count":{}}"#, 200_000)
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&b_query.data).unwrap()["count"],
        101
    );

    // Verify Contract A stored the reply result (id: 42, success: true)
    let a_reply_query = producer
        .query_contract(contract_a, br#"{"last_reply":{}}"#, 200_000)
        .unwrap();
    let reply_val: serde_json::Value = serde_json::from_slice(&a_reply_query.data).unwrap();
    assert_eq!(reply_val["id"], 42);
    assert_eq!(reply_val["success"], true);

    // 5. Contract Composition & Submessage with ReplyOn::Error (Atomic Child Rollback)
    // Contract A calls Contract B's WriteThenFail with reply_on: "error", reply_id: 99
    let submsg_err_call = serde_json::json!({
        "call_other": {
            "contract": contract_b.to_bech32().unwrap(),
            "msg": Binary::new(serde_json::to_vec(&serde_json::json!({ "write_then_fail": {} })).unwrap()),
            "funds": [],
            "reply_on": "error",
            "reply_id": 99
        }
    });
    let (err_call_receipt, executed) = apply_tx(
        &mut producer,
        &mut follower,
        &alice,
        &mut alice_nonce,
        TxMessage::ContractCall {
            contract: contract_a,
            data: serde_json::to_vec(&submsg_err_call).unwrap(),
            funds: Amount::ZERO,
        },
    );
    assert!(executed);
    assert!(err_call_receipt.success);

    // Verify Contract B state was rolled back (count remains 101, NOT 999)
    let b_query_after = producer
        .query_contract(contract_b, br#"{"count":{}}"#, 200_000)
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&b_query_after.data).unwrap()["count"],
        101
    );

    // Verify Contract A received error reply (id: 99, success: false)
    let a_reply_err = producer
        .query_contract(contract_a, br#"{"last_reply":{}}"#, 200_000)
        .unwrap();
    let reply_err_val: serde_json::Value = serde_json::from_slice(&a_reply_err.data).unwrap();
    assert_eq!(reply_err_val["id"], 99);
    assert_eq!(reply_err_val["success"], false);

    // 6. Chain Queries from Contract
    // Contract A queries Contract B's smart count
    let smart_query_call = serde_json::json!({
        "query_other_count": {
            "contract": contract_b.to_bech32().unwrap()
        }
    });
    apply_tx(
        &mut producer,
        &mut follower,
        &alice,
        &mut alice_nonce,
        TxMessage::ContractCall {
            contract: contract_a,
            data: serde_json::to_vec(&smart_query_call).unwrap(),
            funds: Amount::ZERO,
        },
    );
    let queried_other_count = producer
        .query_contract(contract_a, br#"{"queried_other_count":{}}"#, 200_000)
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&queried_other_count.data).unwrap()["count"],
        101
    );

    // Contract A queries Contract B's raw storage key b"count"
    let raw_query_call = serde_json::json!({
        "query_other_raw": {
            "contract": contract_b.to_bech32().unwrap(),
            "key": Binary::new(b"count".to_vec())
        }
    });
    apply_tx(
        &mut producer,
        &mut follower,
        &alice,
        &mut alice_nonce,
        TxMessage::ContractCall {
            contract: contract_a,
            data: serde_json::to_vec(&raw_query_call).unwrap(),
            funds: Amount::ZERO,
        },
    );
    let queried_raw = producer
        .query_contract(contract_a, br#"{"queried_raw":{}}"#, 200_000)
        .unwrap();
    let raw_val: serde_json::Value = serde_json::from_slice(&queried_raw.data).unwrap();
    assert!(raw_val["data"].is_string());

    // Contract A queries Contract B's contract info (admin)
    let contract_info_call = serde_json::json!({
        "query_contract_info": {
            "contract": contract_b.to_bech32().unwrap()
        }
    });
    apply_tx(
        &mut producer,
        &mut follower,
        &alice,
        &mut alice_nonce,
        TxMessage::ContractCall {
            contract: contract_a,
            data: serde_json::to_vec(&contract_info_call).unwrap(),
            funds: Amount::ZERO,
        },
    );
    let queried_admin = producer
        .query_contract(contract_a, br#"{"queried_admin":{}}"#, 200_000)
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&queried_admin.data).unwrap()["admin"],
        alice.address().to_bech32().unwrap()
    );

    // Contract A queries Code Info for code_id 1
    let code_info_call = serde_json::json!({
        "query_code_info": {
            "code_id": 1
        }
    });
    apply_tx(
        &mut producer,
        &mut follower,
        &alice,
        &mut alice_nonce,
        TxMessage::ContractCall {
            contract: contract_a,
            data: serde_json::to_vec(&code_info_call).unwrap(),
            funds: Amount::ZERO,
        },
    );
    let queried_code = producer
        .query_contract(contract_a, br#"{"queried_code_hash":{}}"#, 200_000)
        .unwrap();
    assert!(
        serde_json::from_slice::<serde_json::Value>(&queried_code.data).unwrap()["checksum"]
            .is_string()
    );

    // 7. Native Bank Sends from Contract
    // Contract A sends 15,000 atto to Bob via BankMsg::Send
    let send_funds_call = serde_json::json!({
        "send_bank": {
            "to": bob.address().to_bech32().unwrap(),
            "amount": 15_000u128,
            "denom": "asprx"
        }
    });
    let (send_receipt, executed) = apply_tx(
        &mut producer,
        &mut follower,
        &alice,
        &mut alice_nonce,
        TxMessage::ContractCall {
            contract: contract_a,
            data: serde_json::to_vec(&send_funds_call).unwrap(),
            funds: Amount::ZERO,
        },
    );
    assert!(executed);
    assert!(send_receipt.success);

    // Verify Bob received 15,000 atto and Contract A balance decreased by 15,000
    assert_eq!(
        producer.get_account(&bob.address()).unwrap().balance,
        Amount::from_sprx_whole(10_000)
            .unwrap()
            .checked_add(Amount::from_atto(15_000))
            .unwrap()
    );
    assert_eq!(
        producer.get_account(&contract_a).unwrap().balance,
        Amount::from_atto(35_000) // 50,000 - 15,000
    );

    // 8. Custom Event Emission
    let emit_event_call = serde_json::json!({
        "emit_custom_event": {
            "key": "metric_name",
            "value": "l1_testnet_ready"
        }
    });
    let (event_receipt, executed) = apply_tx(
        &mut producer,
        &mut follower,
        &alice,
        &mut alice_nonce,
        TxMessage::ContractCall {
            contract: contract_a,
            data: serde_json::to_vec(&emit_event_call).unwrap(),
            funds: Amount::ZERO,
        },
    );
    assert!(executed);
    let found_custom_event = event_receipt.events.iter().any(|e| {
        e.contract == contract_a
            && e.event_type == "custom_protocol_event"
            && e.attributes
                .iter()
                .any(|(k, v)| k == "metric_name" && v == "l1_testnet_ready")
    });
    assert!(
        found_custom_event,
        "custom event must be present in receipt events"
    );

    // 9. Authorized Migrations & Admin Updates
    // Bob attempts to migrate Contract A -> must fail because Bob is not admin
    let unauthorized_migrate = make_tx(
        &bob,
        bob_nonce,
        TxMessage::MigrateContract {
            contract: contract_a,
            new_code_id: code_id,
            msg: serde_json::to_vec(&serde_json::json!({ "new_count": 888 })).unwrap(),
        },
    );
    let _ = producer.submit_transaction(unauthorized_migrate);
    let empty_block_1 = producer.mine_block(alice.address()).unwrap();
    assert!(
        empty_block_1.body.transactions.is_empty(),
        "unauthorized migration must be rejected"
    );
    follower.apply_block(empty_block_1).unwrap();

    // Alice updates admin to Bob
    let (admin_receipt, executed) = apply_tx(
        &mut producer,
        &mut follower,
        &alice,
        &mut alice_nonce,
        TxMessage::UpdateAdmin {
            contract: contract_a,
            new_admin: Some(bob.address()),
        },
    );
    assert!(executed);
    assert!(admin_receipt.success);

    // Alice now attempts to migrate -> must fail because Alice is no longer admin
    let alice_migrate = make_tx(
        &alice,
        alice_nonce,
        TxMessage::MigrateContract {
            contract: contract_a,
            new_code_id: code_id,
            msg: serde_json::to_vec(&serde_json::json!({ "new_count": 999 })).unwrap(),
        },
    );
    let _ = producer.submit_transaction(alice_migrate);
    let empty_block_2 = producer.mine_block(alice.address()).unwrap();
    assert!(
        empty_block_2.body.transactions.is_empty(),
        "previous admin cannot migrate after admin update"
    );
    follower.apply_block(empty_block_2).unwrap();

    // Bob migrates Contract A successfully to new_count = 555
    let (migrate_receipt, executed) = apply_tx(
        &mut producer,
        &mut follower,
        &bob,
        &mut bob_nonce,
        TxMessage::MigrateContract {
            contract: contract_a,
            new_code_id: code_id,
            msg: serde_json::to_vec(&serde_json::json!({ "new_count": 555 })).unwrap(),
        },
    );
    assert!(executed);
    assert!(migrate_receipt.success);

    // Verify Contract A count was updated to 555 via migration entrypoint
    let a_query_migrated = producer
        .query_contract(contract_a, br#"{"count":{}}"#, 200_000)
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&a_query_migrated.data).unwrap()["count"],
        555
    );

    // 10. Final State Root Determinism Check
    assert_eq!(
        producer.state_root().unwrap(),
        follower.state_root().unwrap(),
        "final producer and follower state roots must be perfectly deterministic"
    );
}
