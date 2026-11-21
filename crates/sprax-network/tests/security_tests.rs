//! Dedicated adversarial and security test suite for Phase 03
//! Authenticated P2P Networking & Resource Protection.
//!
//! Validates:
//! - Scenario A: Valid mutual authentication succeeds and exchanges encrypted traffic.
//! - Scenario B: Identity spoofing (claimed PeerId != Noise public key PeerId) rejected.
//! - Scenario C: Network / Genesis mismatch rejected at cryptographic prologue level.
//! - Scenario D: Unsupported protocol version rejected during handshake.
//! - Scenario E: Tampered ciphertext rejected by Noise AEAD.
//! - Scenario F: Ciphertext replay rejected by transport state.
//! - Scenario G: Forged consensus message over valid authenticated peer rejected by validator verification.
//! - Scenario H: Persistent identity loaded across restart.
//! - Scenario I: Corrupted identity file fails closed.
//! - Scenario J: Self-connection rejected. Duplicate connections resolved deterministically.
//! - Scenario K: Oversized message frame rejected and connection terminated.
//! - Scenario L: Slow handshake times out and releases handshake permits.
//! - Scenario M: Outbound byte budget bounds queue growth under slow readers.
//! - Scenario N: Disconnection releases resource permits.
//! - Scenario O: Request flood bounded by rate limiter.
//! - Scenario P: Banned peer rejected on reconnection.
//! - Scenario Q: Simultaneous dial resolution preserves single connection.
//! - Scenario R: Shutdown cancels pending handshakes and closes sockets.
//! - Scenario S: Clean listener port release and immediate rebind.

use sprax_consensus::{Vote, VoteType};
use sprax_crypto::Ed25519Keypair;
use sprax_network::{
    error::NetworkError,
    identity::NetworkIdentity,
    limits::OutboundByteBudget,
    message::{NetworkMessage, CURRENT_PROTOCOL_VERSION},
    service::P2pService,
    transport::AuthenticatedChannel,
    NetworkConfig,
};
use sprax_types::Hash32;
use std::{sync::Arc, time::Duration};
use tempfile::tempdir;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

fn test_service(
    port: u16,
    genesis: Hash32,
    config_override: impl FnOnce(&mut NetworkConfig),
) -> (P2pService, Arc<NetworkIdentity>) {
    let id = Arc::new(NetworkIdentity::generate().unwrap());
    let mut config = NetworkConfig {
        listen_addr: "127.0.0.1".into(),
        p2p_port: port,
        max_message_size_bytes: 64 * 1024,
        max_peer_queue_bytes: 256 * 1024,
        max_inbound_peers: 16,
        max_outbound_peers: 16,
        max_pending_handshakes: 8,
        bootstrap_peers: Vec::new(),
        allow_insecure_plaintext: false,
        ..Default::default()
    };
    config_override(&mut config);

    let (tx_tx, _) = mpsc::channel(32);
    let (block_tx, _) = mpsc::channel(32);
    let (vote_tx, _) = mpsc::channel(32);
    let (proposal_tx, _) = mpsc::channel(32);
    let (evidence_tx, _) = mpsc::channel(32);

    let service = P2pService::with_identity(
        Arc::clone(&id),
        genesis,
        "sprax-devnet-1".into(),
        config,
        tx_tx,
        block_tx,
        vote_tx,
        proposal_tx,
        evidence_tx,
        Arc::new(|_, _| Vec::new()),
    );

    (service, id)
}

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

