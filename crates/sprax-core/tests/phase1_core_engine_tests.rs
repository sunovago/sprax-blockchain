use sprax_core::{
    genesis::GenesisConfig,
    ledger::ChainLedger,
    state::{AccountState, StateAccessor},
};
use sprax_crypto::{Ed25519Keypair, MerkleTree, Secp256k1Keypair, INTERNAL_PREFIX, LEAF_PREFIX};
use sprax_storage::{KVStore, MemKVStore, OverlayStore, ReadonlyKVStore};
use sprax_types::{
    Address, Amount, ChainId, Hash32, KeyType, Transaction, TxBody, TxFee, TxMessage,
};

fn helper_create_signed_tx(
    keypair: &Ed25519Keypair,
    chain_id: &str,
    nonce: u64,
    recipient: Address,
    amount: Amount,
) -> Transaction {
    let body = TxBody {
        chain_id: ChainId::new(chain_id).unwrap(),
        sender: keypair.address(),
        nonce,
        messages: vec![TxMessage::Transfer {
            to: recipient,
            amount,
        }],
        fee: TxFee::default(),
        memo: "phase-1 test transaction".into(),
        timeout_height: 0,
    };
    let sign_bytes = body.sign_bytes().expect("sign bytes");
    let signature = keypair.sign(&sign_bytes);
    Transaction::new(
        body,
        KeyType::Ed25519,
        keypair.public_key_bytes().to_vec(),
        signature,
    )
    .expect("valid tx envelope")
}

#[test]
fn test_merkle_tree_engine_proofs_and_domain_separation() {
    // 1. Verify Leaf and Internal node domain separation prefixes
    assert_eq!(LEAF_PREFIX, 0x00);
    assert_eq!(INTERNAL_PREFIX, 0x01);

    // 2. Leaf and internal node collision resistance (second-preimage defense)
    let payload = b"transaction_payload_data";
    let leaf_h = MerkleTree::hash_leaf(payload);
    let int_h = MerkleTree::hash_internal(Hash32::ZERO, Hash32::ZERO);
    assert_ne!(leaf_h, int_h);

    // 3. Merkle tree construction with 8 leaves
    let raw_leaves: Vec<Vec<u8>> = (0..8).map(|i| format!("leaf_{i}").into_bytes()).collect();
    let leaf_hashes: Vec<Hash32> = raw_leaves
        .iter()
        .map(|l| MerkleTree::hash_leaf(l))
        .collect();
    let tree = MerkleTree::from_hashes(&leaf_hashes);

    assert_eq!(tree.leaf_count(), 8);
    let root = tree.root();
    assert_ne!(root, Hash32::ZERO);

    // 4. Generate and verify proof for every single leaf
    for (idx, hash) in leaf_hashes.iter().enumerate() {
        let proof = tree.generate_proof(idx).expect("proof exists");
        assert_eq!(proof.leaf_index, idx);
        assert_eq!(proof.total_leaves, 8);
        assert!(proof.verify(root, *hash));

        // Tampered leaf hash must fail
        assert!(!proof.verify(root, Hash32::ZERO));
        // Wrong root must fail
        assert!(!proof.verify(Hash32::ZERO, *hash));
    }

    // 5. Odd-sized tree (5 leaves)
    let odd_hashes: Vec<Hash32> = leaf_hashes[..5].to_vec();
    let odd_tree = MerkleTree::from_hashes(&odd_hashes);
    assert_eq!(odd_tree.leaf_count(), 5);
    let odd_root = odd_tree.root();
    for (idx, hash) in odd_hashes.iter().enumerate() {
        let proof = odd_tree.generate_proof(idx).expect("proof exists");
        assert!(proof.verify(odd_root, *hash));
    }
}

#[test]
fn test_canonical_rfc8785_serialization_determinism() {
    let keypair = Ed25519Keypair::generate();
    let recipient = Address::new([42u8; 20]);
    let body = TxBody {
        chain_id: ChainId::new("sprax-testnet-1").unwrap(),
        sender: keypair.address(),
        nonce: 10,
        messages: vec![TxMessage::Transfer {
            to: recipient,
            amount: Amount::from_atto(5_000_000),
        }],
        fee: TxFee::default(),
        memo: "canonical test".into(),
        timeout_height: 1000,
    };

    let bytes1 = body.sign_bytes().expect("canonical serialize 1");
    let bytes2 = body.sign_bytes().expect("canonical serialize 2");
    assert_eq!(bytes1, bytes2);

    let json_str = String::from_utf8(bytes1).expect("utf8 string");
    // Verify whitespace-free
    assert!(!json_str.contains(": "));
    assert!(!json_str.contains(", "));
    // Verify key sorting: "chain_id" appears before "fee", "fee" before "memo", "memo" before "messages"
    let chain_id_pos = json_str.find("\"chain_id\"").unwrap();
    let fee_pos = json_str.find("\"fee\"").unwrap();
    let memo_pos = json_str.find("\"memo\"").unwrap();
    let messages_pos = json_str.find("\"messages\"").unwrap();
    let nonce_pos = json_str.find("\"nonce\"").unwrap();
    let sender_pos = json_str.find("\"sender\"").unwrap();
    let timeout_pos = json_str.find("\"timeout_height\"").unwrap();

    assert!(chain_id_pos < fee_pos);
    assert!(fee_pos < memo_pos);
    assert!(memo_pos < messages_pos);
    assert!(messages_pos < nonce_pos);
    assert!(nonce_pos < sender_pos);
    assert!(sender_pos < timeout_pos);
}

