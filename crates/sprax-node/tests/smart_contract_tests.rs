use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use sprax_core::{GenesisAccount, GenesisConfig};
use sprax_crypto::Ed25519Keypair;
use sprax_node::{NodeService, RpcServer};
use sprax_types::{Address, Amount};
use sprax_wasm::{
    ContractError, Env, EscrowContract, EscrowExecuteMsg, EscrowQueryMsg, EscrowState,
    EscrowStatus, GasMeter, InitialBalance, MessageInfo, TokenContract, TokenExecuteMsg, TokenInfo,
    TokenQueryMsg, WasmContractEngine,
};
use tower::ServiceExt;

fn setup_test_node() -> (tempfile::TempDir, NodeService, axum::Router, Ed25519Keypair) {
    let temp_dir = tempfile::tempdir().unwrap();
    let alice_kp = Ed25519Keypair::from_seed(&[1u8; 32]);
    let mut genesis = GenesisConfig::default_development();
    genesis.accounts.push(GenesisAccount {
        name: "alice".into(),
        address: alice_kp.address(),
        initial_balance: Amount::from_sprx_whole(10_000).unwrap(),
    });
    genesis
        .save_to_file(&temp_dir.path().join("genesis.json"))
        .unwrap();

    let node = NodeService::new_or_load(temp_dir.path().to_path_buf()).unwrap();
    let router = RpcServer::router(node.clone());
    (temp_dir, node, router, alice_kp)
}

#[test]
fn test_wasm_bytecode_validation_and_security() {
    let vm = WasmContractEngine::new();
    let creator = Address::new([1u8; 20]);

    // 1. Empty bytecode must be rejected
    let empty_err = vm.store_code(creator, vec![], 1).unwrap_err();
    assert!(matches!(empty_err, ContractError::InvalidInput(_)));

    // 2. Valid bytecode is stored and receives deterministic CodeId
    let code_id = vm.store_code(creator, vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00], 1).unwrap();
    assert_eq!(code_id, 1);

    // 3. CodeInfo is accurately retrieved
    let code_info = vm.get_code_info(code_id).expect("code info should exist");
    assert_eq!(code_info.code_id, 1);
    assert_eq!(code_info.creator, creator);
    assert_eq!(code_info.code_size, 8);
}

#[test]
fn test_deterministic_contract_address_derivation() {
    let vm1 = WasmContractEngine::new();
    let vm2 = WasmContractEngine::new();

    let creator = Address::new([42u8; 20]);
    let code_id = 1;
    let label = "sprax-erc20";

    let mut env1 = Env {
        block_height: 10,
        block_timestamp_unix: 1_700_000_000,
        chain_id: "sprax-mainnet-1".into(),
        contract_address: Address::ZERO,
    };
    let mut gas1 = GasMeter::new(100_000);

    let _ = vm1.store_code(creator, vec![0xbb; 32], 10).unwrap();
    let addr1 = vm1
        .instantiate(creator, code_id, label.into(), None, Amount::ZERO, &mut env1, &mut gas1)
        .unwrap();

    let mut env2 = Env {
        block_height: 10,
        block_timestamp_unix: 1_700_000_000,
        chain_id: "sprax-mainnet-1".into(),
        contract_address: Address::ZERO,
    };
    let mut gas2 = GasMeter::new(100_000);

    let _ = vm2.store_code(creator, vec![0xbb; 32], 10).unwrap();
    let addr2 = vm2
        .instantiate(creator, code_id, label.into(), None, Amount::ZERO, &mut env2, &mut gas2)
        .unwrap();

    // Two independent node VMs must derive the exact identical contract address
    assert_eq!(addr1, addr2);
    assert_ne!(addr1, Address::ZERO);
}