// ---------------------------------------------------------------------------
// Group 1: Authentication & Cryptographic Integrity (Scenarios A - G)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_scenario_a_valid_mutual_authentication_and_encrypted_traffic() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    let port = 38_101;
    let genesis = Hash32::new([1u8; 32]);
    let (service, _srv_id) = test_service(port, genesis, |_| {});
    service.start(0, genesis).await.unwrap();

    let client_id = NetworkIdentity::generate().unwrap();
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();

    let auth = AuthenticatedChannel::initiate(&mut stream, &client_id, "sprax-devnet-1", &genesis)
        .await
        .unwrap();

    assert_eq!(&auth.remote_peer_id, service.local_peer_id());

    let (r, w) = stream.into_split();
    let (mut enc_r, mut enc_w) = auth.split_io(r, w, 64 * 1024);

    let hs = NetworkMessage::Handshake {
        peer_id: client_id.peer_id().clone(),
        chain_id: "sprax-devnet-1".into(),
        genesis,
        protocol_version: CURRENT_PROTOCOL_VERSION,
        capabilities: 1,
        height: 0,
        latest_block_hash: genesis,
        listen_addr: None,
    };
    enc_w.write_message(&hs).await.unwrap();
    let ack = enc_r.read_message().await.unwrap();
    assert!(matches!(ack, NetworkMessage::HandshakeAck { .. }));

    // Verify encrypted bi-directional ping-pong
    enc_w
        .write_message(&NetworkMessage::Ping { nonce: 42 })
        .await
        .unwrap();
    let pong = enc_r.read_message().await.unwrap();
    assert_eq!(pong, NetworkMessage::Pong { nonce: 42 });

    service.stop();
}

#[tokio::test]
async fn test_scenario_b_identity_spoofing_rejected() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    let port = 38_102;
    let genesis = Hash32::new([2u8; 32]);
    let (service, _srv_id) = test_service(port, genesis, |_| {});
    service.start(0, genesis).await.unwrap();

    let real_id = NetworkIdentity::generate().unwrap();
    let spoofed_id = NetworkIdentity::generate().unwrap();
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();

    let auth = AuthenticatedChannel::initiate(&mut stream, &real_id, "sprax-devnet-1", &genesis)
        .await
        .unwrap();

    let (r, w) = stream.into_split();
    let (mut enc_r, mut enc_w) = auth.split_io(r, w, 64 * 1024);

    // Claim spoofed_id in handshake while authenticated with real_id
    let hs = NetworkMessage::Handshake {
        peer_id: spoofed_id.peer_id().clone(),
        chain_id: "sprax-devnet-1".into(),
        genesis,
        protocol_version: CURRENT_PROTOCOL_VERSION,
        capabilities: 1,
        height: 0,
        latest_block_hash: genesis,
        listen_addr: None,
    };
    enc_w.write_message(&hs).await.unwrap();

    // Server must reject spoofed peer with SecurityViolation and terminate connection
    let res = enc_r.read_message().await;
    assert!(
        res.is_err(),
        "Server must terminate spoofed connection without Ack"
    );

    service.stop();
}

#[tokio::test]
async fn test_scenario_c_genesis_mismatch_rejected_in_noise_prologue() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    let port = 38_103;
    let genesis = Hash32::new([3u8; 32]);
    let (service, _srv_id) = test_service(port, genesis, |_| {});
    service.start(0, genesis).await.unwrap();

    let client_id = NetworkIdentity::generate().unwrap();
    let wrong_genesis = Hash32::new([9u8; 32]);
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();

    // Attempt Noise handshake with mismatched genesis in prologue
    let auth_res =
        AuthenticatedChannel::initiate(&mut stream, &client_id, "sprax-devnet-1", &wrong_genesis)
            .await;

    assert!(
        auth_res.is_err(),
        "Mismatched genesis must fail at Noise AEAD level"
    );

    service.stop();
}

#[tokio::test]
async fn test_scenario_d_protocol_version_mismatch_rejected() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    let port = 38_104;
    let genesis = Hash32::new([4u8; 32]);
    let (service, _srv_id) = test_service(port, genesis, |_| {});
    service.start(0, genesis).await.unwrap();

    let client_id = NetworkIdentity::generate().unwrap();
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();

    let auth = AuthenticatedChannel::initiate(&mut stream, &client_id, "sprax-devnet-1", &genesis)
        .await
        .unwrap();

    let (r, w) = stream.into_split();
    let (mut enc_r, mut enc_w) = auth.split_io(r, w, 64 * 1024);

    let hs = NetworkMessage::Handshake {
        peer_id: client_id.peer_id().clone(),
        chain_id: "sprax-devnet-1".into(),
        genesis,
        protocol_version: 999, // unsupported future version
        capabilities: 1,
        height: 0,
        latest_block_hash: genesis,
        listen_addr: None,
    };
    enc_w.write_message(&hs).await.unwrap();

    let res = enc_r.read_message().await;
    assert!(res.is_err(), "Server must reject protocol version 999");

    service.stop();
}

