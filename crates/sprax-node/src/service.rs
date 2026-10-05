use crate::{
    config::NodeConfig, consensus_driver::ConsensusDriver, error::NodeError, keyring::Keyring,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sprax_consensus::{CommissionRates, StakingKeeper, ValidatorDescription};
use sprax_core::{AccountState, ChainLedger, GenesisAccount, GenesisConfig, GenesisValidator};
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_network::{BlockFetchFn, P2pService, PeerHandle, PeerId};
use sprax_storage::RedbStore;
use sprax_types::{Address, Amount, Block, BlockHeader, Hash32, Transaction, TxReceipt};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc;
use tracing::{info, warn};

/// Node runtime monitoring and development metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub chain_id: String,
    pub height: u64,
    pub latest_block_hash: Hash32,
    pub state_root: Hash32,
    pub connected_peers: usize,
    pub total_transactions: usize,
    pub mempool_pending: usize,
    pub status: String,
    pub is_syncing: bool,
}

/// Master Node Service Runtime managing local blockchain state, P2P network, and block production.
#[derive(Debug, Clone)]
pub struct NodeService {
    config: NodeConfig,
    ledger: Arc<RwLock<ChainLedger<RedbStore>>>,
    staking: Arc<RwLock<StakingKeeper>>,
    evidence_pool: Arc<RwLock<crate::evidence_pool::EvidencePool>>,
    keyring: Arc<RwLock<Keyring>>,
    p2p: Arc<RwLock<Option<P2pService>>>,
    rpc_server: Arc<RwLock<Option<crate::rpc_server::RpcServerHandle>>>,
    is_running: Arc<AtomicBool>,
    is_syncing: Arc<AtomicBool>,
}

