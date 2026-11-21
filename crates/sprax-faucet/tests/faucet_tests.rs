use sprax_core::{genesis::GenesisConfig, ledger::ChainLedger};
use sprax_crypto::Ed25519Keypair;
use sprax_faucet::{FaucetError, FaucetService};
use sprax_types::{Address, Amount, ChainId};

#[test]
fn test_faucet_disbursement_rate_limiting_and_auditing() {
    let faucet_kp = Ed25519Keypair::from_seed(&[100u8; 32]);
    let recipient_kp = Ed25519Keypair::from_seed(&[101u8; 32]);

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = faucet_kp.address();
    genesis.accounts[0].initial_balance = Amount::from_sprx_whole(1_000_000).unwrap();

    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();

    let faucet = FaucetService::new(
        faucet_kp,
        ChainId::new("sprax-devnet-1").unwrap(),
        3600, // 1 hour rate limit
        100,  // 100 SPRX max payout
    );

    // 1. Initial Successful Claim
    let claim1 = faucet
        .request_funds(
            &mut ledger,
            &recipient_kp.address().to_string(),
            Some(50),
            "198.51.100.1",
            1_000,
        )
        .unwrap();

    assert!(claim1.success);
    assert_eq!(claim1.recipient, recipient_kp.address());
    assert_eq!(claim1.amount, Amount::from_sprx_whole(50).unwrap());

    // Mine a block to confirm the transfer
    ledger.mine_block(Address::new([0xaa; 20])).unwrap();

    let rec_acc = ledger.get_account(&recipient_kp.address()).unwrap();
    assert_eq!(rec_acc.balance, Amount::from_sprx_whole(50).unwrap());

    // 2. Immediate Second Claim -> Fails with RateLimitExceeded
    let claim_dup = faucet.request_funds(
        &mut ledger,
        &recipient_kp.address().to_string(),
        Some(50),
        "198.51.100.1",
        1_050, // Only 50 seconds later
    );
    assert_eq!(
        claim_dup,
        Err(FaucetError::RateLimitExceeded {
            retry_after_secs: 3550
        })
    );

    // 3. Claim Exceeding Maximum Limit -> Fails with AmountExceedsLimit
    let random_recipient = Address::new([0xfe; 20]);
    let claim_excess = faucet.request_funds(
        &mut ledger,
        &random_recipient.to_string(),
        Some(200), // Exceeds 100 max
        "198.51.100.2",
        1_000,
    );
    assert!(matches!(
        claim_excess,
        Err(FaucetError::AmountExceedsLimit { .. })
    ));

    // 4. Advance Timestamp Beyond Rate Limit Window (1,000 + 3,601 = 4,601) -> Succeeds!
    let claim2 = faucet
        .request_funds(
            &mut ledger,
            &recipient_kp.address().to_string(),
            Some(50),
            "198.51.100.1",
            4_601,
        )
        .unwrap();
    assert!(claim2.success);

    // 5. Verify Stats & Audit Log
    let stats = faucet.get_stats(&ledger);
    assert_eq!(stats.total_claims_count, 2);
    assert_eq!(stats.total_disbursed, Amount::from_sprx_whole(100).unwrap());

    let audit_log = faucet.audit_log();
    assert_eq!(audit_log.len(), 2);
    assert_eq!(audit_log[0].client_ip, "198.51.100.1");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_faucet_http_server_endpoints() {
    let faucet_kp = Ed25519Keypair::from_seed(&[200u8; 32]);
    let recipient_kp = Ed25519Keypair::from_seed(&[201u8; 32]);

    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("faucet_test.redb");
    let store = sprax_storage::RedbStore::open(&db_path).unwrap();

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = faucet_kp.address();
    genesis.accounts[0].initial_balance = Amount::from_sprx_whole(1_000_000).unwrap();

    let ledger = ChainLedger::init_from_genesis_with_store(genesis, store).unwrap();
    let ledger_arc = std::sync::Arc::new(parking_lot::RwLock::new(ledger));

    let faucet = std::sync::Arc::new(FaucetService::new(
        faucet_kp,
        ChainId::new("sprax-devnet-1").unwrap(),
        3600,
        100,
    ));

    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };

    sprax_faucet::FaucetServer::start(faucet.clone(), ledger_arc.clone(), port)
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    // 1. Check /health
    let health_resp: serde_json::Value = ureq::get(&format!("http://127.0.0.1:{port}/health"))
        .call()
        .unwrap()
        .into_json()
        .unwrap();
    assert_eq!(health_resp["status"], "healthy");

    // 2. Check /stats
    let stats_resp: serde_json::Value = ureq::get(&format!("http://127.0.0.1:{port}/stats"))
        .call()
        .unwrap()
        .into_json()
        .unwrap();
    assert_eq!(stats_resp["network"], "sprax-devnet-1");

    // 3. Post /claim
    let claim_body = serde_json::json!({
        "recipient": recipient_kp.address().to_string(),
        "amount": 50
    });
    let claim_resp: serde_json::Value = ureq::post(&format!("http://127.0.0.1:{port}/claim"))
        .send_json(claim_body.clone())
        .unwrap()
        .into_json()
        .unwrap();
    assert_eq!(claim_resp["success"], true);

    // 4. Immediate second claim should get 429 Too Many Requests
    let err = ureq::post(&format!("http://127.0.0.1:{port}/claim"))
        .send_json(claim_body)
        .unwrap_err();
    match err {
        ureq::Error::Status(code, _) => assert_eq!(code, 429),
        other => panic!("Expected status 429, got: {other:?}"),
    }
}
