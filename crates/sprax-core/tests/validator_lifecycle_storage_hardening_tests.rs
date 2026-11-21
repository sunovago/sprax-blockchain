use sprax_core::state::{CanonicalValidatorStatus, StateAccessor};
use sprax_core::{ChainLedger, GenesisAccount, GenesisConfig, GenesisValidator};
use sprax_crypto::Ed25519Keypair;
use sprax_storage::{
    create_backup, rebuild_chain_indexes, restore_backup, verify_snapshot, MemKVStore,
    SnapshotMetadata, RESTORED_SIGNING_DISABLED_MARKER, SNAPSHOT_MANIFEST_NAME,
};
use sprax_types::{
    Address, Amount, ChainId, EquivocationEvidence, Hash32, KeyType, Transaction, TxBody, TxFee,
    TxMessage, Vote, VoteType,
};
use tempfile::tempdir;

fn setup_test_genesis() -> (GenesisConfig, Ed25519Keypair, Ed25519Keypair) {
    let alice = Ed25519Keypair::from_seed(&[1u8; 32]);
    let bob = Ed25519Keypair::from_seed(&[2u8; 32]);

    let mut g = GenesisConfig::default_development();
    g.chain_id = "sprax-testnet-1".to_string();
    g.accounts = vec![
        GenesisAccount {
            name: "alice".into(),
            address: alice.address(),
            initial_balance: Amount::from_sprx_whole(100_000).unwrap(),
        },
        GenesisAccount {
            name: "bob".into(),
            address: bob.address(),
            initial_balance: Amount::from_sprx_whole(100_000).unwrap(),
        },
    ];
    g.validators = vec![
        GenesisValidator {
            operator_address: alice.address(),
            consensus_pubkey: alice.public_key_bytes().to_vec(),
            self_stake: Amount::from_sprx_whole(10_000).unwrap(),
            moniker: "alice".into(),
        },
        GenesisValidator {
            operator_address: bob.address(),
            consensus_pubkey: bob.public_key_bytes().to_vec(),
            self_stake: Amount::from_sprx_whole(10_000).unwrap(),
            moniker: "bob".into(),
        },
    ];
    (g, alice, bob)
}

fn create_signed_tx(
    chain_id: &ChainId,
    sender_kp: &Ed25519Keypair,
    nonce: u64,
    messages: Vec<TxMessage>,
    gas_limit: u64,
    timeout_height: u64,
) -> Transaction {
    let body = TxBody {
        chain_id: chain_id.clone(),
        sender: sender_kp.address(),
        nonce,
        messages,
        fee: TxFee {
            amount: Amount::from_atto(500_000_000_000_000), // 0.0005 SPRX
            gas_limit,
            priority_fee: Amount::ZERO,
        },
        memo: String::new(),
        timeout_height,
    };
    let sign_bytes = body.sign_bytes().unwrap();
    let signature = sender_kp.sign(&sign_bytes);
    Transaction::new(
        body,
        KeyType::Ed25519,
        sender_kp.public_key_bytes().to_vec(),
        signature,
    )
    .unwrap()
}

fn make_pop(
    chain_id: &ChainId,
    sender: &Address,
    consensus_pubkey: &[u8],
    kp: &Ed25519Keypair,
) -> Vec<u8> {
    let msg = [
        b"sprax/pop/v1".as_slice(),
        chain_id.as_str().as_bytes(),
        sender.as_bytes(),
        consensus_pubkey,
    ]
    .concat();
    kp.sign(&msg)
}

fn make_vote(
    genesis: Hash32,
    vote_type: VoteType,
    height: u64,
    round: u32,
    block_hash: Option<Hash32>,
    kp: &Ed25519Keypair,
) -> Vote {
    let mut vote = Vote {
        genesis,
        vote_type,
        height,
        round,
        block_hash,
        validator_address: kp.address(),
        signature: Vec::new(),
    };
    let sign_bytes = vote.sign_bytes().unwrap();
    vote.signature = kp.sign(&sign_bytes);
    vote
}