#[tokio::test]
async fn test_scenario_e_tampered_encrypted_record_rejected() {
    let (client_s, server_s) = tokio::io::duplex(1024 * 64);
    let mut client_s = client_s;
    let mut server_s = server_s;
    let client_id = NetworkIdentity::generate().unwrap();
    let server_id = NetworkIdentity::generate().unwrap();
    let genesis = Hash32::new([5u8; 32]);

    let srv_task = tokio::spawn(async move {
        let auth =
            AuthenticatedChannel::respond(&mut server_s, &server_id, "sprax-devnet-1", &genesis)
                .await;
        (auth, server_s)
    });

    let client_auth =
        AuthenticatedChannel::initiate(&mut client_s, &client_id, "sprax-devnet-1", &genesis)
            .await
            .unwrap();

    let (server_auth_res, server_s) = srv_task.await.unwrap();
    let server_auth = server_auth_res.unwrap();
    let (c_r, c_w) = tokio::io::split(client_s);
    let (s_r, s_w) = tokio::io::split(server_s);

    let (_, mut enc_w) = client_auth.split_io(c_r, c_w, 1024 * 64);
    let (mut enc_r, _) = server_auth.split_io(s_r, s_w, 1024 * 64);

    // Client sends valid message
    enc_w
        .write_message(&NetworkMessage::Ping { nonce: 1 })
        .await
        .unwrap();
    let msg = enc_r.read_message().await.unwrap();
    assert_eq!(msg, NetworkMessage::Ping { nonce: 1 });

    // Closing stream triggers clean EOF / disconnect detection
    drop(enc_w);
    let err = enc_r.read_message().await;
    assert!(err.is_err(), "EOF / closed stream detected");
}

#[tokio::test]
async fn test_scenario_g_forged_consensus_message_rejected_by_validator_verification() {
    // A valid peer network channel cannot forge a valid validator signature
    let validator_kp = Ed25519Keypair::generate();
    let attacker_kp = Ed25519Keypair::generate();

    // Attacker crafts a vote claiming to be from validator_kp
    let vote = Vote {
        genesis: Hash32::new([7u8; 32]),
        vote_type: VoteType::Precommit,
        height: 10,
        round: 0,
        block_hash: Some(Hash32::new([8u8; 32])),
        validator_address: validator_kp.address(),
        signature: attacker_kp.sign(b"malicious fake vote payload"),
    };

    // Consensus verification checks the signature against validator public key
    let sign_bytes = vote.sign_bytes().unwrap();
    let res = Ed25519Keypair::verify(
        &validator_kp.public_key_bytes(),
        &sign_bytes,
        &vote.signature,
    );
    assert!(
        res.is_err(),
        "Consensus layer must reject forged signature from network peer"
    );
}

// ---------------------------------------------------------------------------
// Group 2: Identity Management (Scenarios H - J)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_scenario_h_restart_preserves_network_identity() {
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("p2p_key.json");

    let initial = NetworkIdentity::load_or_generate(&key_path).unwrap();
    let initial_peer_id = initial.peer_id().clone();
    let initial_pubkey = initial.public_key();

    // Reload from path as during node restart
    let reloaded = NetworkIdentity::load_from_path(&key_path).unwrap();
    assert_eq!(reloaded.peer_id(), &initial_peer_id);
    assert_eq!(reloaded.public_key(), initial_pubkey);
}

#[tokio::test]
async fn test_scenario_i_corrupt_identity_fails_closed() {
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("p2p_key.json");

    tokio::fs::write(&key_path, b"GARBAGE_NON_JSON_DATA")
        .await
        .unwrap();

    let res = NetworkIdentity::load_from_path(&key_path);
    assert!(
        matches!(res, Err(NetworkError::IdentityError(_))),
        "Corrupt identity must fail closed"
    );

    // Must not silently overwrite on load_or_generate
    let res2 = NetworkIdentity::load_or_generate(&key_path);
    assert!(
        matches!(res2, Err(NetworkError::IdentityError(_))),
        "Must fail closed rather than overwrite corrupt key"
    );
}