impl NodeService {
    /// Initializes or loads a node service from a home directory.
    pub fn new_or_load(home: PathBuf) -> Result<Self, NodeError> {
        let config_file = home.join("config.toml");
        let mut config = if config_file.exists() {
            NodeConfig::load_from_file(&config_file)?
        } else {
            NodeConfig::for_environment(crate::Environment::Development, home.clone())
        };

        // The selected home is authoritative after moving/restoring a node directory.
        // A stale path in TOML must never send signing state to another directory.
        config.home_dir = home.clone();
        let keyring_dir = home.join("keyring");
        let keyring = Keyring::open_or_create_with_development_keys(
            &keyring_dir,
            config.environment == crate::Environment::Development,
        )?;

        let data_dir = home.join("data");
        let genesis_file = home.join("genesis.json");

        let genesis = if genesis_file.exists() {
            GenesisConfig::load_from_file(&genesis_file)
                .map_err(|e| NodeError::ConfigError(e.to_string()))?
        } else {
            if config.environment != crate::Environment::Development {
                return Err(NodeError::ConfigError("testnet/mainnet requires an explicit genesis file; development keys cannot be used".into()));
            }
            let alice_kp = Ed25519Keypair::from_seed(&[1u8; 32]);
            let bob_kp = Ed25519Keypair::from_seed(&[2u8; 32]);
            let charlie_kp = Ed25519Keypair::from_seed(&[3u8; 32]);

            let mut default_genesis = GenesisConfig::default_development();
            default_genesis.chain_id = config.chain_id.clone();
            default_genesis.accounts = vec![
                GenesisAccount {
                    name: "alice".to_string(),
                    address: alice_kp.address(),
                    initial_balance: Amount::from_sprx_whole(1_000_000).unwrap(),
                },
                GenesisAccount {
                    name: "bob".to_string(),
                    address: bob_kp.address(),
                    initial_balance: Amount::from_sprx_whole(500_000).unwrap(),
                },
                GenesisAccount {
                    name: "charlie".to_string(),
                    address: charlie_kp.address(),
                    initial_balance: Amount::from_sprx_whole(100_000).unwrap(),
                },
            ];

            default_genesis.validators = vec![
                GenesisValidator {
                    operator_address: alice_kp.address(),
                    consensus_pubkey: alice_kp.public_key_bytes().to_vec(),
                    self_stake: Amount::from_sprx_whole(100_000).unwrap(),
                    moniker: "alice".to_string(),
                },
                GenesisValidator {
                    operator_address: bob_kp.address(),
                    consensus_pubkey: bob_kp.public_key_bytes().to_vec(),
                    self_stake: Amount::from_sprx_whole(50_000).unwrap(),
                    moniker: "bob".to_string(),
                },
                GenesisValidator {
                    operator_address: charlie_kp.address(),
                    consensus_pubkey: charlie_kp.public_key_bytes().to_vec(),
                    self_stake: Amount::from_sprx_whole(25_000).unwrap(),
                    moniker: "charlie".to_string(),
                },
            ];
            default_genesis
                .save_to_file(&genesis_file)
                .map_err(|e| NodeError::ConfigError(e.to_string()))?;
            default_genesis
        };

        if genesis.chain_id != config.chain_id {
            return Err(NodeError::ConfigError(
                "genesis chain ID does not match node configuration".into(),
            ));
        }
        if config.environment != crate::Environment::Development {
            if genesis.validators.is_empty() {
                return Err(NodeError::ConfigError(
                    "testnet/mainnet genesis requires validators".into(),
                ));
            }
            let known_keys: Vec<_> = (1..=3)
                .map(|n| {
                    Ed25519Keypair::from_seed(&[n; 32])
                        .public_key_bytes()
                        .to_vec()
                })
                .collect();
            if genesis
                .validators
                .iter()
                .any(|v| known_keys.contains(&v.consensus_pubkey))
            {
                return Err(NodeError::ConfigError(
                    "public development validator keys are forbidden outside development".into(),
                ));
            }
        }

        let store = RedbStore::open(&data_dir.join("state.redb"))
            .map_err(|e| NodeError::StorageError(e.to_string()))?;
        let genesis_for_ledger = genesis.clone();
        let ledger = ChainLedger::open_or_init(store, move || Ok(genesis_for_ledger))
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        // staking.json is an operational cache, never an input to consensus power.
        // Preserve legacy files, but rebuild the display cache from committed ledger state.
        let mut staking = StakingKeeper::new(sprax_consensus::StakingParams {
            // Genesis admission was validated by the ledger; display-cache defaults must
            // not reject an otherwise valid agreed genesis.
            min_self_stake: Amount::ZERO,
            ..Default::default()
        });
        for v in &genesis.validators {
            staking
                .register_validator(
                    v.operator_address,
                    v.consensus_pubkey.clone(),
                    ValidatorDescription {
                        moniker: v.moniker.clone(),
                        identity: String::new(),
                        website: String::new(),
                        details: String::new(),
                    },
                    v.self_stake,
                    CommissionRates::default(),
                )
                .map_err(|e| NodeError::StorageError(e.to_string()))?;
            let stake = ledger
                .get_validator_stake(&v.operator_address)
                .map_err(|e| NodeError::StorageError(e.to_string()))?;
            staking.sync_validator_tokens(&v.operator_address, stake.tokens);
        }

        Ok(Self {
            config,
            ledger: Arc::new(RwLock::new(ledger)),
            staking: Arc::new(RwLock::new(staking)),
            evidence_pool: Arc::new(RwLock::new(crate::evidence_pool::EvidencePool::default())),
            keyring: Arc::new(RwLock::new(keyring)),
            p2p: Arc::new(RwLock::new(None)),
            rpc_server: Arc::new(RwLock::new(None)),
            is_running: Arc::new(AtomicBool::new(false)),
            is_syncing: Arc::new(AtomicBool::new(false)),
        })
    }

    #[must_use]
    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    pub fn override_listen_ports(&mut self, p2p_port: Option<u16>, rpc_port: Option<u16>) {
        if let Some(port) = p2p_port {
            self.config.network.p2p_port = port;
        }
        if let Some(port) = rpc_port {
            self.config.rpc.json_rpc_port = port;
        }
    }

    pub fn override_bootstrap_peers(&mut self, peers: Vec<String>) {
        self.config.network.bootstrap_peers = peers;
    }

    #[must_use]
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    #[must_use]
    pub fn is_syncing(&self) -> bool {
        self.is_syncing.load(Ordering::SeqCst)
    }

    pub fn observed_evidence_count(&self) -> usize {
        self.evidence_pool.read().len()
    }

    pub fn canonical_validator_set(&self) -> Result<sprax_consensus::ValidatorSet, NodeError> {
        sprax_consensus::ValidatorSet::from_canonical(
            self.ledger
                .read()
                .active_validators()
                .map_err(|e| NodeError::RuntimeError(e.to_string()))?,
        )
        .map_err(|e| NodeError::RuntimeError(e.to_string()))
    }

    pub fn keyring(&self) -> Arc<RwLock<Keyring>> {
        Arc::clone(&self.keyring)
    }

