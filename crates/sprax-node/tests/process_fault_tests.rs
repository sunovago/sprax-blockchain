//! Real OS child processes and TCP relays: no direct consensus-driver injection.
use sprax_consensus::{verify_block_commit, Validator, ValidatorSet};
use sprax_core::{GenesisAccount, GenesisConfig, GenesisValidator};
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_node::{Environment, KeyRecord, NodeConfig, NodeService};
use sprax_types::{Amount, Block, KeyType};
use std::{
    collections::BTreeMap,
    path::Path,
    process::{Child, Command, Stdio},
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::watch,
};

// Only the parent launches this ignored test with its private temporary directory.
#[tokio::test]
#[ignore = "subprocess entry point for four_process_partition_and_crash_recovery"]
async fn recovery_child() {
    let home = std::path::PathBuf::from(
        std::env::var_os("SPRAX_RECOVERY_CHILD_HOME").expect("parent must supply home"),
    );
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .try_init();
    let node = NodeService::new_or_load(home.clone()).unwrap();
    std::fs::write(home.join("loaded-height"), node.height().to_string()).unwrap();
    node.start().await.unwrap();
    std::fs::write(home.join("started"), b"ready").unwrap();
    let mut exported = 0;
    loop {
        while exported < node.height() {
            let height = exported + 1;
            let block = node.get_block_by_height(height).unwrap();
            std::fs::write(
                home.join(format!("checkpoint-{height}.json")),
                serde_json::to_vec(&block).unwrap(),
            )
            .unwrap();
            exported = height;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

struct Process(Child);
impl Process {
    fn launch(home: &Path) -> Self {
        let _ = std::fs::remove_file(home.join("started"));
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(home.join("child.log"))
            .unwrap();
        Self(
            Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "recovery_child", "--ignored", "--nocapture"])
                .env("SPRAX_RECOVERY_CHILD_HOME", home)
                .stdout(Stdio::from(log.try_clone().unwrap()))
                .stderr(Stdio::from(log))
                .spawn()
                .unwrap(),
        )
    }
    fn crash(&mut self) {
        self.0.kill().unwrap(); // SIGKILL on Unix / TerminateProcess on Windows; no NodeService::stop.
        self.0.wait().unwrap();
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct Relays(Vec<tokio::task::JoinHandle<()>>);
impl Drop for Relays {
    fn drop(&mut self) {
        for task in &self.0 {
            task.abort();
        }
    }
}

async fn relay(listener: TcpListener, target: u16, mut enabled: watch::Receiver<bool>) {
    let mut connections = tokio::task::JoinSet::new();
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let Ok((mut inbound, _)) = accepted else { break; };
                let mut gate = enabled.clone();
                connections.spawn(async move {
                    let Ok(mut outbound) = TcpStream::connect(("127.0.0.1", target)).await else { return; };
                    // Retain the copy future while paused: never discard bytes or corrupt framing.
                    // Buffering stays bounded by copy_bidirectional and OS socket buffers.
                    let copy = tokio::io::copy_bidirectional(&mut inbound, &mut outbound);
                    tokio::pin!(copy);
                    loop {
                        let open = *gate.borrow_and_update();
                        tokio::select! {
                            result = gate.changed() => { if result.is_err() { return; } }
                            _ = &mut copy, if open => return,
                        }
                    }
                });
            }
            result = enabled.changed() => { if result.is_err() { break; } }
            _ = connections.join_next(), if !connections.is_empty() => {}
        }
    }
}

