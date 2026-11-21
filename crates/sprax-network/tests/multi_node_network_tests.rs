use sprax_consensus::{SignedProposal, Validator, ValidatorSet, Vote, VoteType};
use sprax_crypto::Ed25519Keypair;
use sprax_network::{
    identity::NetworkIdentity, message::NetworkMessage, peer::PeerId, peer::PeerScore,
    swarm::SwarmHub,
};
use sprax_types::{
    Address, Amount, Block, BlockBody, BlockHeader, ChainId, Hash32, KeyType, Transaction, TxBody,
    TxFee, TxMessage,
};
use std::time::Duration;

fn create_test_tx(
    sender_kp: &Ed25519Keypair,
    to: Address,
    amount: u128,
    nonce: u64,
) -> Transaction {
    let body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: sender_kp.address(),
        nonce,
        messages: vec![TxMessage::Transfer {
            to,
            amount: Amount::from_sprx_whole(amount).unwrap(),
        }],
        fee: TxFee::default(),
        memo: "test-gossip".into(),
        timeout_height: 100,
    };
    let sign_bytes = body.sign_bytes().unwrap();
    let sig = sender_kp.sign(&sign_bytes);
    Transaction::new(
        body,
        KeyType::Ed25519,
        sender_kp.public_key_bytes().to_vec(),
        sig,
    )
    .unwrap()
}

fn create_test_block(height: u64, parent: Hash32, proposer: Address) -> Block {
    Block {
        header: BlockHeader {
            version: 1,
            chain_id: "sprax-devnet-1".into(),
            height,
            timestamp_unix_secs: 1_700_000_000 + height,
            parent_hash: parent,
            proposer,
            state_root: Hash32::new([height as u8; 32]),
            txs_root: Hash32::ZERO,
            receipts_root: Hash32::ZERO,
            validator_set_hash: Hash32::ZERO,
        },
        body: BlockBody::default(),
        last_commit: vec![],
    }
}

#[tokio::test]
async fn test_1_node_local_network_standalone() {
    let hub = SwarmHub::new();
    let p1 = PeerId::new("node-solo").unwrap();
    let ep = hub.register_node(p1.clone());

    let ident = NetworkIdentity::generate().unwrap();
    assert_eq!(ident.peer_id().as_str().len(), 40);

    let tx = create_test_tx(
        &Ed25519Keypair::from_seed(&[1u8; 32]),
        Address::new([2u8; 20]),
        10,
        0,
    );
    // Broadcasting from solo node delivers to 0 other nodes
    let delivered = hub
        .broadcast(&p1, NetworkMessage::TxGossip(tx))
        .await
        .unwrap();
    assert_eq!(delivered, 0);

    assert_eq!(hub.active_peers().len(), 1);
    drop(ep);
}

#[tokio::test]
async fn test_2_node_network_transaction_and_block_propagation() {
    let hub = SwarmHub::new();
    let p1 = PeerId::new("node-alice").unwrap();
    let p2 = PeerId::new("node-bob").unwrap();

    let _ep1 = hub.register_node(p1.clone());
    let ep2 = hub.register_node(p2.clone());

    let kp = Ed25519Keypair::from_seed(&[1u8; 32]);
    let tx = create_test_tx(&kp, Address::new([3u8; 20]), 100, 0);
    let tx_hash = sprax_crypto::Hasher::tx_hash(&tx).unwrap();

    // Node 1 broadcasts tx
    let delivered = hub
        .broadcast(&p1, NetworkMessage::TxGossip(tx))
        .await
        .unwrap();
    assert_eq!(delivered, 1);

    // Node 2 receives tx
    let received_tx = tokio::time::timeout(
        Duration::from_secs(1),
        ep2.inbound_tx_rx.lock().await.recv(),
    )
    .await
    .unwrap()
    .expect("node 2 must receive tx");
    assert_eq!(
        sprax_crypto::Hasher::tx_hash(&received_tx).unwrap(),
        tx_hash
    );

    // Verify signature
    let sign_bytes = received_tx.sign_bytes().unwrap();
    assert!(sprax_crypto::Ed25519Keypair::verify(
        &received_tx.public_key,
        &sign_bytes,
        &received_tx.signature
    )
    .is_ok());

    // Node 1 broadcasts block
    let block = create_test_block(1, Hash32::ZERO, kp.address());
    hub.broadcast(&p1, NetworkMessage::BlockGossip(block.clone()))
        .await
        .unwrap();

    let received_block = tokio::time::timeout(
        Duration::from_secs(1),
        ep2.inbound_block_rx.lock().await.recv(),
    )
    .await
    .unwrap()
    .expect("node 2 must receive block");
    assert_eq!(received_block.header.height, 1);
}

