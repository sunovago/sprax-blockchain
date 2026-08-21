use sprax_core::{genesis::GenesisConfig, ledger::ChainLedger};
use sprax_crypto::Ed25519Keypair;
use sprax_network::{message::NetworkMessage, peer::PeerId, swarm::SwarmHub};
use sprax_storage::MemKVStore;
use sprax_types::{
    Address, Amount, ChainId, Hash32, KeyType, Transaction, TxBody, TxFee, TxMessage,
};
use std::time::Duration;

#[tokio::test]
async fn test_three_node_network_gossip_and_synchronization() {
    let hub = SwarmHub::new();

    let p1 = PeerId::new("node-1").unwrap();
    let p2 = PeerId::new("node-2").unwrap();
    let p3 = PeerId::new("node-3").unwrap();

    let _ep1 = hub.register_node(p1.clone());
    let ep2 = hub.register_node(p2.clone());
    let ep3 = hub.register_node(p3.clone());

    // 1. Initialize 3 independent chain ledgers with identical genesis where Alice is funded
    let alice_kp = Ed25519Keypair::from_seed(&[1u8; 32]);
    let alice_addr = alice_kp.address();

    let mut genesis = GenesisConfig::default_development();
    genesis.accounts[0].address = alice_addr;

    let mut ledger1 = ChainLedger::init_from_genesis(genesis.clone()).unwrap();
    let mut ledger2 = ChainLedger::init_from_genesis(genesis.clone()).unwrap();
    let mut ledger3 = ChainLedger::init_from_genesis(genesis).unwrap();

    assert_eq!(ledger1.height(), 0);
    assert_eq!(ledger2.height(), 0);
    assert_eq!(ledger3.height(), 0);

    // 2. Node 1 generates a transaction and broadcasts it over the swarm
    let bob_kp = Ed25519Keypair::from_seed(&[2u8; 32]);
    let bob_addr = bob_kp.address();

    let tx_body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: alice_addr,
        nonce: 0,
        messages: vec![TxMessage::Transfer {
            to: bob_addr,
            amount: Amount::from_sprx_whole(1_000).unwrap(),
        }],
        fee: TxFee::default(),
        memo: "multinode transfer".into(),
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

    // Node 1 submits locally and broadcasts
    let tx_hash = ledger1.submit_transaction(tx.clone()).unwrap();
    let broadcast_count = hub
        .broadcast(&p1, NetworkMessage::TxGossip(tx.clone()))
        .await
        .unwrap();
    assert_eq!(broadcast_count, 2); // Delivered to Node 2 and Node 3

    // Wait for nodes 2 and 3 to receive the tx
    tokio::time::sleep(Duration::from_millis(50)).await;
    let mut rx2 = ep2.inbound_tx_rx.lock().await;
    let mut rx3 = ep3.inbound_tx_rx.lock().await;

    let received_tx2 = rx2.try_recv().expect("node 2 should receive tx");
    let received_tx3 = rx3.try_recv().expect("node 3 should receive tx");
    assert_eq!(received_tx2.body.sender, alice_addr);
    assert_eq!(received_tx3.body.sender, alice_addr);

    // Admit received txs into Node 2 and Node 3 mempools
    ledger2.submit_transaction(received_tx2).unwrap();
    ledger3.submit_transaction(received_tx3).unwrap();

    // 3. Node 1 mines Block #1 and broadcasts BlockGossip
    let block1 = ledger1.mine_block(alice_addr).unwrap();
    assert_eq!(ledger1.height(), 1);

    hub.broadcast(&p1, NetworkMessage::BlockGossip(block1.clone()))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;
    let mut block_rx2 = ep2.inbound_block_rx.lock().await;
    let mut block_rx3 = ep3.inbound_block_rx.lock().await;

    let received_block2 = block_rx2.try_recv().expect("node 2 should receive block");
    let received_block3 = block_rx3.try_recv().expect("node 3 should receive block");

    // Node 2 and Node 3 apply Block #1
    ledger2.apply_block(received_block2).unwrap();
    ledger3.apply_block(received_block3).unwrap();

    // 4. Verify all 3 nodes are in identical state at Height 1
    assert_eq!(ledger1.height(), 1);
    assert_eq!(ledger2.height(), 1);
    assert_eq!(ledger3.height(), 1);

    assert_eq!(ledger1.state_root().unwrap(), ledger2.state_root().unwrap());
    assert_eq!(ledger2.state_root().unwrap(), ledger3.state_root().unwrap());

    let (t, r, h) = ledger2.get_transaction(&tx_hash).unwrap();
    assert_eq!(h, 1);
    assert!(r.success);
    assert_eq!(t.body.sender, alice_addr);
}