    pub fn staking(&self) -> Arc<RwLock<StakingKeeper>> {
        Arc::clone(&self.staking)
    }

    pub fn chain_id(&self) -> String {
        self.ledger.read().chain_id().to_string()
    }

    pub fn height(&self) -> u64 {
        self.ledger.read().height()
    }

    pub fn latest_header(&self) -> BlockHeader {
        self.ledger.read().latest_header().clone()
    }

    pub fn state_root(&self) -> Result<Hash32, NodeError> {
        self.ledger
            .read()
            .state_root()
            .map_err(|e| NodeError::StorageError(e.to_string()))
    }

    pub fn get_account(&self, addr: &Address) -> Result<AccountState, NodeError> {
        self.ledger
            .read()
            .get_account(addr)
            .map_err(|e| NodeError::StorageError(e.to_string()))
    }

    pub fn get_validator_stake(
        &self,
        validator: &Address,
    ) -> Result<sprax_core::state::ValidatorStakeState, NodeError> {
        self.ledger
            .read()
            .get_validator_stake(validator)
            .map_err(|e| NodeError::StorageError(e.to_string()))
    }

    pub fn get_block_by_height(&self, height: u64) -> Option<Block> {
        self.ledger.read().get_block_by_height(height).cloned()
    }

    pub fn get_block_by_hash(&self, hash: &Hash32) -> Option<Block> {
        self.ledger.read().get_block_by_hash(hash).cloned()
    }

    pub fn get_blocks_range(&self, from_height: u64, to_height: u64) -> Vec<Block> {
        let guard = self.ledger.read();
        let mut blocks = Vec::new();
        for h in from_height..=to_height {
            if let Some(b) = guard.get_block_by_height(h) {
                blocks.push(b.clone());
            } else {
                break;
            }
        }
        blocks
    }

    pub fn get_transaction(&self, hash: &Hash32) -> Option<(Transaction, TxReceipt, u64)> {
        self.ledger
            .read()
            .get_transaction(hash)
            .map(|(tx, r, h)| (tx.clone(), r.clone(), h))
    }

    pub fn connected_peers(&self) -> Vec<PeerHandle> {
        if let Some(p2p) = self.p2p.read().as_ref() {
            p2p.connected_peers()
        } else {
            vec![]
        }
    }

    pub fn connected_peers_count(&self) -> usize {
        if let Some(p2p) = self.p2p.read().as_ref() {
            p2p.connected_peers_count()
        } else {
            0
        }
    }

    /// Exposes comprehensive real-time runtime metrics.
    pub fn metrics(&self) -> NodeMetrics {
        let header = self.latest_header();
        let latest_hash = Hasher::block_hash(&header).unwrap_or(Hash32::ZERO);
        let height = header.height;
        let mempool_len = self.ledger.read().mempool_len();
        let peers_count = self.connected_peers_count();
        let is_sync = self.is_syncing();

        let status = if !self.is_running() {
            "offline".to_string()
        } else if is_sync {
            "syncing".to_string()
        } else if peers_count > 0 {
            "online".to_string()
        } else {
            "idle".to_string()
        };

        NodeMetrics {
            chain_id: self.chain_id(),
            height,
            latest_block_hash: latest_hash,
            state_root: header.state_root,
            connected_peers: peers_count,
            total_transactions: height as usize, // approximate block txs
            mempool_pending: mempool_len,
            status,
            is_syncing: is_sync,
        }
    }

    /// Submits a signed transaction to the ledger mempool and gossips to connected peers.
    pub fn submit_transaction(&self, tx: Transaction) -> Result<Hash32, NodeError> {
        let mut guard = self.ledger.write();
        let hash = guard
            .submit_transaction(tx.clone())
            .map_err(|e| NodeError::RuntimeError(e.to_string()))?;

        if let Some(p2p) = self.p2p.read().as_ref() {
            p2p.broadcast_tx(tx);
        }

        Ok(hash)
    }

    pub fn mine_block(&self, proposer: Address) -> Result<Block, NodeError> {
        if self.config.environment != crate::Environment::Development
            || self.config.consensus.enabled
        {
            return Err(NodeError::RuntimeError(
                "manual mining requires development mode with consensus disabled".into(),
            ));
        }
        let mut guard = self.ledger.write();
        let block = guard
            .mine_block(proposer)
            .map_err(|e| NodeError::RuntimeError(e.to_string()))?;
        drop(guard);

        info!(
            height = block.header.height,
            txs = block.body.transactions.len(),
            state_root = %block.header.state_root,
            "Block committed successfully"
        );

        if let Some(p2p) = self.p2p.read().as_ref() {
            p2p.broadcast_block(block.clone());
        }

        Ok(block)
    }

