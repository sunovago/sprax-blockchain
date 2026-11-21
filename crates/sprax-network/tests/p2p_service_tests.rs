//! Tests for `P2pService`'s real TCP transport (as opposed to `SwarmHub`'s in-memory
//! simulator, exercised by `network_tests.rs`) — specifically the catch-up-on-connect
//! `GetBlocksRequest`/`GetBlocksResponse` round trip added when consensus wiring was added.

use sprax_network::{BlockFetchFn, NetworkConfig, P2pService, PeerId};
use sprax_types::{Address, Block, BlockBody, BlockHeader, Hash32};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc;

fn is_tcp_available() -> bool {
    let Ok(listener) = std::net::TcpListener::bind("127.0.0.1:0") else {
        return false;
    };
    let Ok(addr) = listener.local_addr() else {
        return false;
    };
    match std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(50)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn synthetic_block(height: u64) -> Block {
    Block {
        header: BlockHeader {
            version: 1,
            chain_id: "sprax-devnet-1".to_string(),
            height,
            timestamp_unix_secs: 1_700_000_000 + height,
            parent_hash: Hash32::new([height as u8; 32]),
            proposer: Address::new([0xAB; 20]),
            state_root: Hash32::new([height as u8; 32]),
            txs_root: Hash32::ZERO,
            receipts_root: Hash32::ZERO,
            validator_set_hash: Hash32::ZERO,
        },
        body: BlockBody::default(),
        last_commit: vec![],
    }
}

fn build_service(
    port: u16,
    inbound_block_tx: mpsc::Sender<Block>,
    block_fetch_fn: BlockFetchFn,
) -> P2pService {
    build_service_with_config(
        NetworkConfig {
            p2p_port: port,
            ..Default::default()
        },
        inbound_block_tx,
        block_fetch_fn,
    )
}

fn build_service_with_config(
    config: NetworkConfig,
    inbound_block_tx: mpsc::Sender<Block>,
    block_fetch_fn: BlockFetchFn,
) -> P2pService {
    let port = config.p2p_port;
    let peer_id = PeerId::new(format!("node-{port}")).unwrap();
    let (tx_tx, _tx_rx) = mpsc::channel(128);
    let (vote_tx, _vote_rx) = mpsc::channel(128);
    let (proposal_tx, _proposal_rx) = mpsc::channel(128);
    let (evidence_tx, _evidence_rx) = mpsc::channel(128);
    P2pService::new(
        peer_id,
        "sprax-devnet-1".to_string(),
        config,
        tx_tx,
        inbound_block_tx,
        vote_tx,
        proposal_tx,
        evidence_tx,
        block_fetch_fn,
    )
}

#[tokio::test]
async fn reconnect_handshake_uses_the_latest_finalized_tip() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = reserved.local_addr().unwrap().port();
    let (server_blocks, _) = mpsc::channel(8);
    let server = build_service(
        port,
        server_blocks,
        Arc::new(|from, to| (from..=to.min(3)).map(synthetic_block).collect()),
    );
    drop(reserved);
    server.start(0, Hash32::ZERO).await.unwrap();
    let latest_hash = sprax_crypto::Hasher::block_hash(&synthetic_block(3).header).unwrap();
    server.update_finalized_tip(3, latest_hash);
    server.update_finalized_tip(1, Hash32::ZERO);
    let (blocks, mut incoming) = mpsc::channel(8);
    let client = build_service(0, blocks, Arc::new(|_, _| Vec::new()));
    client
        .dial_peer(&format!("127.0.0.1:{port}"), 0, Hash32::ZERO)
        .await
        .unwrap();
    for height in 1..=3 {
        let block = tokio::time::timeout(Duration::from_secs(3), incoming.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(block.header.height, height);
    }
    let peer = client.connected_peers().pop().unwrap();
    assert_eq!(peer.height, 3);
    assert_eq!(peer.latest_block_hash, latest_hash);
    client.stop();
    server.stop();
}

#[tokio::test]
async fn oversized_handshakes_are_closed_and_pending_connections_count_towards_the_limit() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let port = 37_950;
    let (blocks, _receiver) = mpsc::channel(8);
    let service = build_service_with_config(
        NetworkConfig {
            p2p_port: port,
            max_inbound_peers: 1,
            max_outbound_peers: 0,
            ..Default::default()
        },
        blocks,
        Arc::new(|_, _| Vec::new()),
    );
    service.start(0, Hash32::ZERO).await.unwrap();
    let mut first = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    first.write_all(&u32::MAX.to_be_bytes()).await.unwrap();
    let mut byte = [0u8; 1];
    let closed = tokio::time::timeout(Duration::from_secs(2), first.read(&mut byte))
        .await
        .unwrap();
    assert!(
        closed.is_err() || closed.unwrap() == 0,
        "oversized handshake must be rejected before allocation"
    );
    drop(first);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let silent = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let mut excess = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    let closed = tokio::time::timeout(Duration::from_secs(2), excess.read(&mut byte))
        .await
        .unwrap();
    assert!(
        closed.is_err() || closed.unwrap() == 0,
        "a pending handshake must occupy a connection slot"
    );
    assert!(service
        .dial_peer("127.0.0.1:1", 0, Hash32::ZERO)
        .await
        .unwrap_err()
        .to_string()
        .contains("connection limit"));
    drop(silent);
    service.stop();
}

