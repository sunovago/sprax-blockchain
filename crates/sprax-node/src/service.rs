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
    keyring: Arc<RwLock<Keyring>>,
    p2p: Arc<RwLock<Option<P2pService>>>,
    is_running: Arc<AtomicBool>,
    is_syncing: Arc<AtomicBool>,
}

impl NodeService {
    /// Initializes or loads a node service from a home directory.
    pub fn new_or_load(home: PathBuf) -> Result<Self, NodeError> {
        let config_file = home.join("config.toml");
        let config = if config_file.exists() {
            NodeConfig::load_from_file(&config_file)?
        } else {
            NodeConfig::for_environment(crate::Environment::Development, home.clone())
        };

        let keyring_dir = home.join("keyring");
        let keyring = Keyring::open_or_create(&keyring_dir)?;

        let data_dir = home.join("data");
        let genesis_file = home.join("genesis.json");

        let genesis = if genesis_file.exists() {
            GenesisConfig::load_from_file(&genesis_file)
                .map_err(|e| NodeError::ConfigError(e.to_string()))?
        } else {
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

        let store = RedbStore::open(&data_dir.join("state.redb"))
            .map_err(|e| NodeError::StorageError(e.to_string()))?;
        let genesis_for_ledger = genesis.clone();
        let ledger = ChainLedger::open_or_init(store, move || Ok(genesis_for_ledger))
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        let staking_file = data_dir.join("staking.json");
        let staking = if staking_file.exists() {
            StakingKeeper::load_from_file(&staking_file)
                .map_err(|e| NodeError::StorageError(e.to_string()))?
        } else {
            let mut keeper = StakingKeeper::default();
            for v in &genesis.validators {
                keeper
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
            }
            keeper
                .save_to_file(&staking_file)
                .map_err(|e| NodeError::StorageError(e.to_string()))?;
            keeper
        };

        Ok(Self {
            config,
            ledger: Arc::new(RwLock::new(ledger)),
            staking: Arc::new(RwLock::new(staking)),
            keyring: Arc::new(RwLock::new(keyring)),
            p2p: Arc::new(RwLock::new(None)),
            is_running: Arc::new(AtomicBool::new(false)),
            is_syncing: Arc::new(AtomicBool::new(false)),
        })
    }

    #[must_use]
    pub fn config(&self) -> &NodeConfig {
        &self.config
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
        let receipts = guard
            .apply_block(block)
            .map_err(|e| NodeError::RuntimeError(e.to_string()))?;

        Ok(receipts)
    }

    /// Applies a batch of historical blocks for catch-up synchronization.
    pub fn apply_blocks_batch(&self, blocks: Vec<Block>) -> Result<usize, NodeError> {
        let mut guard = self.ledger.write();
        let count = guard
            .apply_blocks_batch(blocks)
            .map_err(|e| NodeError::RuntimeError(e.to_string()))?;

        Ok(count)
    }

    /// Starts the background node service and P2P networking layer.
    pub async fn start(&self) -> Result<(), NodeError> {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return Err(NodeError::RuntimeError("node is already running".into()));
        }

        let (inbound_tx_tx, mut inbound_tx_rx) = mpsc::unbounded_channel::<Transaction>();
        let (inbound_block_tx, mut inbound_block_rx) = mpsc::unbounded_channel::<Block>();
        let (inbound_vote_tx, inbound_vote_rx) = mpsc::unbounded_channel();
        let (inbound_proposal_tx, inbound_proposal_rx) = mpsc::unbounded_channel();
        let (inbound_evidence_tx, inbound_evidence_rx) = mpsc::unbounded_channel();

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
        let _ = p2p_service.start(header.height, latest_hash).await;
        *self.p2p.write() = Some(p2p_service.clone());

        info!(
            chain_id = %self.config.chain_id,
            environment = %self.config.environment,
            height = self.height(),
            p2p_port = self.config.network.p2p_port,
            "SPRX Multi-Node Service online"
        );

        let ledger_clone = Arc::clone(&self.ledger);
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
                        let _ = guard.apply_block(block);
                    }
                    else => break,
                }
            }
        });

        tokio::spawn(crate::consensus_driver::run_evidence_listener(
            Arc::clone(&self.staking),
            inbound_evidence_rx,
            Arc::clone(&self.is_running),
        ));

        let local_validator_key = self
            .config
            .consensus
            .local_validator_key_name
            .as_ref()
            .and_then(|name| self.keyring.read().get_ed25519_keypair(name).ok());

        if self.config.consensus.enabled {
            if let Some(val_key) = local_validator_key {
                let val_set = self
                    .staking
                    .read()
                    .get_active_validator_set()
                    .map_err(|e| {
                        NodeError::ConfigError(format!(
                            "cannot start consensus driver: no active validator set: {e}"
                        ))
                    })?;
                let engine = sprax_consensus::BftConsensusEngine::new(self.height() + 1, val_set);
                let driver = ConsensusDriver::new(
                    engine,
                    Arc::clone(&self.staking),
                    Arc::clone(&self.ledger),
                    p2p_service,
                    val_key,
                    self.config.consensus.timeouts.clone(),
                    inbound_vote_rx,
                    inbound_proposal_rx,
                    Arc::clone(&self.is_running),
                    self.config.network.bootstrap_peers.len(),
                );
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
            if let Err(e) = crate::rpc_server::RpcServer::start(self.clone(), rpc_port).await {
                warn!("failed to start JSON-RPC server on port {rpc_port}: {e}");
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

        info!("SPRX Local Node Service stopped cleanly");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