    pub fn apply_block(&self, block: Block) -> Result<Vec<TxReceipt>, NodeError> {
        let mut guard = self.ledger.write();
        if block.header.height <= guard.height() {
            let known = guard
                .get_block_by_height(block.header.height)
                .is_some_and(|stored| stored.header == block.header && stored.body == block.body);
            if known {
                return Ok(Vec::new());
            }
            return Err(NodeError::RuntimeError(
                "conflicting historical block".into(),
            ));
        }
        if self.config.environment != crate::Environment::Development
            || self.config.consensus.enabled
            || !block.last_commit.is_empty()
        {
            let validators = sprax_consensus::ValidatorSet::from_canonical(
                guard
                    .active_validators()
                    .map_err(|e| NodeError::RuntimeError(e.to_string()))?,
            )
            .map_err(|e| NodeError::RuntimeError(e.to_string()))?;
            sprax_consensus::verify_block_commit(&block, &validators)
                .map_err(|e| NodeError::RuntimeError(e.to_string()))?;
        }
        let receipts = guard
            .apply_block(block)
            .map_err(|e| NodeError::RuntimeError(e.to_string()))?;

        Ok(receipts)
    }

    pub fn query_contract(
        &self,
        address: Address,
        message: &[u8],
        gas: u64,
    ) -> Result<(Vec<u8>, u64), NodeError> {
        let result = self
            .ledger
            .read()
            .query_contract(address, message, gas)
            .map_err(|e| NodeError::RuntimeError(e.to_string()))?;
        Ok((result.data, result.gas_used))
    }

    /// Applies a batch of historical blocks for catch-up synchronization.
    pub fn apply_blocks_batch(&self, blocks: Vec<Block>) -> Result<usize, NodeError> {
        let mut count = 0;
        for block in blocks {
            self.apply_block(block)?;
            count += 1;
        }
        Ok(count)
    }

