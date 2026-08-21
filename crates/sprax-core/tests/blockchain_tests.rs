use sprax_core::{
    executor::TxExecutor,
    genesis::GenesisConfig,
    ledger::ChainLedger,
    state::{AccountState, StateAccessor},
};
use sprax_crypto::{Ed25519Keypair, Secp256k1Keypair};
use sprax_storage::MemKVStore;
use sprax_types::{
    Address, Amount, Block, BlockBody, BlockHeader, ChainId, Hash32, KeyType, Transaction, TxBody,
    TxFee, TxMessage,
};

#[test]
fn test_scenario_1_valid_transaction_and_state_transition() {
    let alice_kp = Ed25519Keypair::generate();
    let bob_kp = Ed25519Keypair::generate();
    let alice_addr = alice_kp.address();
    let bob_addr = bob_kp.address();

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = alice_addr;
    genesis.accounts[0].initial_balance = Amount::from_sprx_whole(1_000).unwrap();

    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();
    assert_eq!(
        ledger.get_account(&alice_addr).unwrap().balance,
        Amount::from_sprx_whole(1_000).unwrap()
    );
    assert_eq!(ledger.get_account(&bob_addr).unwrap().balance, Amount::ZERO);

    let transfer_amount = Amount::from_sprx_whole(300).unwrap();
    let fee = TxFee::default();

    let tx_body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: alice_addr,
        nonce: 0,
        messages: vec![TxMessage::Transfer {
            to: bob_addr,
            amount: transfer_amount,
        }],
        fee: fee.clone(),
        memo: "valid transfer".into(),
        timeout_height: 10,
    };

    let sign_bytes = tx_body.sign_bytes().unwrap();
    let sig = alice_kp.sign(&sign_bytes);
    let tx = Transaction::new(
        tx_body,
        KeyType::Ed25519,
        alice_kp.public_key_bytes().to_vec(),
        sig,
    )
    .unwrap();

    let tx_hash = ledger.submit_transaction(tx).unwrap();
    let block = ledger.mine_block(Address::new([9u8; 20])).unwrap();

    assert_eq!(block.header.height, 1);
    assert_eq!(block.body.transactions.len(), 1);

    // Verify Bob received 300 SPRX
    let bob_state = ledger.get_account(&bob_addr).unwrap();
    assert_eq!(bob_state.balance, transfer_amount);

    // Verify Alice was deducted 300 SPRX + fee, and nonce incremented to 1
    let alice_state = ledger.get_account(&alice_addr).unwrap();
    assert_eq!(alice_state.nonce, 1);
    let expected_alice_bal = Amount::from_sprx_whole(1_000)
        .unwrap()
        .checked_sub(transfer_amount)
        .unwrap()
        .checked_sub(fee.amount)
        .unwrap();
    assert_eq!(alice_state.balance, expected_alice_bal);

    // Verify receipt lookup
    let (_, receipt, height) = ledger.get_transaction(&tx_hash).unwrap();
    assert_eq!(height, 1);
    assert!(receipt.success);
}

#[test]
fn test_scenario_2_invalid_signature_rejection() {
    let alice_kp = Ed25519Keypair::generate();
    let eve_kp = Ed25519Keypair::generate();
    let alice_addr = alice_kp.address();
    let bob_addr = Address::new([2u8; 20]);

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = alice_addr;
    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();

    let tx_body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: alice_addr,
        nonce: 0,
        messages: vec![TxMessage::Transfer {
            to: bob_addr,
            amount: Amount::from_sprx_whole(50).unwrap(),
        }],
        fee: TxFee::default(),
        memo: "".into(),
        timeout_height: 10,
    };

    let sign_bytes = tx_body.sign_bytes().unwrap();
    // Eve signs instead of Alice!
    let forged_sig = eve_kp.sign(&sign_bytes);

    let tx = Transaction::new(
        tx_body,
        KeyType::Ed25519,
        alice_kp.public_key_bytes().to_vec(),
        forged_sig,
    )
    .unwrap();

    let res = ledger.submit_transaction(tx);
    assert!(
        res.is_err(),
        "transaction with forged signature must be rejected"
    );
}

#[test]
fn test_scenario_3_invalid_nonce_rejection() {
    let alice_kp = Ed25519Keypair::generate();
    let alice_addr = alice_kp.address();
    let bob_addr = Address::new([2u8; 20]);

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = alice_addr;
    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();

    // Alice account nonce is 0, but tx has nonce 5
    let tx_body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: alice_addr,
        nonce: 5,
        messages: vec![TxMessage::Transfer {
            to: bob_addr,
            amount: Amount::from_sprx_whole(10).unwrap(),
        }],
        fee: TxFee::default(),
        memo: "".into(),
        timeout_height: 10,
    };

    let sign_bytes = tx_body.sign_bytes().unwrap();
    let sig = alice_kp.sign(&sign_bytes);
    let tx = Transaction::new(
        tx_body,
        KeyType::Ed25519,
        alice_kp.public_key_bytes().to_vec(),
        sig,
    )
    .unwrap();

    let res = ledger.submit_transaction(tx);
    assert!(
        res.is_err(),
        "transaction with future or mismatched nonce must be rejected"
    );
}

