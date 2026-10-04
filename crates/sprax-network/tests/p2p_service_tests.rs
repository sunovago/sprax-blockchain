//! Tests for `P2pService`'s real TCP transport (as opposed to `SwarmHub`'s in-memory
//! simulator, exercised by `network_tests.rs`) — specifically the catch-up-on-connect
//! `GetBlocksRequest`/`GetBlocksResponse` round trip added when consensus wiring was added.

use sprax_network::{BlockFetchFn, NetworkConfig, P2pService, PeerId};
use sprax_types::{Address, Block, BlockBody, BlockHeader, Hash32};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc;

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
async fn oversized_handshakes_are_closed_and_pending_connections_count_towards_the_limit() {
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

async fn raw_handshake(port: u16, peer: &str) -> tokio::net::TcpStream {
    use sprax_network::NetworkMessage;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    let handshake = NetworkMessage::Handshake {
        peer_id: PeerId::new(peer).unwrap(),
        chain_id: "sprax-devnet-1".into(),
        height: 0,
        latest_block_hash: Hash32::ZERO,
        listen_addr: None,
    };
    stream
        .write_all(&handshake.encode().unwrap())
        .await
        .unwrap();
    let mut length = [0; 4];
    tokio::time::timeout(Duration::from_secs(2), stream.read_exact(&mut length))
        .await
        .unwrap()
        .unwrap();
    let mut ack = vec![0; u32::from_be_bytes(length) as usize];
    stream.read_exact(&mut ack).await.unwrap();
    assert!(matches!(
        NetworkMessage::decode(&ack).unwrap(),
        NetworkMessage::HandshakeAck { .. }
    ));
    stream
}

#[tokio::test]
async fn configured_frame_limit_and_duplicate_peer_rejection_over_tcp() {
    use sprax_network::NetworkMessage;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let port = 37_951;
    let (blocks, _receiver) = mpsc::channel(8);
    let service = build_service_with_config(
        NetworkConfig {
            p2p_port: port,
            max_message_size_bytes: 128,
            ..Default::default()
        },
        blocks,
        Arc::new(|_, _| Vec::new()),
    );
    service.start(0, Hash32::ZERO).await.unwrap();
    let mut first = raw_handshake(port, "same-peer").await;
    let mut duplicate = raw_handshake(port, "same-peer").await;
    let mut byte = [0; 1];
    let result = tokio::time::timeout(Duration::from_secs(2), duplicate.read(&mut byte))
        .await
        .unwrap();
    assert!(result.is_err() || result.unwrap() == 0);
    assert_eq!(service.connected_peers_count(), 1);
    // The first socket remains routed and usable after rejecting the duplicate.
    first
        .write_all(&NetworkMessage::Ping { nonce: 7 }.encode().unwrap())
        .await
        .unwrap();
    let mut length = [0; 4];
    tokio::time::timeout(Duration::from_secs(2), first.read_exact(&mut length))
        .await
        .unwrap()
        .unwrap();
    let mut pong = vec![0; u32::from_be_bytes(length) as usize];
    first.read_exact(&mut pong).await.unwrap();
    assert_eq!(
        NetworkMessage::decode(&pong).unwrap(),
        NetworkMessage::Pong { nonce: 7 }
    );
    // Only the prefix is sent: rejection must happen before payload allocation/read.
    first.write_all(&129u32.to_be_bytes()).await.unwrap();
    let result = tokio::time::timeout(Duration::from_secs(2), first.read(&mut byte))
        .await
        .unwrap();
    assert!(result.is_err() || result.unwrap() == 0);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while service.connected_peers_count() != 0 {
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    service.stop();
}