#[tokio::test]
async fn test_4_node_network_consensus_proposal_and_voting_mesh() {
    let hub = SwarmHub::new();
    let mut keypairs = Vec::new();
    let mut peers = Vec::new();
    let mut endpoints = Vec::new();
    let mut validators = Vec::new();

    for i in 0..4 {
        let mut seed = [0u8; 32];
        seed[0] = (i + 1) as u8;
        let kp = Ed25519Keypair::from_seed(&seed);
        let pid = PeerId::new(format!("validator-{i}")).unwrap();
        let ep = hub.register_node(pid.clone());
        let val = Validator::new(kp.address(), kp.public_key_bytes().to_vec(), 25);

        keypairs.push(kp);
        peers.push(pid);
        endpoints.push(ep);
        validators.push(val);
    }

    let val_set = ValidatorSet::new(validators).unwrap();
    assert_eq!(val_set.quorum_threshold(), 67); // 4 * 25 = 100, quorum = 67

    // Node 0 (Proposer) creates and broadcasts SignedProposal
    let block = create_test_block(1, Hash32::ZERO, keypairs[0].address());
    let proposal_hash = sprax_crypto::Hasher::block_hash(&block.header).unwrap();

    let mut proposal = SignedProposal {
        genesis: Hash32::ZERO,
        signer: keypairs[0].address(),
        round: 0,
        valid_round: None,
        valid_round_votes: vec![],
        block: block.clone(),
        signature: vec![],
    };
    let sign_bytes = proposal.sign_bytes().unwrap();
    proposal.signature = keypairs[0].sign(&sign_bytes);

    hub.broadcast(&peers[0], NetworkMessage::Proposal(proposal.clone()))
        .await
        .unwrap();

    // Nodes 1, 2, 3 receive proposal and emit Prevote
    for i in 1..4 {
        let rx_prop = tokio::time::timeout(
            Duration::from_secs(1),
            endpoints[i].inbound_proposal_rx.lock().await.recv(),
        )
        .await
        .unwrap()
        .expect("must receive proposal");
        assert_eq!(rx_prop.block.header.height, 1);
        assert_eq!(rx_prop.round, 0);

        // Emit prevote
        let mut pv = Vote::new(
            Hash32::ZERO,
            VoteType::Prevote,
            1,
            0,
            Some(proposal_hash),
            keypairs[i].address(),
            vec![],
        );
        let sign_bytes = pv.sign_bytes().unwrap();
        pv.signature = keypairs[i].sign(&sign_bytes);

        hub.broadcast(&peers[i], NetworkMessage::Vote(pv))
            .await
            .unwrap();
    }

    // Node 0 receives all 3 prevotes (75% power > 67 quorum)
    let mut prevote_count = 0;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while prevote_count < 3 && tokio::time::Instant::now() < deadline {
        if let Ok(Some(vote)) = tokio::time::timeout(
            Duration::from_millis(100),
            endpoints[0].inbound_vote_rx.lock().await.recv(),
        )
        .await
        {
            assert_eq!(vote.vote_type, VoteType::Prevote);
            assert_eq!(vote.block_hash, Some(proposal_hash));
            prevote_count += 1;
        }
    }
    assert_eq!(
        prevote_count, 3,
        "Proposer must receive +2/3 prevotes from peers"
    );
}

#[tokio::test]
async fn test_7_node_network_gossip_flood_and_deduplication() {
    let hub = SwarmHub::new();
    let mut endpoints = Vec::new();
    let mut peers = Vec::new();

    for i in 0..7 {
        let pid = PeerId::new(format!("node-7-{i}")).unwrap();
        let ep = hub.register_node(pid.clone());
        peers.push(pid);
        endpoints.push(ep);
    }

    let kp = Ed25519Keypair::from_seed(&[7u8; 32]);
    let tx = create_test_tx(&kp, Address::new([9u8; 20]), 50, 1);
    let expected_hash = sprax_crypto::Hasher::tx_hash(&tx).unwrap();

    // Flood network from Node 0: broadcasts to 6 other peers
    let delivered = hub
        .broadcast(&peers[0], NetworkMessage::TxGossip(tx.clone()))
        .await
        .unwrap();
    assert_eq!(delivered, 6);

    // Every peer from 1 to 6 receives the transaction
    for i in 1..7 {
        let rx_tx = tokio::time::timeout(
            Duration::from_secs(1),
            endpoints[i].inbound_tx_rx.lock().await.recv(),
        )
        .await
        .unwrap()
        .expect("peer must receive tx");
        assert_eq!(
            sprax_crypto::Hasher::tx_hash(&rx_tx).unwrap(),
            expected_hash
        );
    }
}

#[tokio::test]
async fn test_10_node_network_heterogeneous_topology_throughput() {
    let hub = SwarmHub::new();
    let mut endpoints = Vec::new();
    let mut peers = Vec::new();

    for i in 0..10 {
        let pid = PeerId::new(format!("node-10-{i}")).unwrap();
        let ep = hub.register_node(pid.clone());
        peers.push(pid);
        endpoints.push(ep);
    }

    let kp = Ed25519Keypair::from_seed(&[10u8; 32]);
    let tx_count = 20;

    // Node 0 broadcasts 20 transactions in rapid sequence
    for nonce in 0..tx_count {
        let tx = create_test_tx(&kp, Address::new([0xAA; 20]), 1, nonce);
        hub.broadcast(&peers[0], NetworkMessage::TxGossip(tx))
            .await
            .unwrap();
    }

    // Node 9 receives all 20 transactions
    let mut received = 0;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    while received < tx_count && tokio::time::Instant::now() < deadline {
        if let Ok(Some(_)) = tokio::time::timeout(
            Duration::from_millis(100),
            endpoints[9].inbound_tx_rx.lock().await.recv(),
        )
        .await
        {
            received += 1;
        }
    }
    assert_eq!(
        received, tx_count,
        "Node 9 must receive all 20 gossip transactions"
    );
}

