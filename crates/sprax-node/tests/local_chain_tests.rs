use sprax_node::NodeService;
use sprax_types::{Amount, ChainId, KeyType, Transaction, TxBody, TxFee, TxMessage};

#[tokio::test]
async fn test_local_node_full_lifecycle_and_cli_flow() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let home = temp_dir.path().to_path_buf();

    // 1. Initialize node service and default genesis
    let service = NodeService::new_or_load(home.clone()).expect("init node service");
    assert_eq!(service.height(), 0);
    assert_eq!(service.chain_id(), "sprax-devnet-1");

    let keyring = service.keyring();
    let alice_kp = keyring
        .read()
        .get_ed25519_keypair("alice")
        .expect("get alice key");
    let bob_kp = keyring
        .read()
        .get_ed25519_keypair("bob")
        .expect("get bob key");
    let alice_addr = alice_kp.address();
    let bob_addr = bob_kp.address();

    let alice_init = service.get_account(&alice_addr).expect("get alice account");
    let bob_init = service.get_account(&bob_addr).expect("get bob account");

    assert_eq!(
        alice_init.balance,
        Amount::from_sprx_whole(1_000_000).unwrap()
    );
    assert_eq!(bob_init.balance, Amount::from_sprx_whole(500_000).unwrap());

    // 2. Create a new account in keyring (David)
    let (david_addr, _) = keyring
        .write()
        .create_key("david", KeyType::Ed25519)
        .expect("create david key");
    let david_init = service.get_account(&david_addr).expect("get david account");
    assert_eq!(david_init.balance, Amount::ZERO);

    // 3. Alice sends 25,000 SPRX to David
    let transfer_amount = Amount::from_sprx_whole(25_000).unwrap();
    let fee = TxFee::default();

    let tx_body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: alice_addr,
        nonce: 0,
        messages: vec![TxMessage::Transfer {
            to: david_addr,
            amount: transfer_amount,
        }],
        fee: fee.clone(),
        memo: "funding david".into(),
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

    let tx_hash = service.submit_transaction(tx).expect("submit tx");
    let block = service.mine_block(alice_addr).expect("mine block");

    assert_eq!(block.header.height, 1);
    assert_eq!(service.height(), 1);

    // 4. Verify updated balances
    let david_after = service.get_account(&david_addr).expect("get david after");
    assert_eq!(david_after.balance, transfer_amount);

    let alice_after = service.get_account(&alice_addr).expect("get alice after");
    let expected_alice_bal = Amount::from_sprx_whole(1_000_000)
        .unwrap()
        .checked_sub(transfer_amount)
        .unwrap()
        .checked_sub(fee.amount)
        .unwrap();
    assert_eq!(alice_after.balance, expected_alice_bal);
    assert_eq!(alice_after.nonce, 1);

    // 5. Verify transaction and block queries
    let (queried_tx, receipt, height) = service.get_transaction(&tx_hash).expect("get tx receipt");
    assert_eq!(height, 1);
    assert!(receipt.success);
    assert_eq!(queried_tx.body.sender, alice_addr);

    let queried_block = service.get_block_by_height(1).expect("get block by height");
    assert_eq!(queried_block.header.height, 1);

    // 6. Mine a couple more empty blocks so the redb-backed reload below must reconstruct
    //    multiple heights from the store, not just height 1.
    let _block2 = service.mine_block(alice_addr).expect("mine block 2");
    let _block3 = service.mine_block(alice_addr).expect("mine block 3");
    assert_eq!(service.height(), 3);
    let state_root_before = service.state_root().expect("state root before reload");

    // 7. Test state persistence by reloading node service from disk (redb-backed): this must
    //    rebuild in-memory caches by reading persisted blocks directly, not by re-executing
    //    every historical transaction from genesis.
    drop(service);

    let reloaded_service = NodeService::new_or_load(home.clone()).expect("reload node service");
    assert_eq!(reloaded_service.height(), 3);
    assert_eq!(
        reloaded_service
            .state_root()
            .expect("state root after reload"),
        state_root_before
    );

    let david_persisted = reloaded_service
        .get_account(&david_addr)
        .expect("get david persisted");
    assert_eq!(david_persisted.balance, transfer_amount);

    let alice_persisted = reloaded_service
        .get_account(&alice_addr)
        .expect("get alice persisted");
    assert_eq!(alice_persisted.nonce, 1);

    let reloaded_tx = reloaded_service
        .get_transaction(&tx_hash)
        .expect("tx index reconstructed after reload without re-execution");
    assert_eq!(reloaded_tx.2, 1);
    assert!(reloaded_tx.1.success);
}

#[tokio::test]
async fn test_redb_persistence_round_trip_across_multiple_restarts() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let home = temp_dir.path().to_path_buf();

    let service = NodeService::new_or_load(home.clone()).expect("init node service");
    let keyring = service.keyring();
    let alice_addr = keyring
        .read()
        .get_ed25519_keypair("alice")
        .expect("get alice key")
        .address();

    service.mine_block(alice_addr).expect("mine block 1");
    drop(service);

    let service2 = NodeService::new_or_load(home.clone()).expect("reload after restart 1");
    assert_eq!(service2.height(), 1);
    service2.mine_block(alice_addr).expect("mine block 2");
    drop(service2);

    let service3 = NodeService::new_or_load(home.clone()).expect("reload after restart 2");
    assert_eq!(service3.height(), 2);
}