#[test]
fn test_account_invariants_and_checked_arithmetic() {
    let store = MemKVStore::new();
    let addr = Address::new([7u8; 20]);

    // Initial state: not found returns 0 balance and 0 nonce
    let init_state = StateAccessor::get_account(&store, &addr).unwrap();
    assert_eq!(init_state.balance, Amount::ZERO);
    assert_eq!(init_state.nonce, 0);

    // Balance addition
    let funded = Amount::from_sprx_whole(100).unwrap();
    let new_state = AccountState {
        nonce: 0,
        balance: funded,
        code_hash: Hash32::ZERO,
        storage_root: Hash32::ZERO,
    };
    StateAccessor::set_account(&store, &addr, &new_state).unwrap();

    let fetched = StateAccessor::get_account(&store, &addr).unwrap();
    assert_eq!(fetched.balance, funded);

    // Overflow protection on Amount
    let max_amt = Amount::from_atto(u128::MAX);
    assert!(max_amt.checked_add(Amount::from_atto(1)).is_err());

    // Underflow protection on Amount
    assert!(Amount::from_atto(10)
        .checked_sub(Amount::from_atto(20))
        .is_err());
}

#[test]
fn test_transaction_signing_tampering_and_verification() {
    let alice = Ed25519Keypair::generate();
    let bob = Ed25519Keypair::generate();
    let charlie = Secp256k1Keypair::generate();

    // 1. Valid Ed25519 transaction
    let mut tx_ed = helper_create_signed_tx(
        &alice,
        "sprax-devnet",
        0,
        bob.address(),
        Amount::from_atto(1000),
    );
    let sign_bytes_ed = tx_ed.sign_bytes().unwrap();
    assert!(Ed25519Keypair::verify(&tx_ed.public_key, &sign_bytes_ed, &tx_ed.signature).is_ok());

    // 2. Tampered body message rejected
    tx_ed.body.messages = vec![TxMessage::Transfer {
        to: bob.address(),
        amount: Amount::from_atto(999_999),
    }];
    let tampered_bytes = tx_ed.sign_bytes().unwrap();
    assert!(Ed25519Keypair::verify(&tx_ed.public_key, &tampered_bytes, &tx_ed.signature).is_err());

    // 3. Valid Secp256k1 transaction
    let body_secp = TxBody {
        chain_id: ChainId::new("sprax-devnet").unwrap(),
        sender: charlie.address(),
        nonce: 0,
        messages: vec![TxMessage::Transfer {
            to: alice.address(),
            amount: Amount::from_atto(500),
        }],
        fee: TxFee::default(),
        memo: "secp test".into(),
        timeout_height: 0,
    };
    let sign_bytes_secp = body_secp.sign_bytes().unwrap();
    let sig_secp = charlie.sign(&sign_bytes_secp);
    let tx_secp = Transaction::new(
        body_secp,
        KeyType::Secp256k1,
        charlie.public_key_bytes(),
        sig_secp,
    )
    .unwrap();
    assert!(
        Secp256k1Keypair::verify(&tx_secp.public_key, &sign_bytes_secp, &tx_secp.signature).is_ok()
    );

    // 4. Tampered Secp256k1 signature rejected
    let mut bad_sig = tx_secp.signature.clone();
    bad_sig[0] ^= 0xFF;
    assert!(Secp256k1Keypair::verify(&tx_secp.public_key, &sign_bytes_secp, &bad_sig).is_err());
}