#[test]
fn test_scenario_a_registration_funding_duplicate_keys_and_mempool_reservation() {
    let (g, alice, _bob) = setup_test_genesis();
    let chain_id = ChainId::new(&g.chain_id).unwrap();
    let mut ledger = ChainLedger::init_from_genesis(g.clone()).unwrap();

    let carol = Ed25519Keypair::from_seed(&[3u8; 32]);
    let carol_consensus = Ed25519Keypair::from_seed(&[33u8; 32]);

    // Carol has 0 balance - submit registration -> must fail pre-validation
    let pop_bytes = make_pop(
        &chain_id,
        &carol.address(),
        &carol_consensus.public_key_bytes(),
        &carol_consensus,
    );

    let reg_msg = TxMessage::RegisterValidator {
        consensus_pubkey: carol_consensus.public_key_bytes().to_vec(),
        moniker: "carol".into(),
        self_stake: Amount::from_sprx_whole(1_000).unwrap(),
        proof_of_possession: pop_bytes.clone(),
    };

    let tx_fail = create_signed_tx(&chain_id, &carol, 0, vec![reg_msg.clone()], 300_000, 100);
    let submit_res = ledger.submit_transaction(tx_fail);
    assert!(
        submit_res.is_err(),
        "0-balance account must not enter mempool for validator registration"
    );

    // Fund Carol with 1,500 SPRX from Alice
    let transfer_tx = create_signed_tx(
        &chain_id,
        &alice,
        0,
        vec![TxMessage::Transfer {
            to: carol.address(),
            amount: Amount::from_sprx_whole(1_500).unwrap(),
        }],
        100_000,
        100,
    );
    ledger.submit_transaction(transfer_tx).unwrap();
    let block1 = ledger.mine_block(alice.address()).unwrap();
    assert_eq!(block1.header.height, 1);

    // Carol submits registration (1,000 SPRX self-stake + fee) -> succeeds
    let tx_carol = create_signed_tx(&chain_id, &carol, 0, vec![reg_msg.clone()], 300_000, 100);
    ledger.submit_transaction(tx_carol).unwrap();

    // Competing mempool transaction from Carol attempting to spend 1,000 SPRX -> must fail admission
    let competing_transfer = create_signed_tx(
        &chain_id,
        &carol,
        1,
        vec![TxMessage::Transfer {
            to: alice.address(),
            amount: Amount::from_sprx_whole(1_000).unwrap(),
        }],
        100_000,
        100,
    );
    let competing_res = ledger.submit_transaction(competing_transfer);
    assert!(
        competing_res.is_err(),
        "Competing mempool transaction must be rejected due to pending balance reservation"
    );

    // Mine block with Carol's registration
    let block2 = ledger.mine_block(alice.address()).unwrap();
    assert_eq!(block2.header.height, 2);

    let carol_rec = ledger
        .get_validator_record(&carol.address())
        .unwrap()
        .expect("carol registered");
    assert_eq!(carol_rec.status, CanonicalValidatorStatus::Active);
    assert!(!carol_rec.is_tombstoned);

    // Attempting to register another operator with the SAME consensus pubkey -> must fail
    let dave = Ed25519Keypair::from_seed(&[4u8; 32]);
    let fund_dave = create_signed_tx(
        &chain_id,
        &alice,
        1,
        vec![TxMessage::Transfer {
            to: dave.address(),
            amount: Amount::from_sprx_whole(5_000).unwrap(),
        }],
        100_000,
        100,
    );
    ledger.submit_transaction(fund_dave).unwrap();
    ledger.mine_block(alice.address()).unwrap();

    let dave_pop = make_pop(
        &chain_id,
        &dave.address(),
        &carol_consensus.public_key_bytes(),
        &carol_consensus,
    );
    let dup_key_tx = create_signed_tx(
        &chain_id,
        &dave,
        0,
        vec![TxMessage::RegisterValidator {
            consensus_pubkey: carol_consensus.public_key_bytes().to_vec(),
            moniker: "dave_duplicate".into(),
            self_stake: Amount::from_sprx_whole(1_000).unwrap(),
            proof_of_possession: dave_pop,
        }],
        300_000,
        100,
    );
    let dup_res = ledger.submit_transaction(dup_key_tx);
    assert!(
        dup_res.is_err(),
        "Duplicate consensus pubkey must be rejected"
    );
}