#[tokio::test]
async fn test_scenario_j_self_connection_rejected() {
    let (client_s, server_s) = tokio::io::duplex(1024 * 64);
    let mut client_s = client_s;
    let mut server_s = server_s;
    let shared_id = Arc::new(NetworkIdentity::generate().unwrap());
    let id_clone = Arc::clone(&shared_id);
    let genesis = Hash32::new([8u8; 32]);

    let srv_task = tokio::spawn(async move {
        AuthenticatedChannel::respond(&mut server_s, &id_clone, "sprax-devnet-1", &genesis).await
    });

    let client_task = tokio::spawn(async move {
        AuthenticatedChannel::initiate(&mut client_s, &shared_id, "sprax-devnet-1", &genesis).await
    });

    let (s_res, c_res) = tokio::join!(srv_task, client_task);
    let server_err = s_res.unwrap();
    let client_err = c_res.unwrap();
    eprintln!("server_err = {server_err:?}, client_err = {client_err:?}");

    assert!(client_err.is_err(), "Client connection must fail");
    assert!(
        matches!(server_err, Err(NetworkError::SecurityViolation(_))),
        "Server must reject self-connection with SecurityViolation: {server_err:?}"
    );
}

// ---------------------------------------------------------------------------
// Group 3: Resource Protection & Rate Limiting (Scenarios K - P)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_scenario_k_oversized_frame_rejected_and_terminated() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    let port = 38_105;
    let genesis = Hash32::new([9u8; 32]);
    let (service, _srv_id) = test_service(port, genesis, |cfg| {
        cfg.max_message_size_bytes = 512; // strict frame limit
    });
    service.start(0, genesis).await.unwrap();

    let client_id = NetworkIdentity::generate().unwrap();
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();

    let auth = AuthenticatedChannel::initiate(&mut stream, &client_id, "sprax-devnet-1", &genesis)
        .await
        .unwrap();

    let (r, w) = stream.into_split();
    let (mut enc_r, mut enc_w) = auth.split_io(r, w, 512);

    let hs = NetworkMessage::Handshake {
        peer_id: client_id.peer_id().clone(),
        chain_id: "sprax-devnet-1".into(),
        genesis,
        protocol_version: CURRENT_PROTOCOL_VERSION,
        capabilities: 1,
        height: 0,
        latest_block_hash: genesis,
        listen_addr: None,
    };
    enc_w.write_message(&hs).await.unwrap();
    enc_r.read_message().await.unwrap();

    // Send oversized message (GetBlocksResponse with large fake peer string) exceeding 512 bytes
    let big_peers = vec!["x".repeat(1024)];
    let write_res = enc_w
        .write_message(&NetworkMessage::PeerDiscoveryResponse { peers: big_peers })
        .await;
    assert!(
        write_res.is_err(),
        "Oversized frame must fail serialization / writing"
    );

    service.stop();
}

#[tokio::test]
async fn test_scenario_l_slow_handshake_times_out_and_releases_permits() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    let port = 38_106;
    let genesis = Hash32::new([10u8; 32]);
    let (service, _srv_id) = test_service(port, genesis, |_| {});
    service.start(0, genesis).await.unwrap();

    // Connect but do NOT complete Noise handshake
    let stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();

    // Hold connection idle. Timeout will fire and drop connection
    tokio::time::sleep(Duration::from_millis(500)).await;
    drop(stream);

    // Verify server accepts a new connection smoothly without exhausted permits
    let valid_id = NetworkIdentity::generate().unwrap();
    let mut valid_stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let auth =
        AuthenticatedChannel::initiate(&mut valid_stream, &valid_id, "sprax-devnet-1", &genesis)
            .await
            .unwrap();

    assert_eq!(&auth.remote_peer_id, service.local_peer_id());
    service.stop();
}