#[test]
fn test_atomic_state_overlay_commit_and_rollback() {
    let base_store = MemKVStore::new();
    let key = b"sprax/test/account/alice";

    // 1. Initial write to base store
    base_store.set(key, b"state_v1").unwrap();
    assert_eq!(base_store.get(key).unwrap().unwrap(), b"state_v1");

    // 2. Overlay modification without commit
    let overlay = OverlayStore::new(base_store.clone());
    overlay.set(key, b"state_v2_uncommitted").unwrap();
    assert_eq!(overlay.get(key).unwrap().unwrap(), b"state_v2_uncommitted");
    // Base store remains unchanged
    assert_eq!(base_store.get(key).unwrap().unwrap(), b"state_v1");

    // 3. Drop overlay (rollback)
    drop(overlay);
    assert_eq!(base_store.get(key).unwrap().unwrap(), b"state_v1");

    // 4. Overlay commit_state
    let overlay2 = OverlayStore::new(base_store.clone());
    overlay2.set(key, b"state_v3_committed").unwrap();
    overlay2.commit_state().unwrap();
    assert_eq!(base_store.get(key).unwrap().unwrap(), b"state_v3_committed");
}

#[test]
fn test_block_creation_validation_and_chain_integrity_audit() {
    let alice_kp = Ed25519Keypair::generate();
    let bob_kp = Ed25519Keypair::generate();
    let proposer = Address::new([99u8; 20]);

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = alice_kp.address();
    genesis.accounts[0].initial_balance = Amount::from_sprx_whole(10_000).unwrap();

    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();
    assert_eq!(ledger.height(), 0);

    // Run integrity audit on genesis state
    let genesis_audit = ledger.verify_chain_integrity().unwrap();
    assert!(genesis_audit.is_valid);
    assert_eq!(genesis_audit.total_blocks_verified, 1);
    assert_eq!(genesis_audit.violations.len(), 0);

    // Submit transaction
    let tx = helper_create_signed_tx(
        &alice_kp,
        ledger.chain_id(),
        0,
        bob_kp.address(),
        Amount::from_sprx_whole(10).unwrap(),
    );
    let tx_hash = ledger.submit_transaction(tx).unwrap();

    // Mine block 1
    let block_1 = ledger.mine_block(proposer).unwrap();
    assert_eq!(block_1.header.height, 1);
    assert_eq!(block_1.body.transactions.len(), 1);

    // Verify receipt was generated and stored
    let (_tx_ref, receipt, h) = ledger.get_transaction(&tx_hash).expect("tx indexed");
    assert_eq!(h, 1);
    assert!(receipt.success);
    assert_eq!(receipt.tx_hash, tx_hash);
    assert_eq!(receipt.height, 1);
    assert!(receipt.gas_used > 0);

    // Verify chain integrity passes with block 1
    let audit_h1 = ledger.verify_chain_integrity().unwrap();
    assert!(audit_h1.is_valid);
    assert_eq!(audit_h1.total_blocks_verified, 2);
    assert_eq!(audit_h1.total_transactions_verified, 1);
    assert!(audit_h1.violations.is_empty());

    // Mine block 2 with another transaction
    let tx2 = helper_create_signed_tx(
        &alice_kp,
        ledger.chain_id(),
        1,
        bob_kp.address(),
        Amount::from_sprx_whole(5).unwrap(),
    );
    ledger.submit_transaction(tx2).unwrap();
    let block_2 = ledger.mine_block(proposer).unwrap();
    assert_eq!(block_2.header.height, 2);

    let audit_h2 = ledger.verify_chain_integrity().unwrap();
    assert!(audit_h2.is_valid);
    assert_eq!(audit_h2.total_blocks_verified, 3);
    assert_eq!(audit_h2.total_transactions_verified, 2);
}

#[test]
fn test_chain_integrity_verifier_detects_tampering() {
    let alice_kp = Ed25519Keypair::generate();
    let bob_kp = Ed25519Keypair::generate();
    let proposer = Address::new([99u8; 20]);

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = alice_kp.address();
    genesis.accounts[0].initial_balance = Amount::from_sprx_whole(10_000).unwrap();

    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();

    let tx = helper_create_signed_tx(
        &alice_kp,
        ledger.chain_id(),
        0,
        bob_kp.address(),
        Amount::from_sprx_whole(10).unwrap(),
    );
    ledger.submit_transaction(tx).unwrap();
    let block_1 = ledger.mine_block(proposer).unwrap();

    // Verify clean chain passes
    let audit_clean = ledger.verify_chain_integrity().unwrap();
    assert!(audit_clean.is_valid);

    // 1. Block with invalid parent hash at height 2 is rejected by apply_block
    let mut bad_block_2 = block_1;
    bad_block_2.header.height = 2;
    bad_block_2.header.parent_hash = Hash32::new([0xDE; 32]); // Wrong parent hash
    bad_block_2.header.timestamp_unix_secs += 5;
    let rejection = ledger.apply_block(bad_block_2);
    assert!(rejection.is_err());
}