#[tokio::test]
async fn test_get_blocks_request_response_round_trip_over_real_tcp() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    let high_port = 38_900u16;
    let low_port = 38_901u16;

    // The "ahead" node (height 5) serves blocks 1..=5 out of its in-memory store when asked.
    let (high_block_tx, mut high_block_rx) = mpsc::channel(128);
    let fetch_fn: BlockFetchFn = Arc::new(|from, to| (from..=to).map(synthetic_block).collect());
    let high = build_service(high_port, high_block_tx, fetch_fn);
    high.start(5, Hash32::new([5u8; 32])).await.unwrap();

    // The "behind" node (height 0) has nothing to serve and just observes what it catches up on.
    let (low_block_tx, mut low_block_rx) = mpsc::channel(128);
    let empty_fetch_fn: BlockFetchFn = Arc::new(|_, _| vec![]);
    let low = build_service(low_port, low_block_tx, empty_fetch_fn);
    low.start(0, Hash32::ZERO).await.unwrap();

    // Low dials High: during the handshake, Low learns High is at height 5 > its own 0, and
    // (per the catch-up-on-connect logic added in `run_connection_loop`) automatically sends
    // `GetBlocksRequest{1, 5}` before entering the steady-state read loop.
    low.dial_peer(&format!("127.0.0.1:{high_port}"), 0, Hash32::ZERO)
        .await
        .unwrap();

    // High should receive the request and reply with GetBlocksResponse{blocks: [1..=5]}, which
    // Low's connection loop feeds into its own `inbound_block` channel (reusing the existing
    // BlockGossip ingestion path).
    let mut received_heights = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while received_heights.len() < 5 && tokio::time::Instant::now() < deadline {
        if let Ok(Some(block)) =
            tokio::time::timeout(Duration::from_millis(200), low_block_rx.recv()).await
        {
            received_heights.push(block.header.height);
        }
    }
    received_heights.sort_unstable();
    assert_eq!(received_heights, vec![1, 2, 3, 4, 5]);

    // High never requested anything (it was already ahead), so its own inbound_block channel
    // should have received nothing from this exchange.
    assert!(
        tokio::time::timeout(Duration::from_millis(200), high_block_rx.recv())
            .await
            .is_err(),
        "the ahead node should not receive any blocks from this catch-up exchange"
    );

    high.stop();
    low.stop();
}

#[test]
fn bounded_encoding_checks_exact_payload_size() {
    use sprax_network::NetworkMessage;
    let msg = NetworkMessage::Ping { nonce: 42 };
    let framed = msg.encode().unwrap();
    let size = framed.len() - 4;
    assert_eq!(msg.encode_bounded(size).unwrap(), framed);
    assert!(msg.encode_bounded(size - 1).is_err());
    assert!(msg.encode_bounded(0).is_err());
}

