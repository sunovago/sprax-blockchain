use axum::{body::Body, http::{Request, StatusCode}};
use serde_json::{json, Value};
use sprax_crypto::Ed25519Keypair;
use sprax_node::{NodeService, RpcServer};
use sprax_types::{Amount, ChainId, KeyType, Transaction, TxBody, TxFee, TxMessage};
use tower::ServiceExt;

fn setup_test_node() -> (tempfile::TempDir, NodeService, axum::Router, Ed25519Keypair) {
    let temp_dir = tempfile::tempdir().unwrap();
    let alice_kp = Ed25519Keypair::from_seed(&[1u8; 32]);
    let mut genesis = sprax_core::GenesisConfig::default_development();
    genesis.accounts.push(sprax_core::GenesisAccount {
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

#[tokio::test]
async fn test_json_rpc_blockchain_methods() {
    let (_dir, node, router, _alice) = setup_test_node();

    // 1. sprx_chainId
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sprx_chainId"
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["result"], "sprax-devnet-1");

    // 2. sprx_blockNumber
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "sprx_blockNumber"
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["result"], 0);

    // 3. sprx_getChainInfo
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "sprx_getChainInfo"
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["result"]["chainId"], "sprax-devnet-1");
    assert_eq!(body["result"]["height"], 0);

    // 4. sprx_getSyncStatus
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "sprx_getSyncStatus"
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["result"]["isSyncing"], false);

    // Mine a block and query sprx_getBlockByNumber & sprx_getBlockHeader
    node.mine_block(sprax_types::Address::ZERO).unwrap();

    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "sprx_getBlockByNumber",
            "params": [1]
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 8192).await.unwrap()).unwrap();
    assert_eq!(body["result"]["header"]["height"], 1);

    // sprx_getBlockTransactionCount
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "sprx_getBlockTransactionCount",
            "params": [1]
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["result"], 0);
}

#[tokio::test]
async fn test_json_rpc_batch_requests() {
    let (_dir, _node, router, _alice) = setup_test_node();

    // Valid batch of 3 requests
    let batch_req = json!([
        { "jsonrpc": "2.0", "id": 10, "method": "sprx_chainId" },
        { "jsonrpc": "2.0", "id": 11, "method": "sprx_blockNumber" },
        { "jsonrpc": "2.0", "id": 12, "method": "sprx_netListening" }
    ]);

    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(batch_req.to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    let arr = body.as_array().expect("must return JSON array for batch");
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0]["id"], 10);
    assert_eq!(arr[0]["result"], "sprax-devnet-1");
    assert_eq!(arr[1]["id"], 11);
    assert_eq!(arr[1]["result"], 0);
    assert_eq!(arr[2]["id"], 12);
    assert_eq!(arr[2]["result"], true);

    // Empty batch returns error -32600
    let empty_batch = json!([]);
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(empty_batch.to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["error"]["code"], -32600);
}

#[tokio::test]
async fn test_json_rpc_error_handling() {
    let (_dir, _node, router, _alice) = setup_test_node();

    // Unknown method -> -32601
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 99,
            "method": "sprx_nonExistentMethod"
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["error"]["code"], -32601);

    // Invalid params -> -32602
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 100,
            "method": "sprx_getBlockByHash",
            "params": ["invalid_hash_string"]
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["error"]["code"], -32602);
}

#[tokio::test]
async fn test_transaction_submission_and_status_lifecycle() {
    let (_dir, node, router, alice) = setup_test_node();
    let bob = Ed25519Keypair::from_seed(&[2u8; 32]);

    // Build valid transaction
    let tx_body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: alice.address(),
        nonce: 0,
        messages: vec![TxMessage::Transfer {
            to: bob.address(),
            amount: Amount::from_sprx_whole(100).unwrap(),
        }],
        fee: TxFee::default(),
        memo: "rpc test tx".into(),
        timeout_height: 50,
    };
    let sign_bytes = tx_body.sign_bytes().unwrap();
    let sig = alice.sign(&sign_bytes);
    let tx = Transaction::new(
        tx_body,
        KeyType::Ed25519,
        alice.public_key_bytes().to_vec(),
        sig,
    )
    .unwrap();

    let tx_val = serde_json::to_value(&tx).unwrap();

    // 1. Submit via JSON-RPC sprx_sendRawTransaction
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 201,
            "method": "sprx_sendRawTransaction",
            "params": [tx_val]
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    let tx_hash = body["result"]["txHash"].as_str().expect("must return txHash");
    assert!(!tx_hash.is_empty());

    // 2. Query status -> pending in mempool
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 202,
            "method": "sprx_getTransactionStatus",
            "params": [tx_hash]
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["result"]["status"], "pending");
    assert_eq!(body["result"]["isPending"], true);

    // 3. Query sprx_getPendingTransactions
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 203,
            "method": "sprx_getPendingTransactions"
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["result"].as_array().unwrap().len(), 1);

    // 4. Mine block to finalize
    let block = node.mine_block(sprax_types::Address::ZERO).unwrap();
    assert_eq!(block.body.transactions.len(), 1);

    // 5. Query status -> finalized
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 204,
            "method": "sprx_getTransactionStatus",
            "params": [tx_hash]
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["result"]["status"], "finalized");
    assert_eq!(body["result"]["isFinalized"], true);
    assert_eq!(body["result"]["height"], 1);

    // 6. Query sprx_getTransactionReceipt
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 205,
            "method": "sprx_getTransactionReceipt",
            "params": [tx_hash]
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["result"]["success"], true);

    // 7. Verify recipient balance via sprx_getBalance
    let req = Request::builder()
        .method("POST")
        .uri("/rpc")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "id": 206,
            "method": "sprx_getBalance",
            "params": [bob.address().to_hex()]
        }).to_string()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert!(body["result"]["balance_sprx"].as_str().unwrap().contains("100"));
}

#[tokio::test]
async fn test_rest_api_v1_endpoints() {
    let (_dir, node, router, alice) = setup_test_node();

    // GET /api/v1/chain/info
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/chain/info")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["chain_id"], "sprax-devnet-1");

    // GET /api/v1/chain/status
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/chain/status")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // GET /api/v1/accounts/:address/balance
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/accounts/{}/balance", alice.address().to_hex()))
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["balance_atto"], "10000000000000000000000");

    // GET /api/v1/validators
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/validators")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // GET /api/v1/staking/overview
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/staking/overview")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // GET /api/v1/network/status
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/network/status")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // GET /api/v1/network/sync
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/network/sync")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Mine a block and query GET /api/v1/blocks
    node.mine_block(sprax_types::Address::ZERO).unwrap();
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/blocks?limit=10&offset=0")
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["current_height"], 1);
}