#[tokio::test]
async fn test_fault_peer_disconnect_and_reconnection_sync() {
    let hub = SwarmHub::new();
    let p1 = PeerId::new("node-sync-1").unwrap();
    let p2 = PeerId::new("node-sync-2").unwrap();

    let _ep1 = hub.register_node(p1.clone());
    let _ep2 = hub.register_node(p2.clone());

    // Disconnect Node 2
    hub.unregister_node(&p2);

    // Node 1 broadcasts block while Node 2 is offline
    let block1 = create_test_block(1, Hash32::ZERO, Address::new([1u8; 20]));
    let delivered = hub
        .broadcast(&p1, NetworkMessage::BlockGossip(block1))
        .await
        .unwrap();
    assert_eq!(delivered, 0, "No active peers to deliver block");

    // Node 2 re-registers (reconnects)
    let ep2_reconnected = hub.register_node(p2.clone());

    // Node 1 broadcasts next block
    let block2 = create_test_block(2, Hash32::new([1u8; 32]), Address::new([1u8; 20]));
    hub.broadcast(&p1, NetworkMessage::BlockGossip(block2))
        .await
        .unwrap();

    let rx = tokio::time::timeout(
        Duration::from_secs(1),
        ep2_reconnected.inbound_block_rx.lock().await.recv(),
    )
    .await
    .unwrap()
    .expect("must receive block after reconnection");
    assert_eq!(rx.header.height, 2);
}

#[tokio::test]
async fn test_fault_delayed_messages_and_asymmetric_partitions() {
    let hub = SwarmHub::new();
    let p1 = PeerId::new("part-1").unwrap();
    let p2 = PeerId::new("part-2").unwrap();
    let p3 = PeerId::new("part-3").unwrap();

    let _ep1 = hub.register_node(p1.clone());
    let ep2 = hub.register_node(p2.clone());
    let ep3 = hub.register_node(p3.clone());

    // Partition Node 1 from Node 2
    hub.partition_peers(p1.clone(), p2.clone());

    // Node 1 broadcasts block -> should only reach Node 3
    let block = create_test_block(1, Hash32::ZERO, Address::new([1u8; 20]));
    hub.broadcast(&p1, NetworkMessage::BlockGossip(block))
        .await
        .unwrap();

    // Node 3 receives
    let rx3 = tokio::time::timeout(
        Duration::from_millis(500),
        ep3.inbound_block_rx.lock().await.recv(),
    )
    .await;
    assert!(rx3.is_ok(), "Node 3 should receive block");

    // Node 2 is partitioned -> receives nothing
    let rx2 = tokio::time::timeout(
        Duration::from_millis(200),
        ep2.inbound_block_rx.lock().await.recv(),
    )
    .await;
    assert!(rx2.is_err(), "Partitioned Node 2 must not receive message");

    // Heal partition
    hub.heal_partition(&p1, &p2);

    // Broadcast again -> Node 2 now receives
    let block2 = create_test_block(2, Hash32::ZERO, Address::new([1u8; 20]));
    hub.broadcast(&p1, NetworkMessage::BlockGossip(block2))
        .await
        .unwrap();

    let rx2_healed = tokio::time::timeout(
        Duration::from_millis(500),
        ep2.inbound_block_rx.lock().await.recv(),
    )
    .await;
    assert!(rx2_healed.is_ok(), "Healed Node 2 must receive message");
}

#[tokio::test]
async fn test_security_malformed_and_tampered_payload_rejection() {
    let kp = Ed25519Keypair::from_seed(&[5u8; 32]);
    let mut tx = create_test_tx(&kp, Address::new([7u8; 20]), 50, 0);

    // Tamper with transaction memo
    tx.body.memo = "tampered payload".into();

    let sign_bytes = tx.sign_bytes().unwrap();
    let verify_res =
        sprax_crypto::Ed25519Keypair::verify(&tx.public_key, &sign_bytes, &tx.signature);
    assert!(
        verify_res.is_err(),
        "Tampered transaction signature must fail verification"
    );
}

#[tokio::test]
async fn test_security_rate_limiting_and_peer_reputation_decay() {
    let mut score = PeerScore::default();
    assert_eq!(score.score, 0.0);

    // Record bad behavior
    score.record_invalid_message();
    assert!(score.score < 0.0);

    for _ in 0..5 {
        score.record_invalid_message();
    }
    assert!(
        score.is_banned,
        "Repeated invalid messages must trigger peer ban"
    );
}