async fn raw_handshake(
    port: u16,
    identity: &sprax_network::NetworkIdentity,
) -> (
    sprax_network::EncryptedReader<tokio::net::tcp::OwnedReadHalf>,
    sprax_network::EncryptedWriter<tokio::net::tcp::OwnedWriteHalf>,
) {
    use sprax_network::{AuthenticatedChannel, NetworkMessage, CURRENT_PROTOCOL_VERSION};
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    let auth =
        AuthenticatedChannel::initiate(&mut stream, identity, "sprax-devnet-1", &Hash32::ZERO)
            .await
            .unwrap();
    let (reader, writer) = stream.into_split();
    let (mut enc_r, mut enc_w) = auth.split_io(reader, writer, 1024 * 1024);
    let handshake = NetworkMessage::Handshake {
        peer_id: identity.peer_id().clone(),
        chain_id: "sprax-devnet-1".into(),
        genesis: Hash32::ZERO,
        protocol_version: CURRENT_PROTOCOL_VERSION,
        capabilities: 1,
        height: 0,
        latest_block_hash: Hash32::ZERO,
        listen_addr: None,
    };
    enc_w.write_message(&handshake).await.unwrap();
    let ack = enc_r.read_message().await.unwrap();
    assert!(matches!(ack, NetworkMessage::HandshakeAck { .. }));
    (enc_r, enc_w)
}

#[tokio::test]
async fn configured_frame_limit_and_duplicate_peer_rejection_over_tcp() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    use sprax_network::NetworkMessage;
    let port = 37_951;
    let (blocks, _receiver) = mpsc::channel(8);
    let service = build_service_with_config(
        NetworkConfig {
            p2p_port: port,
            max_message_size_bytes: 1024,
            ..Default::default()
        },
        blocks,
        Arc::new(|_, _| Vec::new()),
    );
    service.start(0, Hash32::ZERO).await.unwrap();
    let id = sprax_network::NetworkIdentity::generate().unwrap();
    let (mut first_r, mut first_w) = raw_handshake(port, &id).await;

    // Attempt second connection with same authenticated identity
    let dup_res = async {
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port)).await?;
        let auth = sprax_network::AuthenticatedChannel::initiate(
            &mut stream,
            &id,
            "sprax-devnet-1",
            &Hash32::ZERO,
        )
        .await?;
        let (reader, writer) = stream.into_split();
        let (mut enc_r, mut enc_w) = auth.split_io(reader, writer, 1024 * 1024);
        let handshake = NetworkMessage::Handshake {
            peer_id: id.peer_id().clone(),
            chain_id: "sprax-devnet-1".into(),
            genesis: Hash32::ZERO,
            protocol_version: sprax_network::message::CURRENT_PROTOCOL_VERSION,
            capabilities: 1,
            height: 0,
            latest_block_hash: Hash32::ZERO,
            listen_addr: None,
        };
        enc_w.write_message(&handshake).await?;
        enc_r.read_message().await
    }
    .await;
    assert!(dup_res.is_err());
    assert_eq!(service.connected_peers_count(), 1);

    // The first socket remains routed and usable after rejecting the duplicate.
    first_w
        .write_message(&NetworkMessage::Ping { nonce: 7 })
        .await
        .unwrap();
    let pong = first_r.read_message().await.unwrap();
    assert_eq!(pong, NetworkMessage::Pong { nonce: 7 });

    service.stop();
}

#[tokio::test]
async fn stop_closes_pending_handshakes_active_sockets_and_releases_listener() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    use tokio::io::AsyncReadExt;
    let port = 37_952;
    let (blocks, _receiver) = mpsc::channel(8);
    let service = build_service_with_config(
        NetworkConfig {
            p2p_port: port,
            ..Default::default()
        },
        blocks,
        Arc::new(|_, _| Vec::new()),
    );
    service.start(0, Hash32::ZERO).await.unwrap();
    let id_shutdown = sprax_network::NetworkIdentity::generate().unwrap();
    let (mut active_r, _active_w) = raw_handshake(port, &id_shutdown).await;
    let mut pending = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;
    service.stop();

    assert!(active_r.read_message().await.is_err());
    let mut byte = [0; 1];
    let result = tokio::time::timeout(Duration::from_secs(2), pending.read(&mut byte))
        .await
        .unwrap();
    assert!(result.is_err() || result.unwrap() == 0);
    assert_eq!(service.connected_peers_count(), 0);

    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    let listener = loop {
        match tokio::net::TcpListener::bind(("0.0.0.0", port)).await {
            Ok(listener) => break listener,
            Err(_) => {
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "stop must release the listening port"
                );
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    };
    // A failed bind must not leave is_running=true and turn the next start into a no-op.
    assert!(service.start(0, Hash32::ZERO).await.is_err());
    drop(listener);
    service.start(0, Hash32::ZERO).await.unwrap();
    let id_restart = sprax_network::NetworkIdentity::generate().unwrap();
    let (_restarted_r, _restarted_w) = raw_handshake(port, &id_restart).await;
    service.stop();
}