#[test]
fn test_sprax20_token_full_lifecycle_and_gas_metering() {
    let vm = WasmContractEngine::new();
    let admin = Address::new([10u8; 20]);
    let alice = Address::new([11u8; 20]);
    let bob = Address::new([12u8; 20]);

    let mut env = Env {
        block_height: 1,
        block_timestamp_unix: 1_700_000_000,
        chain_id: "test-chain".into(),
        contract_address: Address::ZERO,
    };
    let mut gas = GasMeter::new(1_000_000);

    let code_id = vm.store_code(admin, vec![0xaa; 64], 1).unwrap();
    let _contract_addr = vm
        .instantiate(admin, code_id, "SPRX-20 Gold".into(), Some(admin), Amount::ZERO, &mut env, &mut gas)
        .unwrap();

    let info = MessageInfo {
        sender: admin,
        funds: Amount::ZERO,
    };
    let mut storage = vm.storage().write();

    // 1. Instantiate Token Contract with 1,000,000 initial supply to Alice
    let initial_amount = Amount::from_atto(1_000_000_000_000_000_000); // 1 Token
    let init_resp = TokenContract::instantiate(
        &mut storage,
        &env,
        &info,
        &mut gas,
        "SPRX Gold".into(),
        "GLD".into(),
        18,
        vec![InitialBalance {
            address: alice,
            amount: initial_amount,
        }],
        Some(admin),
    )
    .unwrap();

    assert_eq!(init_resp.events.len(), 1);
    assert_eq!(init_resp.events[0].event_type, "cw20-instantiate");

    // 2. Query Alice's Balance
    let alice_bal_bytes = TokenContract::query(&storage, &env, TokenQueryMsg::Balance { address: alice }).unwrap();
    let alice_bal: Amount = serde_json::from_slice(&alice_bal_bytes).unwrap();
    assert_eq!(alice_bal, initial_amount);

    // 3. Alice Transfers 400,000 to Bob
    let transfer_amount = Amount::from_atto(400_000_000_000_000_000);
    let alice_info = MessageInfo {
        sender: alice,
        funds: Amount::ZERO,
    };
    let transfer_resp = TokenContract::execute(
        &mut storage,
        &env,
        &alice_info,
        &mut gas,
        TokenExecuteMsg::Transfer {
            recipient: bob,
            amount: transfer_amount,
        },
    )
    .unwrap();

    assert_eq!(transfer_resp.events.len(), 1);
    assert_eq!(transfer_resp.events[0].event_type, "cw20-transfer");

    // 4. Verify balances after transfer
    let bob_bal_bytes = TokenContract::query(&storage, &env, TokenQueryMsg::Balance { address: bob }).unwrap();
    let bob_bal: Amount = serde_json::from_slice(&bob_bal_bytes).unwrap();
    assert_eq!(bob_bal, transfer_amount);

    let alice_bal_after: Amount = serde_json::from_slice(
        &TokenContract::query(&storage, &env, TokenQueryMsg::Balance { address: alice }).unwrap(),
    )
    .unwrap();
    assert_eq!(alice_bal_after, initial_amount.checked_sub(transfer_amount).unwrap());

    // 5. Query TokenInfo
    let token_info_bytes = TokenContract::query(&storage, &env, TokenQueryMsg::TokenInfo {}).unwrap();
    let token_info: TokenInfo = serde_json::from_slice(&token_info_bytes).unwrap();
    assert_eq!(token_info.symbol, "GLD");
    assert_eq!(token_info.decimals, 18);
    assert_eq!(token_info.total_supply, initial_amount);

    // 6. Assert Gas Metering was deducted
    assert!(gas.consumed() > 0);
    assert!(gas.remaining() < 1_000_000);
}