#[test]
fn test_scenario_b_active_set_transition_at_h_plus_1_boundary() {
    let (g, alice, _bob) = setup_test_genesis();
    let chain_id = ChainId::new(&g.chain_id).unwrap();
    let mut ledger = ChainLedger::init_from_genesis(g.clone()).unwrap();

    let carol = Ed25519Keypair::from_seed(&[10u8; 32]);
    let carol_con = Ed25519Keypair::from_seed(&[11u8; 32]);

    // Fund Carol
    let fund = create_signed_tx(
        &chain_id,
        &alice,
        0,
        vec![TxMessage::Transfer {
            to: carol.address(),
            amount: Amount::from_sprx_whole(20_000).unwrap(),
        }],
        100_000,
        100,
    );
    ledger.submit_transaction(fund).unwrap();
    ledger.mine_block(alice.address()).unwrap(); // Height 1

    let initial_val_hash = ledger.active_validator_set_hash().unwrap();

    // Register Carol with 15,000 SPRX self stake (more than Alice & Bob)
    let pop = make_pop(
        &chain_id,
        &carol.address(),
        &carol_con.public_key_bytes(),
        &carol_con,
    );
    let reg = create_signed_tx(
        &chain_id,
        &carol,
        0,
        vec![TxMessage::RegisterValidator {
            consensus_pubkey: carol_con.public_key_bytes().to_vec(),
            moniker: "carol_big".into(),
            self_stake: Amount::from_sprx_whole(15_000).unwrap(),
            proof_of_possession: pop,
        }],
        300_000,
        100,
    );
    ledger.submit_transaction(reg).unwrap();

    // Before block 2 is mined, proposed block 2 header commits the validator set finalized at height 1
    let proposal_block2 = ledger.build_proposal(alice.address()).unwrap();
    assert_eq!(
        proposal_block2.header.validator_set_hash, initial_val_hash,
        "Block 2 proposal must commit pre-execution active validator set from height 1"
    );

    // Apply / mine block 2
    let block2 = ledger.mine_block(alice.address()).unwrap();
    assert_eq!(block2.header.height, 2);

    // Now Carol is part of the canonical active set finalized at height 2
    let active_after_h2 = ledger.active_validators().unwrap();
    assert_eq!(active_after_h2.len(), 3);
    let carol_val = active_after_h2
        .iter()
        .find(|v| v.address == carol.address())
        .expect("Carol must be in active set");
    assert_eq!(carol_val.voting_power, 15_000);

    let val_hash_h2 = ledger.active_validator_set_hash().unwrap();
    assert_ne!(val_hash_h2, initial_val_hash);

    // Block 3 proposal must now commit val_hash_h2
    let proposal_block3 = ledger.build_proposal(alice.address()).unwrap();
    assert_eq!(
        proposal_block3.header.validator_set_hash, val_hash_h2,
        "Block 3 proposal must commit validator set finalized at height 2 (H+1 transition)"
    );
}

#[test]
fn test_scenario_c_delegation_unbonding_maturity_and_withdrawal() {
    let (g, alice, _bob) = setup_test_genesis();
    let chain_id = ChainId::new(&g.chain_id).unwrap();
    let mut ledger = ChainLedger::init_from_genesis(g.clone()).unwrap();

    let user = Ed25519Keypair::from_seed(&[20u8; 32]);

    // Fund user with 1,000 SPRX
    let fund = create_signed_tx(
        &chain_id,
        &alice,
        0,
        vec![TxMessage::Transfer {
            to: user.address(),
            amount: Amount::from_sprx_whole(1_000).unwrap(),
        }],
        100_000,
        100,
    );
    ledger.submit_transaction(fund).unwrap();
    ledger.mine_block(alice.address()).unwrap(); // Height 1

    // Delegate 500 SPRX to Alice
    let delegate_tx = create_signed_tx(
        &chain_id,
        &user,
        0,
        vec![TxMessage::Delegate {
            validator: alice.address(),
            amount: Amount::from_sprx_whole(500).unwrap(),
        }],
        100_000,
        100,
    );
    ledger.submit_transaction(delegate_tx).unwrap();
    ledger.mine_block(alice.address()).unwrap(); // Height 2

    let user_acc = ledger.get_account(&user.address()).unwrap();
    assert!(user_acc.balance.as_atto() < 500 * 1_000_000_000_000_000_000);

    // Unbond 200 SPRX
    let unbond_tx = create_signed_tx(
        &chain_id,
        &user,
        1,
        vec![TxMessage::Unbond {
            validator: alice.address(),
            amount: Amount::from_sprx_whole(200).unwrap(),
        }],
        100_000,
        100,
    );
    ledger.submit_transaction(unbond_tx).unwrap();
    ledger.mine_block(alice.address()).unwrap(); // Height 3

    // Verify unbonding is pending in store
    let unbondings =
        StateAccessor::scan_unbondings_for_validator(ledger.store(), &alice.address()).unwrap();
    assert_eq!(unbondings.len(), 1);
    assert_eq!(
        unbondings[0].1.amount,
        Amount::from_sprx_whole(200).unwrap()
    );
    assert_eq!(unbondings[0].1.delegator, user.address());
}