#[tokio::test]
async fn simultaneous_dials_keep_one_shared_connection_and_bidirectional_delivery() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    let (a_blocks, mut a_receiver) = mpsc::channel(8);
    let (b_blocks, mut b_receiver) = mpsc::channel(8);
    let a = build_service(37_953, a_blocks, Arc::new(|_, _| Vec::new()));
    let b = build_service(37_954, b_blocks, Arc::new(|_, _| Vec::new()));
    a.start(0, Hash32::ZERO).await.unwrap();
    b.start(0, Hash32::ZERO).await.unwrap();
    let (left, right) = tokio::join!(
        a.dial_peer("127.0.0.1:37954", 0, Hash32::ZERO),
        b.dial_peer("127.0.0.1:37953", 0, Hash32::ZERO),
    );
    left.unwrap();
    right.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(a.connected_peers_count(), 1);
    assert_eq!(b.connected_peers_count(), 1);
    a.broadcast_block(synthetic_block(1));
    b.broadcast_block(synthetic_block(2));
    let received_a = tokio::time::timeout(Duration::from_secs(2), a_receiver.recv())
        .await
        .unwrap()
        .unwrap();
    let received_b = tokio::time::timeout(Duration::from_secs(2), b_receiver.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(received_a.header.height, 2);
    assert_eq!(received_b.header.height, 1);
    a.stop();
    b.stop();
}

#[tokio::test]
async fn test_dynamic_peer_discovery_and_crawler_dialing() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    let port_a = 38_101;
    let port_b = 38_102;
    let port_c = 38_103;

    let (a_blocks, _) = mpsc::channel(8);
    let (b_blocks, _) = mpsc::channel(8);
    let (c_blocks, _) = mpsc::channel(8);

    // Node A is a standalone seed node
    let a = build_service_with_config(
        NetworkConfig {
            p2p_port: port_a,
            max_outbound_peers: 10,
            ..Default::default()
        },
        a_blocks,
        Arc::new(|_, _| Vec::new()),
    );

    // Node B bootstraps to Node A
    let b = build_service_with_config(
        NetworkConfig {
            p2p_port: port_b,
            bootstrap_peers: vec![format!("127.0.0.1:{port_a}")],
            max_outbound_peers: 10,
            ..Default::default()
        },
        b_blocks,
        Arc::new(|_, _| Vec::new()),
    );

    // Node C bootstraps ONLY to Node B (does NOT know Node A initially!)
    let c = build_service_with_config(
        NetworkConfig {
            p2p_port: port_c,
            bootstrap_peers: vec![format!("127.0.0.1:{port_b}")],
            max_outbound_peers: 10,
            ..Default::default()
        },
        c_blocks,
        Arc::new(|_, _| Vec::new()),
    );

    a.start(0, Hash32::ZERO).await.unwrap();
    b.start(0, Hash32::ZERO).await.unwrap();
    c.start(0, Hash32::ZERO).await.unwrap();

    // Give crawler and peer exchange time to discover Node A and connect
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        if c.connected_peers_count() >= 2 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Node C must have learned about Node A via peer exchange with Node B,
    // and dialed Node A through its background discovery crawler.
    assert_eq!(
        c.connected_peers_count(),
        2,
        "Node C should have discovered and connected to Node A"
    );
    assert!(c.known_addresses().contains(&format!("127.0.0.1:{port_a}")));

    a.stop();
    b.stop();
    c.stop();
}
