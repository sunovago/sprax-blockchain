use sprax_consensus::verify_block_commit;
use sprax_core::{GenesisAccount, GenesisConfig, GenesisValidator};
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_node::{Environment, KeyRecord, NodeConfig, NodeService};
use sprax_types::{Amount, KeyType};
use std::{collections::BTreeMap, time::Duration};

async fn wait_for_height(nodes: &[Option<NodeService>], height: u64) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(120);
    while !nodes.iter().flatten().all(|node| node.height() >= height) {
        assert!(
            tokio::time::Instant::now() < deadline,
            "nodes did not reach {height}; heights/peers: {:?}",
            nodes
                .iter()
                .flatten()
                .map(|node| (node.height(), node.connected_peers_count()))
                .collect::<Vec<_>>()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn assert_certified_checkpoint(
    nodes: &[Option<NodeService>],
    genesis: &GenesisConfig,
    height: u64,
) {
    let mut expected = None;
    for node in nodes.iter().flatten() {
        let block = node.get_block_by_height(height).unwrap();
        verify_block_commit(
            &block,
            &node.canonical_validator_set().unwrap(),
            genesis.fingerprint().unwrap(),
        )
        .unwrap();
        assert!(
            block.last_commit.len() >= 3,
            "equal-power quorum needs three of four signatures"
        );
        let hash = Hasher::block_hash(&block.header).unwrap();
        assert_eq!(
            *expected.get_or_insert(hash),
            hash,
            "finalized histories diverged"
        );
    }
}

#[tokio::test]
async fn four_active_validators_recover_from_restart_and_quorum_outage() {
    tokio::time::timeout(Duration::from_secs(240), run_restart_scenario())
        .await
        .expect("recovery scenario exceeded hard deadline");
}

async fn run_restart_scenario() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_test_writer()
        .try_init();
    let homes: Vec<_> = (0..4).map(|_| tempfile::tempdir().unwrap()).collect();
    let keys: Vec<_> = (71..=74)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();
    let reservations: Vec<_> = (0..4)
        .map(|_| std::net::TcpListener::bind("127.0.0.1:0").unwrap())
        .collect();
    let ports: Vec<_> = reservations
        .iter()
        .map(|listener| listener.local_addr().unwrap().port())
        .collect();
    let mut genesis = GenesisConfig::default_development();
    genesis.accounts = keys
        .iter()
        .enumerate()
        .map(|(i, key)| GenesisAccount {
            name: format!("validator-{i}"),
            address: key.address(),
            initial_balance: Amount::from_sprx_whole(1_000).unwrap(),
        })
        .collect();
    genesis.validators = keys
        .iter()
        .enumerate()
        .map(|(i, key)| GenesisValidator {
            operator_address: key.address(),
            consensus_pubkey: key.public_key_bytes().to_vec(),
            self_stake: Amount::from_sprx_whole(100).unwrap(),
            moniker: format!("validator-{i}"),
        })
        .collect();
    for (i, home) in homes.iter().enumerate() {
        let key_dir = home.path().join("keyring");
        std::fs::create_dir_all(&key_dir).unwrap();
        let record = KeyRecord {
            name: "validator".into(),
            address: keys[i].address(),
            key_type: KeyType::Ed25519,
            mnemonic: None,
            secret_seed_hex: hex::encode([71 + i as u8; 32]),
        };
        std::fs::write(
            key_dir.join("keys.json"),
            serde_json::to_vec(&BTreeMap::from([("validator", record)])).unwrap(),
        )
        .unwrap();
        genesis
            .save_to_file(&home.path().join("genesis.json"))
            .unwrap();
        let mut config =
            NodeConfig::for_environment(Environment::Development, home.path().to_path_buf());
        config.network.listen_addr = "127.0.0.1".into();
        config.network.p2p_port = ports[i];
        config.network.bootstrap_peers = ports
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != i)
            .map(|(_, port)| format!("127.0.0.1:{port}"))
            .collect();
        config.rpc.json_rpc_port = 0;
        config.consensus.enabled = true;
        config.consensus.local_validator_key_name = Some("validator".into());
        config
            .save_to_file(&home.path().join("config.toml"))
            .unwrap();
    }
    drop(reservations);
    let mut nodes = Vec::new();
    for home in &homes {
        let node = NodeService::new_or_load(home.path().to_path_buf()).unwrap();
        node.start().await.unwrap();
        nodes.push(Some(node));
    }
    eprintln!("phase: initial four-validator quorum");
    wait_for_height(&nodes, 2).await;
    assert_certified_checkpoint(&nodes, &genesis, 2);
    let stopped = nodes[3].take().unwrap();
    stopped.stop().await.unwrap();
    let stopped_height = stopped.height();
    drop(stopped);
    let signing_path = homes[3].path().join("data/validator-signing.redb");
    let before = {
        let journal = sprax_node::signing_journal::SigningJournal::open(
            &signing_path,
            genesis.fingerprint().unwrap(),
            &keys[3],
        )
        .unwrap();
        journal
            .state()
            .unwrap()
            .expect("fourth validator must have signed before shutdown")
    };
    let checkpoint = stopped_height + 2;
    eprintln!("phase: progress with one validator offline");
    wait_for_height(&nodes, checkpoint).await;
    assert_certified_checkpoint(&nodes, &genesis, checkpoint);
    let restarted = NodeService::new_or_load(homes[3].path().to_path_buf()).unwrap();
    assert_eq!(restarted.height(), stopped_height);
    restarted.start().await.unwrap();
    nodes[3] = Some(restarted);
    eprintln!("phase: restarted validator catch-up");
    wait_for_height(&nodes, checkpoint + 1).await;
    assert_certified_checkpoint(&nodes, &genesis, checkpoint + 1);

    // Two remaining equal-power validators cannot form a three-signature quorum.
    // This is a quorum outage, not a simulation of two live network partitions.
    for slot in nodes.iter_mut().skip(2) {
        let node = slot.take().unwrap();
        node.stop().await.unwrap();
    }
    tokio::time::sleep(Duration::from_secs(2)).await;
    let halted: Vec<_> = nodes.iter().flatten().map(NodeService::height).collect();
    tokio::time::sleep(Duration::from_secs(6)).await;
    assert_eq!(
        nodes
            .iter()
            .flatten()
            .map(NodeService::height)
            .collect::<Vec<_>>(),
        halted,
        "half the voting power must not finalize a new block"
    );
    for i in 2..4 {
        let node = NodeService::new_or_load(homes[i].path().to_path_buf()).unwrap();
        node.start().await.unwrap();
        nodes[i] = Some(node);
    }
    let recovered_height = halted.into_iter().max().unwrap() + 2;
    eprintln!("phase: recovery after losing half the voting power");
    wait_for_height(&nodes, recovered_height).await;
    assert_certified_checkpoint(&nodes, &genesis, recovered_height);
    for node in nodes.iter().flatten() {
        node.stop().await.unwrap();
    }
    let journal = sprax_node::signing_journal::SigningJournal::open(
        &signing_path,
        genesis.fingerprint().unwrap(),
        &keys[3],
    )
    .unwrap();
    let after = journal.state().unwrap().unwrap();
    assert!((after.vote.height, after.vote.round) >= (before.vote.height, before.vote.round));
    assert_eq!(nodes[3].as_ref().unwrap().observed_evidence_count(), 0);
}
