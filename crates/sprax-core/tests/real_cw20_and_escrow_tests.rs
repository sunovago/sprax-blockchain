use sprax_core::{ChainLedger, GenesisAccount, GenesisConfig};
use sprax_crypto::Ed25519Keypair;
use sprax_storage::RedbStore;
use sprax_types::{Address, ChainId, Hash32, KeyType, Transaction, TxBody, TxFee, TxMessage};

fn load_cw20_wasm() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/examples/cw20-token/target/wasm32-unknown-unknown/release/sprax_cw20_token.wasm");
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to read sprax_cw20_token.wasm at {}: {e}",
            path.display()
        )
    })
}

fn load_escrow_wasm() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../contracts/examples/escrow/target/wasm32-unknown-unknown/release/sprax_escrow.wasm",
    );
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to read sprax_escrow.wasm at {}: {e}",
            path.display()
        )
    })
}

fn create_tx(key: &Ed25519Keypair, nonce: u64, message: TxMessage) -> Transaction {
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

fn send_tx(
    ledger: &mut ChainLedger<RedbStore>,
    key: &Ed25519Keypair,
    nonce: &mut u64,
    msg: TxMessage,
) -> Vec<u8> {
    let tx = create_tx(key, *nonce, msg);
    *nonce += 1;
    let hash = ledger.submit_transaction(tx).unwrap();
    let block = ledger.mine_block(key.address()).unwrap();
    assert_eq!(block.body.transactions.len(), 1);
    let receipt = ledger.get_transaction(&hash).unwrap().1;
    assert!(receipt.success, "tx failed: {:?}", receipt.error_message);
    receipt.return_data.clone()
}

#[test]
fn real_cw20_token_executes_on_ledger() {
    let wasm = load_cw20_wasm();
    let alice = Ed25519Keypair::from_seed(&[11; 32]);
    let bob = Ed25519Keypair::from_seed(&[22; 32]);

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = alice.address();

    let directory = tempfile::tempdir().unwrap();
    let store = RedbStore::open(&directory.path().join("cw20_ledger.redb")).unwrap();
    let mut ledger = ChainLedger::init_from_genesis_with_store(genesis, store.clone()).unwrap();

    let mut nonce = 0;

    // 1. StoreCode
    let code_return = send_tx(
        &mut ledger,
        &alice,
        &mut nonce,
        TxMessage::StoreCode {
            wasm_bytecode: wasm,
        },
    );
    let code_id: Hash32 = serde_json::from_slice(&code_return).unwrap();

    // 2. Instantiate CW20 Token
    let init_msg = serde_json::json!({
        "name": "Sprax Community Token",
        "symbol": "SPRXC",
        "decimals": 18,
        "initial_balances": [
            {
                "address": alice.address().to_string(),
                "amount": 1_000_000
            }
        ],
        "minter": alice.address().to_string()
    });

    let addr_return = send_tx(
        &mut ledger,
        &alice,
        &mut nonce,
        TxMessage::InstantiateContract {
            code_id,
            msg: serde_json::to_vec(&init_msg).unwrap(),
            funds: sprax_types::Amount::ZERO,
            label: "cw20-token".into(),
            admin: None,
        },
    );
    let contract_addr: Address = serde_json::from_slice(&addr_return).unwrap();

    // 3. Query initial balance of Alice
    let query_alice = serde_json::json!({
        "balance": {
            "address": alice.address().to_string()
        }
    });
    let res = ledger
        .query_contract(
            contract_addr,
            &serde_json::to_vec(&query_alice).unwrap(),
            200_000,
        )
        .unwrap();
    let val: serde_json::Value = serde_json::from_slice(&res.data).unwrap();
    assert_eq!(val["balance"], 1_000_000);

    // 4. Transfer 400,000 from Alice to Bob
    let transfer_msg = serde_json::json!({
        "transfer": {
            "recipient": bob.address().to_string(),
            "amount": 400_000
        }
    });
    send_tx(
        &mut ledger,
        &alice,
        &mut nonce,
        TxMessage::ContractCall {
            contract: contract_addr,
            data: serde_json::to_vec(&transfer_msg).unwrap(),
            funds: sprax_types::Amount::ZERO,
        },
    );

    // 5. Query balances
    let res_alice = ledger
        .query_contract(
            contract_addr,
            &serde_json::to_vec(&query_alice).unwrap(),
            200_000,
        )
        .unwrap();
    let val_alice: serde_json::Value = serde_json::from_slice(&res_alice.data).unwrap();
    assert_eq!(val_alice["balance"], 600_000);

    let query_bob = serde_json::json!({
        "balance": {
            "address": bob.address().to_string()
        }
    });
    let res_bob = ledger
        .query_contract(
            contract_addr,
            &serde_json::to_vec(&query_bob).unwrap(),
            200_000,
        )
        .unwrap();
    let val_bob: serde_json::Value = serde_json::from_slice(&res_bob.data).unwrap();
    assert_eq!(val_bob["balance"], 400_000);

    // 6. Burn 50,000 tokens from Alice
    let burn_msg = serde_json::json!({
        "burn": {
            "amount": 50_000
        }
    });
    send_tx(
        &mut ledger,
        &alice,
        &mut nonce,
        TxMessage::ContractCall {
            contract: contract_addr,
            data: serde_json::to_vec(&burn_msg).unwrap(),
            funds: sprax_types::Amount::ZERO,
        },
    );

    // 7. Query TokenInfo
    let query_info = serde_json::json!({
        "token_info": {}
    });
    let res_info = ledger
        .query_contract(
            contract_addr,
            &serde_json::to_vec(&query_info).unwrap(),
            200_000,
        )
        .unwrap();
    let val_info: serde_json::Value = serde_json::from_slice(&res_info.data).unwrap();
    assert_eq!(val_info["total_supply"], 950_000);
    assert_eq!(val_info["name"], "Sprax Community Token");
    assert_eq!(val_info["symbol"], "SPRXC");
}

#[test]
fn real_escrow_contract_executes_on_ledger() {
    let wasm = load_escrow_wasm();
    let buyer = Ed25519Keypair::from_seed(&[33; 32]);
    let seller = Ed25519Keypair::from_seed(&[44; 32]);
    let arbiter = Ed25519Keypair::from_seed(&[55; 32]);

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = buyer.address();
    genesis.accounts.push(GenesisAccount {
        name: "arbiter".to_string(),
        address: arbiter.address(),
        initial_balance: sprax_types::Amount::from_sprx_whole(1_000_000).unwrap(),
    });

    let directory = tempfile::tempdir().unwrap();
    let store = RedbStore::open(&directory.path().join("escrow_ledger.redb")).unwrap();
    let mut ledger = ChainLedger::init_from_genesis_with_store(genesis, store).unwrap();

    let mut buyer_nonce = 0;
    let mut arbiter_nonce = 0;

    // 1. StoreCode
    let code_return = send_tx(
        &mut ledger,
        &buyer,
        &mut buyer_nonce,
        TxMessage::StoreCode {
            wasm_bytecode: wasm,
        },
    );
    let code_id: Hash32 = serde_json::from_slice(&code_return).unwrap();

    // 2. Instantiate Escrow
    let init_msg = serde_json::json!({
        "arbiter": arbiter.address().to_string(),
        "recipient": seller.address().to_string(),
        "timeout_height": 200
    });
    let addr_return = send_tx(
        &mut ledger,
        &buyer,
        &mut buyer_nonce,
        TxMessage::InstantiateContract {
            code_id,
            msg: serde_json::to_vec(&init_msg).unwrap(),
            funds: sprax_types::Amount::ZERO,
            label: "escrow".into(),
            admin: None,
        },
    );
    let contract_addr: Address = serde_json::from_slice(&addr_return).unwrap();

    // 3. Deposit into Escrow
    let deposit_msg = serde_json::json!({
        "deposit": {
            "amount": 75_000
        }
    });
    send_tx(
        &mut ledger,
        &buyer,
        &mut buyer_nonce,
        TxMessage::ContractCall {
            contract: contract_addr,
            data: serde_json::to_vec(&deposit_msg).unwrap(),
            funds: sprax_types::Amount::ZERO,
        },
    );

    // 4. Query Escrow status
    let query_escrow = serde_json::json!({
        "get_escrow": {}
    });
    let res = ledger
        .query_contract(
            contract_addr,
            &serde_json::to_vec(&query_escrow).unwrap(),
            200_000,
        )
        .unwrap();
    let val: serde_json::Value = serde_json::from_slice(&res.data).unwrap();
    assert_eq!(val["amount"], 75_000);
    assert_eq!(val["status"], "Locked");

    // 5. Arbiter releases Escrow
    let release_msg = serde_json::json!({
        "release": {}
    });
    send_tx(
        &mut ledger,
        &arbiter,
        &mut arbiter_nonce,
        TxMessage::ContractCall {
            contract: contract_addr,
            data: serde_json::to_vec(&release_msg).unwrap(),
            funds: sprax_types::Amount::ZERO,
        },
    );

    // 6. Query Escrow final status
    let res_final = ledger
        .query_contract(
            contract_addr,
            &serde_json::to_vec(&query_escrow).unwrap(),
            200_000,
        )
        .unwrap();
    let val_final: serde_json::Value = serde_json::from_slice(&res_final.data).unwrap();
    assert_eq!(val_final["status"], "Released");
}