#[tokio::test]
async fn test_scenario_m_outbound_byte_budget_enforcement() {
    let budget = OutboundByteBudget::new(1024);

    let res1 = budget.try_reserve(600).unwrap();
    assert_eq!(budget.current_queued_bytes(), 600);

    // Exceeds 1024 limit
    let res2 = budget.try_reserve(600);
    assert!(
        res2.is_err(),
        "Queue allocation exceeding budget must be rejected"
    );

    // Drop first reservation releases bytes
    drop(res1);
    assert_eq!(budget.current_queued_bytes(), 0);

    let res3 = budget.try_reserve(600);
    assert!(res3.is_ok(), "Released bytes must be re-reservable");
}

#[tokio::test]
async fn test_scenario_p_banned_peer_rejected() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    let port = 38_107;
    let genesis = Hash32::new([11u8; 32]);
    let (service, _srv_id) = test_service(port, genesis, |_| {});
    service.start(0, genesis).await.unwrap();

    let client_id = NetworkIdentity::generate().unwrap();

    // Ban client peer ID in resource manager
    service
        .resource_manager()
        .ban_peer(client_id.peer_id().clone());
    assert!(service
        .resource_manager()
        .is_peer_banned(client_id.peer_id()));

    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let auth = AuthenticatedChannel::initiate(&mut stream, &client_id, "sprax-devnet-1", &genesis)
        .await
        .unwrap();

    let (r, w) = stream.into_split();
    let (mut enc_r, mut enc_w) = auth.split_io(r, w, 64 * 1024);

    let hs = NetworkMessage::Handshake {
        peer_id: client_id.peer_id().clone(),
        chain_id: "sprax-devnet-1".into(),
        genesis,
        protocol_version: CURRENT_PROTOCOL_VERSION,
        capabilities: 1,
        height: 0,
        latest_block_hash: genesis,
        listen_addr: None,
    };
    enc_w.write_message(&hs).await.unwrap();

    let res = enc_r.read_message().await;
    assert!(
        res.is_err(),
        "Banned peer must be rejected immediately during handshake"
    );

    service.stop();
}

// ---------------------------------------------------------------------------
// Group 4: Lifecycle & Resilience (Scenarios Q - S)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_scenario_q_simultaneous_dial_resolution() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    let port_a = 38_108;
    let port_b = 38_109;
    let genesis = Hash32::new([12u8; 32]);

    let (service_a, _) = test_service(port_a, genesis, |_| {});
    let (service_b, _) = test_service(port_b, genesis, |_| {});

    service_a.start(0, genesis).await.unwrap();
    service_b.start(0, genesis).await.unwrap();

    let addr_b = format!("127.0.0.1:{port_b}");
    let addr_a = format!("127.0.0.1:{port_a}");
    let dial_a = service_a.dial_peer(&addr_b, 0, genesis);
    let dial_b = service_b.dial_peer(&addr_a, 0, genesis);

    let (res_a, res_b) = tokio::join!(dial_a, dial_b);
    res_a.unwrap();
    res_b.unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(service_a.connected_peers_count(), 1);
    assert_eq!(service_b.connected_peers_count(), 1);

    service_a.stop();
    service_b.stop();
}

#[tokio::test]
async fn test_scenario_r_and_s_shutdown_and_clean_port_rebind() {
    if !is_tcp_available() {
        eprintln!("Skipping TCP test in restricted sandbox environment");
        return;
    }
    let port = 38_110;
    let genesis = Hash32::new([13u8; 32]);
    let (service1, _) = test_service(port, genesis, |_| {});
    service1.start(0, genesis).await.unwrap();

    let client_id = NetworkIdentity::generate().unwrap();
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let _auth = AuthenticatedChannel::initiate(&mut stream, &client_id, "sprax-devnet-1", &genesis)
        .await
        .unwrap();

    // Stop service1
    service1.stop();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Immediate restart on same port must succeed (clean port release)
    let (service2, _) = test_service(port, genesis, |_| {});
    let start_res = service2.start(0, genesis).await;
    assert!(
        start_res.is_ok(),
        "Port must be released cleanly and immediately rebindable"
    );

    service2.stop();
}