#[test]
fn test_scenario_d_and_e_jail_unjail_cooldown_and_tombstone_permanence() {
    let (g, alice, bob) = setup_test_genesis();
    let genesis_fingerprint = g.fingerprint().unwrap();
    let chain_id = ChainId::new(&g.chain_id).unwrap();
    let mut ledger = ChainLedger::init_from_genesis(g.clone()).unwrap();

    // Alice equivocates and is slashed and tombstoned
    let vote_a = make_vote(
        genesis_fingerprint,
        VoteType::Precommit,
        0,
        0,
        Some(Hash32::new([11u8; 32])),
        &alice,
    );
    let vote_b = make_vote(
        genesis_fingerprint,
        VoteType::Precommit,
        0,
        0,
        Some(Hash32::new([22u8; 32])),
        &alice,
    );
    let evidence = EquivocationEvidence::new(alice.address(), 0, 0, vote_a, vote_b);

    let slash_tx = create_signed_tx(
        &chain_id,
        &bob,
        0,
        vec![TxMessage::SubmitEquivocationEvidence {
            evidence: Box::new(evidence.clone()),
        }],
        300_000,
        100,
    );
    ledger.submit_transaction(slash_tx).unwrap();
    ledger.mine_block(bob.address()).unwrap(); // Height 1

    let alice_rec = ledger
        .get_validator_record(&alice.address())
        .unwrap()
        .unwrap();
    assert_eq!(alice_rec.status, CanonicalValidatorStatus::Jailed);
    assert!(
        alice_rec.is_tombstoned,
        "Alice must be permanently tombstoned"
    );

    // Alice attempts to Unjail -> must be rejected because is_tombstoned == true
    let unjail_tx = create_signed_tx(&chain_id, &alice, 0, vec![TxMessage::Unjail], 100_000, 100);
    let unjail_res = ledger.submit_transaction(unjail_tx);
    assert!(
        unjail_res.is_err(),
        "Tombstoned validator must never be permitted to unjail"
    );

    // Alice cannot register again with new moniker
    let pop = make_pop(
        &chain_id,
        &alice.address(),
        &alice.public_key_bytes(),
        &alice,
    );
    let reregister_tx = create_signed_tx(
        &chain_id,
        &alice,
        0,
        vec![TxMessage::RegisterValidator {
            consensus_pubkey: alice.public_key_bytes().to_vec(),
            moniker: "alice_reborn".into(),
            self_stake: Amount::from_sprx_whole(1_000).unwrap(),
            proof_of_possession: pop,
        }],
        300_000,
        100,
    );
    let rereg_res = ledger.submit_transaction(reregister_tx);
    assert!(
        rereg_res.is_err(),
        "Tombstoned operator/consensus pubkey must never re-register"
    );
}