fn block(home: &Path, height: u64) -> Option<Block> {
    serde_json::from_slice(&std::fs::read(home.join(format!("checkpoint-{height}.json"))).ok()?)
        .ok()
}
fn exported_height(home: &Path) -> u64 {
    let mut height = 0;
    while block(home, height + 1).is_some() {
        height += 1;
    }
    height
}
fn diagnostics(homes: &[tempfile::TempDir]) -> String {
    homes
        .iter()
        .enumerate()
        .map(|(i, home)| {
            let log = std::fs::read_to_string(home.path().join("child.log")).unwrap_or_default();
            format!(
                "node {i}, height {}:\n{}",
                exported_height(home.path()),
                log.lines().rev().take(12).collect::<Vec<_>>().join("\n")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
async fn wait_started(home: &Path, child: &mut Process) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while !home.join("started").exists() {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "child exited: {}",
            std::fs::read_to_string(home.join("child.log")).unwrap_or_default()
        );
        assert!(
            tokio::time::Instant::now() < deadline,
            "child startup deadline exceeded"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
async fn checkpoint(
    homes: &[tempfile::TempDir],
    indices: &[usize],
    height: u64,
    genesis: &GenesisConfig,
    validators: &ValidatorSet,
) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    while !indices
        .iter()
        .all(|&i| block(homes[i].path(), height).is_some())
    {
        assert!(
            tokio::time::Instant::now() < deadline,
            "checkpoint {height} timed out: {}",
            diagnostics(homes)
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let mut expected = None;
    for &i in indices {
        let block = block(homes[i].path(), height).unwrap();
        verify_block_commit(&block, validators, genesis.fingerprint().unwrap()).unwrap();
        assert!(block.last_commit.len() >= 3);
        let hash = Hasher::block_hash(&block.header).unwrap();
        assert_eq!(
            *expected.get_or_insert(hash),
            hash,
            "divergent finalized checkpoint"
        );
    }
}

#[tokio::test]
async fn four_process_partition_and_crash_recovery() {
    tokio::time::timeout(Duration::from_secs(300), fault_scenario())
        .await
        .expect("fault scenario deadline exceeded");
}
async fn fault_scenario() {
    let homes: Vec<_> = (0..4).map(|_| tempfile::tempdir().unwrap()).collect();
    let keys: Vec<_> = (71..=74)
        .map(|seed| Ed25519Keypair::from_seed(&[seed; 32]))
        .collect();
    let reservations: Vec<_> = (0..4)
        .map(|_| std::net::TcpListener::bind("127.0.0.1:0").unwrap())
        .collect();
    let ports: Vec<_> = reservations
        .iter()
        .map(|socket| socket.local_addr().unwrap().port())
        .collect();
    let mut bootstrap = vec![Vec::new(); 4];
    let mut gates = Vec::new();
    let mut relays = Relays(Vec::new());
    for (i, peers) in bootstrap.iter_mut().enumerate() {
        for (j, &target) in ports.iter().enumerate() {
            if i == j {
                continue;
            }
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            peers.push(listener.local_addr().unwrap().to_string());
            let (gate, receiver) = watch::channel(true);
            gates.push((i, j, gate));
            relays
                .0
                .push(tokio::spawn(relay(listener, target, receiver)));
        }
    }
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
        config.network.bootstrap_peers = bootstrap[i].clone();
        config.rpc.json_rpc_port = 0;
        config.consensus.enabled = true;
        config.consensus.local_validator_key_name = Some("validator".into());
        config
            .save_to_file(&home.path().join("config.toml"))
            .unwrap();
    }

    let validators = ValidatorSet::new(
        keys.iter()
            .map(|key| Validator::new(key.address(), key.public_key_bytes().to_vec(), 100))
            .collect(),
    )
    .unwrap();
    drop(reservations);
    let mut children = Vec::new();
    for home in &homes {
        let mut child = Process::launch(home.path());
        wait_started(home.path(), &mut child).await;
        children.push(child);
    }
    checkpoint(&homes, &[0, 1, 2, 3], 2, &genesis, &validators).await;
    eprintln!("partition: two live groups of two validators");
    for (i, j, gate) in &gates {
        gate.send_replace((i < &2) == (j < &2));
    }
    tokio::time::sleep(Duration::from_secs(3)).await;
    let halted: Vec<_> = homes.iter().map(|h| exported_height(h.path())).collect();
    tokio::time::sleep(Duration::from_secs(8)).await;
    assert_eq!(
        halted,
        homes
            .iter()
            .map(|h| exported_height(h.path()))
            .collect::<Vec<_>>(),
        "half-power partition finalized new blocks"
    );
    for child in &mut children {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "partition killed a validator"
        );
    }
    for (_, _, gate) in &gates {
        gate.send_replace(true);
    }
    let healed = *halted.iter().max().unwrap() + 2;
    checkpoint(&homes, &[0, 1, 2, 3], healed, &genesis, &validators).await;

    eprintln!("partition: three-validator majority and one live isolated validator");
    for (i, j, gate) in &gates {
        gate.send_replace((*i == 3) == (*j == 3));
    }
    tokio::time::sleep(Duration::from_secs(3)).await;
    let minority_height = exported_height(homes[3].path());
    let majority_target = homes
        .iter()
        .map(|h| exported_height(h.path()))
        .max()
        .unwrap()
        + 3;
    checkpoint(&homes, &[0, 1, 2], majority_target, &genesis, &validators).await;
    assert_eq!(
        exported_height(homes[3].path()),
        minority_height,
        "isolated validator finalized without quorum"
    );
    assert!(
        children[3].0.try_wait().unwrap().is_none(),
        "isolated validator exited"
    );
    for (_, _, gate) in &gates {
        gate.send_replace(true);
    }
    checkpoint(
        &homes,
        &[0, 1, 2, 3],
        majority_target + 1,
        &genesis,
        &validators,
    )
    .await;

    eprintln!("crash: terminate validator 3 without graceful shutdown");
    children[3].crash();
    let crashed_height = exported_height(homes[3].path());
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
            .expect("crashed validator never signed")
    };
    let advanced = homes
        .iter()
        .map(|h| exported_height(h.path()))
        .max()
        .unwrap()
        + 2;
    checkpoint(&homes, &[0, 1, 2], advanced, &genesis, &validators).await;
    children[3] = Process::launch(homes[3].path());
    wait_started(homes[3].path(), &mut children[3]).await;
    let loaded_height: u64 = std::fs::read_to_string(homes[3].path().join("loaded-height"))
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        loaded_height >= crashed_height,
        "restart lost finalized storage"
    );
    checkpoint(&homes, &[0, 1, 2, 3], advanced + 2, &genesis, &validators).await;
    // Compare every finalized checkpoint, including those produced before and during faults.
    for height in 1..=advanced + 2 {
        checkpoint(&homes, &[0, 1, 2, 3], height, &genesis, &validators).await;
    }
    for child in &mut children {
        child.crash();
    }
    let journal = sprax_node::signing_journal::SigningJournal::open(
        &signing_path,
        genesis.fingerprint().unwrap(),
        &keys[3],
    )
    .unwrap();
    let after = journal.state().unwrap().unwrap();
    assert!((after.vote.height, after.vote.round) >= (before.vote.height, before.vote.round));
}
