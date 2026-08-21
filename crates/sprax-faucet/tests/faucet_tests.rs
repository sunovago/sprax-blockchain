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