    /// Starts the background node service and P2P networking layer.
    pub async fn start(&self) -> Result<(), NodeError> {
        if self.is_running() {
            return Err(NodeError::RuntimeError("node is already running".into()));
        }
        let local_validator_key = if self.config.consensus.enabled {
            self.config
                .consensus
                .local_validator_key_name
                .as_ref()
                .map(|name| self.keyring.read().get_ed25519_keypair(name))
                .transpose()?
        } else {
            None
        };
        let initial_val_set = if let Some(key) = &local_validator_key {
            let validators = self.canonical_validator_set()?;
            if !validators
                .validators()
                .iter()
                .any(|v| v.address == key.address() && v.public_key == key.public_key_bytes())
            {
                return Err(NodeError::ConfigError(
                    "configured signing key is not an active validator".into(),
                ));
            }
            Some(validators)
        } else {
            None
        };

        let signing_journal = if self.config.consensus.enabled {
            local_validator_key
                .as_ref()
                .map(|key| {
                    let genesis = serde_json::to_vec(self.ledger.read().genesis())
                        .map_err(|e| NodeError::ConfigError(e.to_string()))?;
                    crate::signing_journal::SigningJournal::open(
                        &self.config.home_dir.join("data/validator-signing.redb"),
                        Hasher::sha256(&genesis),
                        key,
                    )
                    .map_err(NodeError::StorageError)
                })
                .transpose()?
        } else {
            None
        };

        if self.is_running.swap(true, Ordering::SeqCst) {
            return Err(NodeError::RuntimeError("node is already running".into()));
        }

        let (inbound_tx_tx, mut inbound_tx_rx) = mpsc::channel::<Transaction>(128);
        let (inbound_block_tx, mut inbound_block_rx) = mpsc::channel::<Block>(16);
        let (inbound_vote_tx, inbound_vote_rx) = mpsc::channel(4096);
        let (inbound_proposal_tx, inbound_proposal_rx) = mpsc::channel(16);
        let (inbound_evidence_tx, inbound_evidence_rx) = mpsc::channel(128);

        let peer_id_seed = format!(
            "{}:{}:{}",
            self.config.chain_id, self.config.network.listen_addr, self.config.network.p2p_port
        );
        let pubkey_hash = Hasher::blake3(peer_id_seed.as_bytes());
        let local_peer_id = PeerId::from_pubkey_hash(&pubkey_hash);

        let ledger_for_fetch = Arc::clone(&self.ledger);
        let block_fetch_fn: BlockFetchFn = Arc::new(move |from_height, to_height| {
            let guard = ledger_for_fetch.read();
            let mut blocks = Vec::new();
            for h in from_height..=to_height {
                match guard.get_block_by_height(h) {
                    Some(b) => blocks.push(b.clone()),
                    None => break,
                }
            }
            blocks
        });

        let p2p_service = P2pService::new(
            local_peer_id,
            self.chain_id(),
            self.config.network.clone(),
            inbound_tx_tx,
            inbound_block_tx,
            inbound_vote_tx,
            inbound_proposal_tx,
            inbound_evidence_tx,
            block_fetch_fn,
        );

        let header = self.latest_header();
        let latest_hash = Hasher::block_hash(&header).unwrap_or(Hash32::ZERO);

        // Start P2P listener & bootstrap dialing
        if let Err(error) = p2p_service.start(header.height, latest_hash).await {
            self.is_running.store(false, Ordering::SeqCst);
            return Err(NodeError::RuntimeError(format!(
                "failed to start P2P: {error}"
            )));
        }
        *self.p2p.write() = Some(p2p_service.clone());

        info!(
            chain_id = %self.config.chain_id,
            environment = %self.config.environment,
            height = self.height(),
            p2p_port = self.config.network.p2p_port,
            "SPRX Multi-Node Service online"
        );

        let ledger_clone = Arc::clone(&self.ledger);
        let require_commit = self.config.environment != crate::Environment::Development
            || self.config.consensus.enabled;
        let is_running = Arc::clone(&self.is_running);

        // Background Inbound Gossip Processor
        tokio::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                tokio::select! {
                    Some(tx) = inbound_tx_rx.recv() => {
                        let mut guard = ledger_clone.write();
                        let _ = guard.submit_transaction(tx);
                    }
                    Some(block) = inbound_block_rx.recv() => {
                        let mut guard = ledger_clone.write();
                        if block.header.height <= guard.height() { continue; }
                        if require_commit || !block.last_commit.is_empty() {
                            let validators = guard.active_validators()
                                .map_err(|e| e.to_string())
                                .and_then(|v| sprax_consensus::ValidatorSet::from_canonical(v).map_err(|e| e.to_string()));
                            match validators {
                                Ok(validators) if sprax_consensus::verify_block_commit(&block, &validators).is_ok() => {}
                                _ => { warn!("rejected block gossip without a valid quorum certificate"); continue; }
                            }
                        }
                        let _ = guard.apply_block(block);
                    }
                    else => break,
                }
            }
        });

        tokio::spawn(crate::consensus_driver::run_evidence_listener(
            Arc::clone(&self.ledger),
            Arc::clone(&self.evidence_pool),
            inbound_evidence_rx,
            Arc::clone(&self.is_running),
        ));

        if self.config.consensus.enabled {
            if let Some(val_key) = local_validator_key {
                let val_set = initial_val_set.ok_or_else(|| {
                    NodeError::ConfigError("missing initial validator set".into())
                })?;
                let engine = sprax_consensus::BftConsensusEngine::new(self.height() + 1, val_set);
                let driver = ConsensusDriver::new(
                    engine,
                    Arc::clone(&self.staking),
                    Arc::clone(&self.ledger),
                    p2p_service,
                    val_key,
                    Arc::clone(&self.evidence_pool),
                    signing_journal.ok_or_else(|| {
                        NodeError::RuntimeError("missing validator signing journal".into())
                    })?,
                    self.config.consensus.timeouts.clone(),
                    inbound_vote_rx,
                    inbound_proposal_rx,
                    Arc::clone(&self.is_running),
                    self.config.network.bootstrap_peers.len(),
                )
                .map_err(|e| NodeError::RuntimeError(e.to_string()))?;
                tokio::spawn(driver.run());
            } else {
                warn!(
                    "consensus.enabled is true but no local_validator_key_name configured; \
                     running as a non-validator full node relying on block gossip/sync"
                );
            }
        } else {
            let is_running = Arc::clone(&self.is_running);
            tokio::spawn(async move {
                let mut vote_rx = inbound_vote_rx;
                let mut proposal_rx = inbound_proposal_rx;
                while is_running.load(Ordering::SeqCst) {
                    tokio::select! {
                        msg = vote_rx.recv() => if msg.is_none() { break; },
                        msg = proposal_rx.recv() => if msg.is_none() { break; },
                        else => break,
                    }
                }
            });
            if self.config.consensus.enabled {
                warn!(
                    "consensus.enabled is true but no local_validator_key_name configured; \
                     running as a non-validator full node relying on block gossip/sync"
                );
            }
        }

        if self.config.rpc.enable_json_rpc {
            let rpc_port = self.config.rpc.json_rpc_port;
            match crate::rpc_server::RpcServer::start(self.clone(), rpc_port).await {
                Ok(handle) => *self.rpc_server.write() = Some(handle),
                Err(error) => {
                    self.stop().await?;
                    return Err(error);
                }
            }
        }

        Ok(())
    }

    /// Gracefully stops the node service.
    pub async fn stop(&self) -> Result<(), NodeError> {
        if !self.is_running.swap(false, Ordering::SeqCst) {
            return Ok(());
        }

        if let Some(p2p) = self.p2p.write().take() {
            p2p.stop();
        }

        let rpc_handle = self.rpc_server.write().take();
        if let Some(handle) = rpc_handle {
            handle.stop().await;
        }
        info!("SPRX Local Node Service stopped cleanly");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn listen_port_overrides_bind_actual_p2p_and_rpc_sockets() {
        let temp = tempfile::tempdir().unwrap();
        let first = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let second = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let p2p = first.local_addr().unwrap().port();
        let rpc = second.local_addr().unwrap().port();
        drop((first, second));
        let mut service = NodeService::new_or_load(temp.path().to_path_buf()).unwrap();
        service.override_listen_ports(Some(p2p), Some(rpc));
        service.start().await.unwrap();
        for port in [p2p, rpc] {
            let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
            loop {
                if tokio::net::TcpStream::connect(("127.0.0.1", port))
                    .await
                    .is_ok()
                {
                    break;
                }
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "overridden port must actually bind"
                );
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
        service.stop().await.unwrap();
        let rebound_rpc = tokio::net::TcpListener::bind(("127.0.0.1", rpc))
            .await
            .unwrap();
        drop(rebound_rpc);
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            match tokio::net::TcpListener::bind(("127.0.0.1", p2p)).await {
                Ok(listener) => {
                    drop(listener);
                    break;
                }
                Err(_) => {
                    assert!(tokio::time::Instant::now() < deadline);
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }
        }
        service.start().await.unwrap();
        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn rpc_bind_failure_rolls_back_startup_and_allows_retry() {
        let temp = tempfile::tempdir().unwrap();
        let occupied = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let rpc = occupied.local_addr().unwrap().port();
        let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let p2p = reserved.local_addr().unwrap().port();
        drop(reserved);
        let mut service = NodeService::new_or_load(temp.path().to_path_buf()).unwrap();
        service.override_listen_ports(Some(p2p), Some(rpc));
        assert!(service.start().await.is_err());
        assert!(!service.is_running());
        drop(occupied);
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            match tokio::net::TcpListener::bind(("127.0.0.1", p2p)).await {
                Ok(listener) => {
                    drop(listener);
                    break;
                }
                Err(_) => {
                    assert!(tokio::time::Instant::now() < deadline);
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }
        }
        service.start().await.unwrap();
        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn invalid_validator_key_configuration_fails_before_signing_or_listening() {
        let temp = tempfile::tempdir().unwrap();
        let mut service = NodeService::new_or_load(temp.path().to_path_buf()).unwrap();
        service.config.consensus.enabled = true;
        service.config.consensus.local_validator_key_name = Some("missing".into());
        assert!(service.start().await.is_err());
        assert!(!service.is_running());
        service
            .keyring
            .write()
            .create_key("outsider", sprax_types::KeyType::Ed25519)
            .unwrap();
        service.config.consensus.local_validator_key_name = Some("outsider".into());
        assert!(service
            .start()
            .await
            .unwrap_err()
            .to_string()
            .contains("not an active validator"));
        assert!(!service.is_running());
        assert!(!temp.path().join("data/validator-signing.redb").exists());
    }

    #[tokio::test]
    async fn test_node_service_lifecycle() {
        let temp_dir = tempfile::tempdir().unwrap();
        let service = NodeService::new_or_load(temp_dir.path().to_path_buf()).unwrap();
        assert!(!service.is_running());

        service.start().await.unwrap();
        assert!(service.is_running());

        // Starting again should error
        assert!(service.start().await.is_err());

        let metrics = service.metrics();
        assert_eq!(metrics.height, 0);

        service.stop().await.unwrap();
        assert!(!service.is_running());
    }
}