#[test]
fn test_escrow_contract_multi_party_workflow() {
    let vm = WasmContractEngine::new();
    let depositor = Address::new([20u8; 20]);
    let recipient = Address::new([21u8; 20]);
    let arbiter = Address::new([22u8; 20]);

    let mut env = Env {
        block_height: 5,
        block_timestamp_unix: 1_700_000_000,
        chain_id: "test-chain".into(),
        contract_address: Address::ZERO,
    };
    let mut gas = GasMeter::new(500_000);

    let code_id = vm.store_code(depositor, vec![0xee; 64], 5).unwrap();
    let _contract_addr = vm
        .instantiate(depositor, code_id, "SPRX Escrow".into(), None, Amount::ZERO, &mut env, &mut gas)
        .unwrap();

    let deposit_amount = Amount::from_atto(50_000_000_000_000_000);
    let info = MessageInfo {
        sender: depositor,
        funds: deposit_amount,
    };
    let mut storage = vm.storage().write();

    // 1. Initialize Escrow
    EscrowContract::instantiate(
        &mut storage,
        &env,
        &info,
        &mut gas,
        recipient,
        arbiter,
        100, // timeout height
    )
    .unwrap();

    // 2. Query Escrow status (must be Locked)
    let state_bytes = EscrowContract::query(&storage, &env, EscrowQueryMsg::GetEscrow {}).unwrap();
    let state: EscrowState = serde_json::from_slice(&state_bytes).unwrap();
    assert_eq!(state.status, EscrowStatus::Locked);
    assert_eq!(state.arbiter, arbiter);
    assert_eq!(state.recipient, recipient);
    assert_eq!(state.amount, deposit_amount);

    // 3. Unauthorized sender cannot release funds
    let third_party = Address::new([99u8; 20]);
    let bad_info = MessageInfo {
        sender: third_party,
        funds: Amount::ZERO,
    };
    let unauth_err = EscrowContract::execute(&mut storage, &env, &bad_info, &mut gas, EscrowExecuteMsg::Release {}).unwrap_err();
    assert!(matches!(unauth_err, ContractError::Unauthorized(_)));

    // 4. Arbiter approves and releases
    let arbiter_info = MessageInfo {
        sender: arbiter,
        funds: Amount::ZERO,
    };
    let release_resp = EscrowContract::execute(&mut storage, &env, &arbiter_info, &mut gas, EscrowExecuteMsg::Release {}).unwrap();
    assert_eq!(release_resp.events[0].event_type, "escrow-release");

    // 5. Escrow status is now Released
    let state_after: EscrowState = serde_json::from_slice(
        &EscrowContract::query(&storage, &env, EscrowQueryMsg::GetEscrow {}).unwrap(),
    )
    .unwrap();
    assert_eq!(state_after.status, EscrowStatus::Released);
}

#[test]
fn test_contract_security_reentrancy_and_gas_exhaustion() {
    let vm = WasmContractEngine::new();
    let target = Address::new([99u8; 20]);

    // 1. Reentrancy Lock Test
    vm.enter_call(&target).unwrap();
    let reentrancy_err = vm.enter_call(&target).unwrap_err();
    assert!(matches!(reentrancy_err, ContractError::ReentrancyDetected(_)));
    vm.exit_call(&target);
    // After exiting call, entering again succeeds
    assert!(vm.enter_call(&target).is_ok());
    vm.exit_call(&target);

    // 2. Gas Exhaustion Test
    let mut tiny_gas = GasMeter::new(100);
    let mut env = Env {
        block_height: 1,
        block_timestamp_unix: 1_700_000_000,
        chain_id: "test-chain".into(),
        contract_address: target,
    };
    let creator = Address::new([1u8; 20]);
    let code_id = vm.store_code(creator, vec![0x11; 32], 1).unwrap();

    let out_of_gas_err = vm.instantiate(
        creator,
        code_id,
        "Tiny Gas Contract".into(),
        None,
        Amount::ZERO,
        &mut env,
        &mut tiny_gas,
    ).unwrap_err();
    assert!(matches!(out_of_gas_err, ContractError::OutOfGas { .. }));
}

