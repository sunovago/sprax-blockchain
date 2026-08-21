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
    inbound_block_tx: mpsc::UnboundedSender<Block>,
    block_fetch_fn: BlockFetchFn,
) -> P2pService {
    let peer_id = PeerId::new(format!("node-{port}")).unwrap();
    let (tx_tx, _tx_rx) = mpsc::unbounded_channel();
    let (vote_tx, _vote_rx) = mpsc::unbounded_channel();
    let (proposal_tx, _proposal_rx) = mpsc::unbounded_channel();
    let (evidence_tx, _evidence_rx) = mpsc::unbounded_channel();
    let mut config = NetworkConfig::default();
    config.p2p_port = port;
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
async fn test_get_blocks_request_response_round_trip_over_real_tcp() {
    let high_port = 38_900u16;
    let low_port = 38_901u16;

    // The "ahead" node (height 5) serves blocks 1..=5 out of its in-memory store when asked.
    let (high_block_tx, mut high_block_rx) = mpsc::unbounded_channel();
    let fetch_fn: BlockFetchFn = Arc::new(|from, to| (from..=to).map(synthetic_block).collect());
    let high = build_service(high_port, high_block_tx, fetch_fn);
    high.start(5, Hash32::new([5u8; 32])).await.unwrap();

    // The "behind" node (height 0) has nothing to serve and just observes what it catches up on.
    let (low_block_tx, mut low_block_rx) = mpsc::unbounded_channel();
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