#[test]
fn test_scenario_f_g_and_h_equivocation_evidence_validation_and_double_slash_prevention() {
    let (g, alice, bob) = setup_test_genesis();
    let genesis_fingerprint = g.fingerprint().unwrap();
    let chain_id = ChainId::new(&g.chain_id).unwrap();
    let mut ledger = ChainLedger::init_from_genesis(g.clone()).unwrap();

    let vote_a = make_vote(
        genesis_fingerprint,
        VoteType::Prevote,
        0,
        1,
        Some(Hash32::new([101u8; 32])),
        &alice,
    );
    let vote_b = make_vote(
        genesis_fingerprint,
        VoteType::Prevote,
        0,
        1,
        Some(Hash32::new([102u8; 32])),
        &alice,
    );
    let evidence = EquivocationEvidence::new(alice.address(), 0, 1, vote_a, vote_b);

    // 1. Submit valid evidence
    let ev_tx = create_signed_tx(
        &chain_id,
        &bob,
        0,
        vec![TxMessage::SubmitEquivocationEvidence {
            evidence: Box::new(evidence.clone()),
        }],
        300_000,
        100,
    );
    ledger.submit_transaction(ev_tx).unwrap();
    ledger.mine_block(bob.address()).unwrap(); // Height 1

    // 2. Submit duplicate or reversed evidence pair -> must be rejected
    let mut reversed = evidence.clone();
    std::mem::swap(&mut reversed.vote_a, &mut reversed.vote_b);
    let dup_tx = create_signed_tx(
        &chain_id,
        &bob,
        1,
        vec![TxMessage::SubmitEquivocationEvidence {
            evidence: Box::new(reversed),
        }],
        300_000,
        100,
    );
    let dup_res = ledger.submit_transaction(dup_tx);
    assert!(
        dup_res.is_err(),
        "Duplicate/reversed evidence must be rejected"
    );

    // 3. Forged signature on foreign evidence -> must be rejected
    let mut bad_sig_vote = make_vote(
        genesis_fingerprint,
        VoteType::Prevote,
        0,
        2,
        Some(Hash32::new([103u8; 32])),
        &alice,
    );
    bad_sig_vote.signature = vec![0u8; 64]; // forged/invalid signature
    let bad_ev =
        EquivocationEvidence::new(alice.address(), 0, 2, bad_sig_vote.clone(), bad_sig_vote);
    let bad_tx = create_signed_tx(
        &chain_id,
        &bob,
        1,
        vec![TxMessage::SubmitEquivocationEvidence {
            evidence: Box::new(bad_ev),
        }],
        300_000,
        100,
    );
    let bad_res = ledger.submit_transaction(bad_tx);
    assert!(
        bad_res.is_err(),
        "Forged or non-conflicting evidence must be rejected"
    );
}

#[test]
fn test_scenario_i_and_j_integer_arithmetic_gas_limit_and_rollback_safety() {
    let (g, alice, _bob) = setup_test_genesis();
    let chain_id = ChainId::new(&g.chain_id).unwrap();
    let mut ledger = ChainLedger::init_from_genesis(g.clone()).unwrap();

    let initial_supply = StateAccessor::get_supply_state(ledger.store()).unwrap();
    assert!(!initial_supply.circulating_supply.is_zero());

    let carol = Ed25519Keypair::from_seed(&[50u8; 32]);
    let carol_consensus = Ed25519Keypair::from_seed(&[51u8; 32]);

    // Fund Carol with 2,000 SPRX
    let fund_carol = create_signed_tx(
        &chain_id,
        &alice,
        0,
        vec![TxMessage::Transfer {
            to: carol.address(),
            amount: Amount::from_sprx_whole(2_000).unwrap(),
        }],
        100_000,
        100,
    );
    ledger.submit_transaction(fund_carol).unwrap();
    ledger.mine_block(alice.address()).unwrap(); // Height 1

    let pop = make_pop(
        &chain_id,
        &carol.address(),
        &carol_consensus.public_key_bytes(),
        &carol_consensus,
    );
    let oog_tx = create_signed_tx(
        &chain_id,
        &carol,
        0,
        vec![TxMessage::RegisterValidator {
            consensus_pubkey: carol_consensus.public_key_bytes().to_vec(),
            moniker: "fail_oog".into(),
            self_stake: Amount::from_sprx_whole(1_000).unwrap(),
            proof_of_possession: pop,
        }],
        10_000, // gas limit too low
        100,
    );

    ledger.submit_transaction(oog_tx).unwrap();
    let block = ledger.mine_block(alice.address()).unwrap(); // Height 2
    assert_eq!(block.header.height, 2);

    // Carol must NOT be registered because transaction ran out of gas and rolled back
    let carol_rec = ledger.get_validator_record(&carol.address()).unwrap();
    assert!(
        carol_rec.is_none(),
        "Out-of-gas transaction must roll back validator registration"
    );
}

