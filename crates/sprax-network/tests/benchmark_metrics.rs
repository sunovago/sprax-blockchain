use sprax_crypto::Ed25519Keypair;
use sprax_network::{
    message::NetworkMessage, transport::AuthenticatedChannel, NetworkIdentity, SwarmHub,
};
use sprax_types::{Amount, ChainId, Hash32, KeyType, Transaction, TxBody, TxFee, TxMessage};
use std::{sync::Arc, time::Instant};

#[tokio::test]
async fn benchmark_network_performance_metrics() {
    println!("\n=== SPRX PHASE 3 P2P PERFORMANCE BENCHMARK ===");

    // 1. Message Serialization / Deserialization Latency
    let alice = Ed25519Keypair::from_seed(&[1u8; 32]);
    let bob = Ed25519Keypair::from_seed(&[2u8; 32]);
    let tx_body = TxBody {
        chain_id: ChainId::new("sprax-devnet-1").unwrap(),
        sender: alice.address(),
        nonce: 1,
        messages: vec![TxMessage::Transfer {
            to: bob.address(),
            amount: Amount::from_sprx_whole(50).unwrap(),
        }],
        fee: TxFee::default(),
        memo: "perf test tx".to_string(),
        timeout_height: 100,
    };
    let sign_bytes = tx_body.sign_bytes().unwrap();
    let sig = alice.sign(&sign_bytes);
    let tx = Transaction::new(
        tx_body,
        KeyType::Ed25519,
        alice.public_key_bytes().to_vec(),
        sig,
    )
    .unwrap();
    let msg = NetworkMessage::TxGossip(tx);

    let iters = 2_000;
    let start_ser = Instant::now();
    let mut serialized_bytes = Vec::new();
    for _ in 0..iters {
        serialized_bytes = msg.encode().unwrap();
    }
    let ser_duration = start_ser.elapsed();
    let avg_ser_us = (ser_duration.as_micros() as f64) / (iters as f64);

    let raw_payload = &serialized_bytes[4..];
    let start_de = Instant::now();
    for _ in 0..iters {
        let _decoded = NetworkMessage::decode(raw_payload).unwrap();
    }
    let de_duration = start_de.elapsed();
    let avg_de_us = (de_duration.as_micros() as f64) / (iters as f64);

    println!(
        "Framed Message size: {} bytes (Payload: {} bytes)",
        serialized_bytes.len(),
        raw_payload.len()
    );
    println!("Avg Serialization Latency: {:.3} µs", avg_ser_us);
    println!("Avg Deserialization Latency: {:.3} µs", avg_de_us);

    // 2. Noise Handshake Latency
    let id_a = Arc::new(NetworkIdentity::generate().unwrap());
    let id_b = Arc::new(NetworkIdentity::generate().unwrap());
    let genesis_hash = Hash32::new([42u8; 32]);

    let handshake_iters = 200;
    let start_handshake = Instant::now();
    for _ in 0..handshake_iters {
        let (mut stream_a, mut stream_b) = tokio::io::duplex(65_536);
        let id_a_c = Arc::clone(&id_a);
        let id_b_c = Arc::clone(&id_b);

        let t_a = tokio::spawn(async move {
            AuthenticatedChannel::initiate(&mut stream_a, &id_a_c, "sprax-devnet-1", &genesis_hash)
                .await
                .unwrap()
        });

        let t_b = tokio::spawn(async move {
            AuthenticatedChannel::respond(&mut stream_b, &id_b_c, "sprax-devnet-1", &genesis_hash)
                .await
                .unwrap()
        });

        let (res_a, res_b) = tokio::join!(t_a, t_b);
        let _ = res_a.unwrap();
        let _ = res_b.unwrap();
    }
    let handshake_duration = start_handshake.elapsed();
    let avg_handshake_ms = (handshake_duration.as_secs_f64() * 1000.0) / (handshake_iters as f64);
    println!(
        "Avg Mutual Noise Handshake Latency: {:.3} ms",
        avg_handshake_ms
    );

    // 3. GossipSub Propagation Throughput in SwarmHub (10 nodes)
    let hub = Arc::new(SwarmHub::new());
    let num_nodes = 10;
    let mut identities = Vec::new();
    let mut rx_channels = Vec::new();

    for _ in 0..num_nodes {
        let id = NetworkIdentity::generate().unwrap();
        let rx = hub.register_node(id.peer_id().clone());
        identities.push(id);
        rx_channels.push(rx);
    }

    let tx_count = 1_000;
    let start_flood = Instant::now();
    let sender_peer = identities[0].peer_id();
    for _ in 0..tx_count {
        let _ = hub.broadcast(sender_peer, msg.clone()).await;
    }
    let flood_duration = start_flood.elapsed();
    let total_deliveries = tx_count * (num_nodes - 1);
    let throughput = (total_deliveries as f64) / flood_duration.as_secs_f64();
    println!(
        "Gossip Flood (10 nodes mesh): {} total deliveries in {:.3} s",
        total_deliveries,
        flood_duration.as_secs_f64()
    );
    println!("Gossip Throughput: {:.1} msgs/sec", throughput);

    // 4. Memory footprint per peer connection estimate
    let per_peer_channel_buffers = (128 * 1024) + (64 * 1024); // RX/TX framing buffers + queues
    println!(
        "Estimated memory footprint per active peer session: ~{} KB",
        per_peer_channel_buffers / 1024
    );
    println!("=== BENCHMARK COMPLETE ===\n");
}