#[tokio::test]
async fn test_network_partition_and_healing() {
    let hub = SwarmHub::new();

    let p1 = PeerId::new("node-1").unwrap();
    let p2 = PeerId::new("node-2").unwrap();

    let _ep1 = hub.register_node(p1.clone());
    let ep2 = hub.register_node(p2.clone());

    // Partition Node 1 and Node 2
    hub.partition_peers(p1.clone(), p2.clone());

    let dummy_tx = Transaction::new(
        TxBody {
            chain_id: ChainId::new("sprax-devnet-1").unwrap(),
            sender: Address::new([1u8; 20]),
            nonce: 0,
            messages: vec![TxMessage::Transfer {
                to: Address::new([2u8; 20]),
                amount: Amount::from_atto(100),
            }],
            fee: TxFee::default(),
            memo: "".into(),
            timeout_height: 10,
        },
        KeyType::Ed25519,
        vec![1; 32],
        vec![2; 64],
    )
    .unwrap();

    // Broadcast while partitioned -> 0 delivered
    let delivered = hub
        .broadcast(&p1, NetworkMessage::TxGossip(dummy_tx.clone()))
        .await
        .unwrap();
    assert_eq!(delivered, 0, "partitioned message must not be delivered");

    let mut rx2 = ep2.inbound_tx_rx.lock().await;
    assert!(
        rx2.try_recv().is_err(),
        "node 2 must not receive partitioned message"
    );

    // Heal partition
    hub.heal_partition(&p1, &p2);

    // Broadcast after healing -> delivered!
    let delivered_after = hub
        .broadcast(&p1, NetworkMessage::TxGossip(dummy_tx))
        .await
        .unwrap();
    assert_eq!(
        delivered_after, 1,
        "message must be delivered after partition is healed"
    );

    tokio::time::sleep(Duration::from_millis(50)).await;
    let received = rx2
        .try_recv()
        .expect("node 2 should receive message after healing");
    assert_eq!(received.body.sender, Address::new([1u8; 20]));
}

#[tokio::test]
async fn test_delayed_message_handling() {
    let hub = SwarmHub::new();

    let p1 = PeerId::new("node-1").unwrap();
    let p2 = PeerId::new("node-2").unwrap();

    let _ep1 = hub.register_node(p1.clone());
    let ep2 = hub.register_node(p2.clone());

    hub.set_delay(p2.clone());

    let dummy_tx = Transaction::new(
        TxBody {
            chain_id: ChainId::new("sprax-devnet-1").unwrap(),
            sender: Address::new([5u8; 20]),
            nonce: 0,
            messages: vec![TxMessage::Transfer {
                to: Address::new([6u8; 20]),
                amount: Amount::from_atto(500),
            }],
            fee: TxFee::default(),
            memo: "".into(),
            timeout_height: 10,
        },
        KeyType::Ed25519,
        vec![1; 32],
        vec![2; 64],
    )
    .unwrap();

    hub.broadcast(&p1, NetworkMessage::TxGossip(dummy_tx))
        .await
        .unwrap();

    let mut rx2 = ep2.inbound_tx_rx.lock().await;
    // Immediately, it shouldn't be ready because of the 50ms delay
    assert!(rx2.try_recv().is_err());

    // After waiting for delay, message arrives intact
    tokio::time::sleep(Duration::from_millis(80)).await;
    let received = rx2
        .try_recv()
        .expect("delayed message should arrive after delay interval");
    assert_eq!(received.body.sender, Address::new([5u8; 20]));
}

#[tokio::test]
async fn test_duplicate_message_deduplication() {
    let genesis = GenesisConfig::default_development();
    let alice_kp = Ed25519Keypair::from_seed(&[1u8; 32]);
    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();

    let block1 = ledger.mine_block(alice_kp.address()).unwrap();
    assert_eq!(ledger.height(), 1);

    // Applying duplicate block #1 must be idempotent and succeed without error
    let dup_res = ledger.apply_block(block1);
    assert!(dup_res.is_ok());
    assert_eq!(ledger.height(), 1);
}

#[tokio::test]
async fn test_invalid_block_tamper_rejection() {
    let genesis = GenesisConfig::default_development();
    let alice_kp = Ed25519Keypair::from_seed(&[1u8; 32]);
    let mut ledger = ChainLedger::init_from_genesis(genesis).unwrap();

    let mut block1 = ledger.mine_block(alice_kp.address()).unwrap();

    // Tamper with state root in block header
    block1.header.state_root = Hash32::new([0xfe; 32]);

    let mut receiver_ledger =
        ChainLedger::init_from_genesis(GenesisConfig::default_development()).unwrap();
    let apply_res = receiver_ledger.apply_block(block1);
    assert!(
        apply_res.is_err(),
        "tampered block state root must be rejected"
    );
    assert_eq!(receiver_ledger.height(), 0);
}

#[tokio::test]
async fn test_node_restart_and_state_catchup_sync() {
    let genesis = GenesisConfig::default_development();
    let alice_kp = Ed25519Keypair::from_seed(&[1u8; 32]);

    // 1. Node 1 produces 3 blocks; each is persisted through the store's ChainMetaStore as it's mined.
    let store1 = MemKVStore::new();
    let mut ledger1 = ChainLedger::open_or_init(store1.clone(), || Ok(genesis.clone())).unwrap();
    let b1 = ledger1.mine_block(alice_kp.address()).unwrap();
    let b2 = ledger1.mine_block(alice_kp.address()).unwrap();
    let b3 = ledger1.mine_block(alice_kp.address()).unwrap();
    assert_eq!(ledger1.height(), 3);

    // 2. Node 1 simulates restart: reopen against the same store, rebuilding from persisted
    //    blocks directly (no transaction re-execution) rather than replaying from genesis.
    let reloaded_ledger1 =
        ChainLedger::open_or_init(store1.clone(), || Ok(genesis.clone())).unwrap();
    assert_eq!(reloaded_ledger1.height(), 3);
    assert_eq!(
        reloaded_ledger1.state_root().unwrap(),
        ledger1.state_root().unwrap()
    );

    // 3. Node 2 starts fresh at height 0 and syncs batch [b1, b2, b3]
    let mut ledger2 = ChainLedger::init_from_genesis(genesis).unwrap();
    assert_eq!(ledger2.height(), 0);

    let synced_count = ledger2.apply_blocks_batch(vec![b1, b2, b3]).unwrap();
    assert_eq!(synced_count, 3);
    assert_eq!(ledger2.height(), 3);
    assert_eq!(ledger2.state_root().unwrap(), ledger1.state_root().unwrap());
}