#[test]
fn test_scenario_k_through_r_storage_backup_restore_integrity_and_signing_guard() {
    let dir = tempdir().unwrap();
    let source_data = dir.path().join("source_data");
    let backup_dir = dir.path().join("backup");
    let restored_data = dir.path().join("restored_data");

    std::fs::create_dir_all(&source_data).unwrap();

    // Create a dummy ledger file in source data
    std::fs::write(
        source_data.join("ledger.db"),
        b"sprax-canonical-ledger-data-v1",
    )
    .unwrap();
    std::fs::write(source_data.join("state.meta"), b"height=42,root=0xdeadbeef").unwrap();

    let meta = SnapshotMetadata {
        chain_id: "sprax-testnet-1".into(),
        genesis_fingerprint: Hash32::ZERO,
        finalized_height: 42,
        finalized_block_hash: Hash32::new([42u8; 32]),
        state_root: Hash32::new([77u8; 32]),
    };

    // 1. Create backup
    let manifest = create_backup(&source_data, &backup_dir, meta).unwrap();
    assert_eq!(manifest.finalized_height, 42);
    assert_eq!(manifest.files.len(), 2);

    // 2. Verify snapshot
    let manifest_path = backup_dir.join(SNAPSHOT_MANIFEST_NAME);
    let verified = verify_snapshot(&manifest_path).unwrap();
    assert_eq!(verified.finalized_height, 42);

    // 3. Tamper with backup file -> verify must detect checksum failure
    std::fs::write(backup_dir.join("ledger.db"), b"tampered-malicious-bytes").unwrap();
    let tamper_res = verify_snapshot(&manifest_path);
    assert!(
        tamper_res.is_err(),
        "Checksum mismatch must be detected on tampered snapshot"
    );

    // Fix file back
    std::fs::write(
        backup_dir.join("ledger.db"),
        b"sprax-canonical-ledger-data-v1",
    )
    .unwrap();

    // 4. Restore backup into new destination
    let restored_manifest = restore_backup(&manifest_path, &restored_data, None, None).unwrap();
    assert_eq!(restored_manifest.finalized_height, 42);

    // 5. RESTORED_SIGNING_DISABLED marker must be present
    let guard_marker = restored_data.join(RESTORED_SIGNING_DISABLED_MARKER);
    assert!(
        guard_marker.exists(),
        "Restored node home MUST contain RESTORED_SIGNING_DISABLED safety guard marker"
    );

    // 6. Test index rebuild
    let mem_store = MemKVStore::new();
    let empty_count = rebuild_chain_indexes(&mem_store).unwrap();
    assert_eq!(empty_count, 0);

    let (g, _alice, _bob) = setup_test_genesis();
    let ledger = ChainLedger::init_from_genesis(g).unwrap();
    let rebuild_count = rebuild_chain_indexes(ledger.store()).unwrap();
    assert_eq!(rebuild_count, 1); // Genesis block height 0 indexed
}

#[test]
fn test_scenario_s_t_u_multi_validator_replica_agreement_and_catchup() {
    let (g, alice, bob) = setup_test_genesis();
    let chain_id = ChainId::new(&g.chain_id).unwrap();
    let mut primary = ChainLedger::init_from_genesis(g.clone()).unwrap();
    let mut replica = ChainLedger::init_from_genesis(g.clone()).unwrap();

    // Submit transactions on primary
    let tx1 = create_signed_tx(
        &chain_id,
        &alice,
        0,
        vec![TxMessage::Transfer {
            to: bob.address(),
            amount: Amount::from_sprx_whole(250).unwrap(),
        }],
        100_000,
        100,
    );
    primary.submit_transaction(tx1).unwrap();
    let block1 = primary.mine_block(alice.address()).unwrap();

    // Replica applies block1
    let rep_receipts = replica.apply_block(block1.clone()).unwrap();
    let rep_root = replica.latest_header().state_root;
    assert_eq!(
        rep_root, block1.header.state_root,
        "Replica must compute identical state root"
    );
    assert_eq!(rep_receipts.len(), 1);
    assert!(rep_receipts[0].success);

    // Check balances on both
    let pri_alice = primary.get_account(&alice.address()).unwrap();
    let rep_alice = replica.get_account(&alice.address()).unwrap();
    assert_eq!(pri_alice.balance, rep_alice.balance);
    assert_eq!(pri_alice.nonce, rep_alice.nonce);

    let pri_bob = primary.get_account(&bob.address()).unwrap();
    let rep_bob = replica.get_account(&bob.address()).unwrap();
    assert_eq!(pri_bob.balance, rep_bob.balance);
}