#[tokio::test]
async fn test_smart_contract_rpc_and_rest_endpoints() {
    let (_dir, _node, router, alice) = setup_test_node();
    let alice_addr_str = alice.address().to_string();

    // 1. JSON-RPC sprx_estimateContractGas
    let gas_req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "method": "sprx_estimateContractGas",
            "params": [alice_addr_str, {"balance": {}}],
            "id": 1
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(gas_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body["result"]["gas_estimated"].as_u64().unwrap() > 0);

    // 2. JSON-RPC sprx_getContract (non-existent returns null)
    let get_req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "method": "sprx_getContract",
            "params": [alice_addr_str],
            "id": 2
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(get_req).await.unwrap();
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body["result"].is_null());

    // 3. REST GET /api/v1/contracts
    let list_req = Request::builder()
        .method("GET")
        .uri("/api/v1/contracts")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(list_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let list_body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(list_body["contracts"].is_array());

    // 4. REST GET /api/v1/contracts/{address} (non-existent returns 404)
    let get_addr_req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/contracts/{alice_addr_str}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(get_addr_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_cross_node_execution_determinism() {
    let vm_node1 = WasmContractEngine::new();
    let vm_node2 = WasmContractEngine::new();

    let creator = Address::new([50u8; 20]);
    let recipient = Address::new([51u8; 20]);

    // Store identical code on both nodes
    let bytecode = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    let code_id1 = vm_node1.store_code(creator, bytecode.clone(), 1).unwrap();
    let code_id2 = vm_node2.store_code(creator, bytecode, 1).unwrap();
    assert_eq!(code_id1, code_id2);

    // Instantiate token on Node 1
    let mut env1 = Env {
        block_height: 1,
        block_timestamp_unix: 1_700_000_000,
        chain_id: "sprx-determinism-1".into(),
        contract_address: Address::ZERO,
    };
    let mut gas1 = GasMeter::new(500_000);
    let addr1 = vm_node1.instantiate(creator, code_id1, "Token".into(), Some(creator), Amount::ZERO, &mut env1, &mut gas1).unwrap();

    // Instantiate token on Node 2
    let mut env2 = Env {
        block_height: 1,
        block_timestamp_unix: 1_700_000_000,
        chain_id: "sprx-determinism-1".into(),
        contract_address: Address::ZERO,
    };
    let mut gas2 = GasMeter::new(500_000);
    let addr2 = vm_node2.instantiate(creator, code_id2, "Token".into(), Some(creator), Amount::ZERO, &mut env2, &mut gas2).unwrap();

    assert_eq!(addr1, addr2);
    assert_eq!(gas1.consumed(), gas2.consumed());

    // Execute transfer on Node 1
    let mut storage1 = vm_node1.storage().write();
    let info = MessageInfo { sender: creator, funds: Amount::ZERO };
    TokenContract::instantiate(&mut storage1, &env1, &info, &mut gas1, "T".into(), "T".into(), 6, vec![InitialBalance { address: creator, amount: Amount::from_atto(1_000) }], Some(creator)).unwrap();
    TokenContract::execute(&mut storage1, &env1, &info, &mut gas1, TokenExecuteMsg::Transfer { recipient, amount: Amount::from_atto(300) }).unwrap();

    // Execute transfer on Node 2
    let mut storage2 = vm_node2.storage().write();
    TokenContract::instantiate(&mut storage2, &env2, &info, &mut gas2, "T".into(), "T".into(), 6, vec![InitialBalance { address: creator, amount: Amount::from_atto(1_000) }], Some(creator)).unwrap();
    TokenContract::execute(&mut storage2, &env2, &info, &mut gas2, TokenExecuteMsg::Transfer { recipient, amount: Amount::from_atto(300) }).unwrap();

    // Assert gas consumed is bit-for-bit identical across independent nodes
    assert_eq!(gas1.consumed(), gas2.consumed());

    // Assert resulting balances are bit-for-bit identical
    let b1: Amount = serde_json::from_slice(&TokenContract::query(&storage1, &env1, TokenQueryMsg::Balance { address: recipient }).unwrap()).unwrap();
    let b2: Amount = serde_json::from_slice(&TokenContract::query(&storage2, &env2, TokenQueryMsg::Balance { address: recipient }).unwrap()).unwrap();
    assert_eq!(b1, b2);
    assert_eq!(b1, Amount::from_atto(300));
}