#[test]
fn test_scenario_4_insufficient_balance_rejection() {
    let alice_kp = Ed25519Keypair::generate();
    let alice_addr = alice_kp.address();
    let bob_addr = Address::new([2u8; 20]);

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = alice_addr;
    genesis.accounts[0].initial_balance = Amount::from_sprx_whole(100).unwrap();
    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();

    // Alice has 100 SPRX, but tries to transfer 500 SPRX
    let tx_body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: alice_addr,
        nonce: 0,
        messages: vec![TxMessage::Transfer {
            to: bob_addr,
            amount: Amount::from_sprx_whole(500).unwrap(),
        }],
        fee: TxFee::default(),
        memo: "".into(),
        timeout_height: 10,
    };

    let sign_bytes = tx_body.sign_bytes().unwrap();
    let sig = alice_kp.sign(&sign_bytes);
    let tx = Transaction::new(
        tx_body,
        KeyType::Ed25519,
        alice_kp.public_key_bytes().to_vec(),
        sig,
    )
    .unwrap();

    let executor = TxExecutor::default();
    let res = executor.execute_transaction(&MemKVStore::new(), &tx, 1, "sprax-devnet-1");
    assert!(
        res.is_err(),
        "tx exceeding sender balance must fail execution"
    );

    let ledger_res = ledger.submit_transaction(tx);
    assert!(
        ledger_res.is_err(),
        "tx exceeding sender balance must fail submission"
    );
}

#[test]
fn test_scenario_5_invalid_amount_rejection() {
    let alice_kp = Ed25519Keypair::generate();
    let alice_addr = alice_kp.address();
    let bob_addr = Address::new([2u8; 20]);

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = alice_addr;
    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();

    // Zero transfer amount
    let tx_body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: alice_addr,
        nonce: 0,
        messages: vec![TxMessage::Transfer {
            to: bob_addr,
            amount: Amount::ZERO,
        }],
        fee: TxFee::default(),
        memo: "".into(),
        timeout_height: 10,
    };

    let sign_bytes = tx_body.sign_bytes().unwrap();
    let sig = alice_kp.sign(&sign_bytes);
    let tx = Transaction::new(
        tx_body,
        KeyType::Ed25519,
        alice_kp.public_key_bytes().to_vec(),
        sig,
    )
    .unwrap();

    let res = ledger.submit_transaction(tx);
    assert!(res.is_err(), "zero amount transfer must be rejected");
}

#[test]
fn test_scenario_6_replay_attempt_prevention() {
    let alice_kp = Ed25519Keypair::generate();
    let alice_addr = alice_kp.address();
    let bob_addr = Address::new([2u8; 20]);

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = alice_addr;
    genesis.accounts[0].initial_balance = Amount::from_sprx_whole(1_000).unwrap();
    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();

    let tx_body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: alice_addr,
        nonce: 0,
        messages: vec![TxMessage::Transfer {
            to: bob_addr,
            amount: Amount::from_sprx_whole(100).unwrap(),
        }],
        fee: TxFee::default(),
        memo: "".into(),
        timeout_height: 10,
    };

    let sign_bytes = tx_body.sign_bytes().unwrap();
    let sig = alice_kp.sign(&sign_bytes);
    let tx = Transaction::new(
        tx_body,
        KeyType::Ed25519,
        alice_kp.public_key_bytes().to_vec(),
        sig,
    )
    .unwrap();

    // 1st submission succeeds and gets mined
    ledger.submit_transaction(tx.clone()).unwrap();
    ledger.mine_block(Address::new([9u8; 20])).unwrap();

    // 2nd submission of the exact same tx MUST fail
    let replay_res = ledger.submit_transaction(tx);
    assert!(
        replay_res.is_err(),
        "replaying committed transaction must fail"
    );
}

#[test]
fn test_scenario_7_invalid_block_structure_rejection() {
    // Attempting to create non-genesis block with parent_hash = ZERO must fail constructor
    let header = BlockHeader {
        version: 1,
        chain_id: "sprax-devnet-1".into(),
        height: 5,
        timestamp_unix_secs: 1_700_000_010,
        parent_hash: Hash32::ZERO, // Invalid!
        proposer: Address::new([1u8; 20]),
        state_root: Hash32::ZERO,
        txs_root: Hash32::ZERO,
        receipts_root: Hash32::ZERO,
        validator_set_hash: Hash32::ZERO,
    };

    let block_res = Block::new(header, BlockBody::default(), vec![]);
    assert!(
        block_res.is_err(),
        "non-genesis block with zero parent hash must be rejected"
    );
}

#[test]
fn test_scenario_8_secp256k1_transaction_execution() {
    let store = MemKVStore::new();
    let alice_secp = Secp256k1Keypair::generate();
    let alice_addr = alice_secp.address();
    let bob_addr = Address::new([5u8; 20]);

    // Fund Alice
    let alice_init = AccountState {
        nonce: 0,
        balance: Amount::from_sprx_whole(500).unwrap(),
        code_hash: Hash32::ZERO,
        storage_root: Hash32::ZERO,
    };
    StateAccessor::set_account(&store, &alice_addr, &alice_init).unwrap();

    let tx_body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: alice_addr,
        nonce: 0,
        messages: vec![TxMessage::Transfer {
            to: bob_addr,
            amount: Amount::from_sprx_whole(100).unwrap(),
        }],
        fee: TxFee::default(),
        memo: "secp256k1 transfer".into(),
        timeout_height: 10,
    };

    let sign_bytes = tx_body.sign_bytes().unwrap();
    let sig = alice_secp.sign(&sign_bytes);
    let tx = Transaction::new(
        tx_body,
        KeyType::Secp256k1,
        alice_secp.public_key_bytes(),
        sig,
    )
    .unwrap();

    let executor = TxExecutor::default();
    let receipt = executor
        .execute_transaction(&store, &tx, 1, "sprax-devnet-1")
        .unwrap();
    assert!(receipt.success);

    let bob_state = StateAccessor::get_account(&store, &bob_addr).unwrap();
    assert_eq!(bob_state.balance, Amount::from_sprx_whole(100).unwrap());
}
