use crate::{
    config::NetworkConfig,
    error::NetworkError,
    identity::NetworkIdentity,
    limits::{NetworkLimits, OutboundByteBudget, ResourceManager},
    message::{NetworkMessage, CURRENT_PROTOCOL_VERSION},
    peer::{PeerId, PeerScore},
    transport::{AuthenticatedChannel, EncryptedReader, EncryptedWriter},
};
use parking_lot::RwLock;
use sprax_consensus::{EquivocationEvidence, SignedProposal, Vote};
use sprax_types::{Block, Hash32, Transaction};
use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, watch, OwnedSemaphorePermit, Semaphore},
};
use tracing::{debug, info, warn};

/// Live connection handle to an active P2P peer.
#[derive(Debug, Clone)]
pub struct PeerHandle {
    pub peer_id: PeerId,
    pub remote_addr: String,
    pub height: u64,
    pub latest_block_hash: Hash32,
    pub score: PeerScore,
    pub byte_budget: Arc<OutboundByteBudget>,
    sender: mpsc::Sender<NetworkMessage>,
    preferred_direction: bool,
    disconnect: watch::Sender<bool>,
}

impl PeerHandle {
    pub fn send(&self, msg: NetworkMessage) -> Result<(), NetworkError> {
        let is_consensus = matches!(
            msg,
            NetworkMessage::Vote(_)
                | NetworkMessage::Proposal(_)
                | NetworkMessage::ProposalRequest { .. }
                | NetworkMessage::ProposalResponse { .. }
                | NetworkMessage::Evidence(_)
        );

        // Under queue pressure, drop bulk transaction gossip to protect consensus progress
        if !is_consensus && self.byte_budget.current_queued_bytes() > 6 * 1024 * 1024 {
            return Err(NetworkError::BudgetExceeded(
                "dropping low-priority gossip under outbound queue pressure".into(),
            ));
        }

        self.sender
            .try_send(msg)
            .map_err(|e| NetworkError::ConnectionFailed(format!("failed to send message: {e}")))
    }
}

pub type BlockFetchFn = Arc<dyn Fn(u64, u64) -> Vec<Block> + Send + Sync>;

pub type ProposalFetchFn =
    Arc<dyn Fn(Hash32, u64, u32, Option<Hash32>) -> Option<SignedProposal> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct ProposalResponseMsg {
    pub peer_id: PeerId,
    pub request_id: u64,
    pub genesis: Hash32,
    pub height: u64,
    pub round: u32,
    pub proposal: Box<SignedProposal>,
}

/// Master P2P Network Service managing peer connections, discovery, and gossip.
#[derive(Clone)]
pub struct P2pService {
    local_peer_id: PeerId,
    identity: Arc<NetworkIdentity>,
    chain_id: String,
    genesis: Hash32,
    config: NetworkConfig,
    resource_manager: ResourceManager,
    inbound_slots: Arc<Semaphore>,
    outbound_slots: Arc<Semaphore>,
    peers: Arc<RwLock<HashMap<PeerId, PeerHandle>>>,
    known_addresses: Arc<RwLock<HashSet<String>>>,
    is_running: Arc<AtomicBool>,
    shutdown: watch::Sender<u64>,
    inbound_tx_tx: mpsc::Sender<Transaction>,
    inbound_block_tx: mpsc::Sender<Block>,
    inbound_vote_tx: mpsc::Sender<Vote>,
    inbound_proposal_tx: mpsc::Sender<SignedProposal>,
    inbound_evidence_tx: mpsc::Sender<EquivocationEvidence>,
    block_fetch_fn: BlockFetchFn,
    finalized_tip: Arc<RwLock<(u64, Hash32)>>,
    proposal_fetch_fn: Arc<RwLock<Option<ProposalFetchFn>>>,
    proposal_response_tx: Arc<RwLock<Option<mpsc::Sender<ProposalResponseMsg>>>>,
}

impl std::fmt::Debug for P2pService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("P2pService")
            .field("local_peer_id", &self.local_peer_id)
            .field("chain_id", &self.chain_id)
            .field("genesis", &self.genesis)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl P2pService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        _local_peer_id: PeerId,
        chain_id: String,
        config: NetworkConfig,
        inbound_tx_tx: mpsc::Sender<Transaction>,
        inbound_block_tx: mpsc::Sender<Block>,
        inbound_vote_tx: mpsc::Sender<Vote>,
        inbound_proposal_tx: mpsc::Sender<SignedProposal>,
        inbound_evidence_tx: mpsc::Sender<EquivocationEvidence>,
        block_fetch_fn: BlockFetchFn,
    ) -> Self {
        let identity = Arc::new(NetworkIdentity::generate().unwrap());
        Self::with_identity(
            identity,
            Hash32::ZERO,
            chain_id,
            config,
            inbound_tx_tx,
            inbound_block_tx,
            inbound_vote_tx,
            inbound_proposal_tx,
            inbound_evidence_tx,
            block_fetch_fn,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_identity(
        identity: Arc<NetworkIdentity>,
        genesis: Hash32,
        chain_id: String,
        config: NetworkConfig,
        inbound_tx_tx: mpsc::Sender<Transaction>,
        inbound_block_tx: mpsc::Sender<Block>,
        inbound_vote_tx: mpsc::Sender<Vote>,
        inbound_proposal_tx: mpsc::Sender<SignedProposal>,
        inbound_evidence_tx: mpsc::Sender<EquivocationEvidence>,
        block_fetch_fn: BlockFetchFn,
    ) -> Self {
        let local_peer_id = identity.peer_id().clone();
        let limits = NetworkLimits {
            max_inbound_peers: config.max_inbound_peers,
            max_outbound_peers: config.max_outbound_peers,
            max_pending_handshakes_global: config.max_pending_handshakes,
            max_message_size_bytes: config.max_message_size_bytes,
            max_peer_queue_bytes: config.max_peer_queue_bytes,
            ..Default::default()
        };
        let resource_manager = ResourceManager::new(limits);
        let inbound_slots = Arc::new(Semaphore::new(config.max_inbound_peers.min(1024)));
        let outbound_slots = Arc::new(Semaphore::new(config.max_outbound_peers.min(1024)));
        let mut known = HashSet::new();
        for peer in &config.bootstrap_peers {
            known.insert(peer.clone());
        }

        Self {
            local_peer_id,
            identity,
            chain_id,
            genesis,
            config,
            resource_manager,
            inbound_slots,
            outbound_slots,
            peers: Arc::new(RwLock::new(HashMap::new())),
            known_addresses: Arc::new(RwLock::new(known)),
            is_running: Arc::new(AtomicBool::new(false)),
            shutdown: watch::channel(0).0,
            inbound_tx_tx,
            inbound_block_tx,
            inbound_vote_tx,
            inbound_proposal_tx,
            inbound_evidence_tx,
            block_fetch_fn,
            finalized_tip: Arc::new(RwLock::new((0, Hash32::ZERO))),
            proposal_fetch_fn: Arc::new(RwLock::new(None)),
            proposal_response_tx: Arc::new(RwLock::new(None)),
        }
    }

    #[must_use]
    pub fn identity(&self) -> &NetworkIdentity {
        &self.identity
    }

    #[must_use]
    pub fn genesis(&self) -> &Hash32 {
        &self.genesis
    }

    #[must_use]
    pub fn resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    #[must_use]
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    #[must_use]
    pub fn connected_peers_count(&self) -> usize {
        self.peers.read().len()
    }

    pub fn connected_peers(&self) -> Vec<PeerHandle> {
        self.peers.read().values().cloned().collect()
    }

    #[must_use]
    pub fn known_addresses(&self) -> Vec<String> {
        self.known_addresses.read().iter().cloned().collect()
    }

    pub fn add_known_address(&self, addr: &str) -> bool {
        if addr.len() <= 128 && addr.parse::<SocketAddr>().is_ok() {
            let mut known = self.known_addresses.write();
            if known.len() < 1024 {
                return known.insert(addr.to_string());
            }
        }
        false
    }

    pub fn set_proposal_fetch_fn(&self, f: ProposalFetchFn) {
        *self.proposal_fetch_fn.write() = Some(f);
    }

    pub fn set_proposal_response_sender(&self, tx: mpsc::Sender<ProposalResponseMsg>) {
        *self.proposal_response_tx.write() = Some(tx);
    }

    pub fn broadcast_proposal_request(&self, req: NetworkMessage) {
        let peers = self.peers.read();
        for peer in peers.values() {
            let _ = peer.send(req.clone());
        }
    }

    pub fn send_proposal_request_to_peer(&self, peer_id: &PeerId, req: NetworkMessage) -> bool {
        let peers = self.peers.read();
        if let Some(peer) = peers.get(peer_id) {
            peer.send(req).is_ok()
        } else {
            false
        }
    }

    pub fn broadcast_tx(&self, tx: Transaction) {
        let peers = self.peers.read();
        let msg = NetworkMessage::TxGossip(tx);
        for peer in peers.values() {
            let _ = peer.send(msg.clone());
        }
    }

    pub fn broadcast_block(&self, block: Block) {
        if let Ok(hash) = sprax_crypto::Hasher::block_hash(&block.header) {
            self.update_finalized_tip(block.header.height, hash);
        }
        let peers = self.peers.read();
        let msg = NetworkMessage::BlockGossip(block);
        for peer in peers.values() {
            let _ = peer.send(msg.clone());
        }
    }

    pub fn update_finalized_tip(&self, height: u64, hash: Hash32) {
        let mut tip = self.finalized_tip.write();
        if height > tip.0 || (tip.0 == 0 && tip.1 == Hash32::ZERO) {
            *tip = (height, hash);
        }
    }

    pub fn broadcast_proposal(&self, proposal: SignedProposal) {
        let peers = self.peers.read();
        let msg = NetworkMessage::Proposal(proposal);
        for peer in peers.values() {
            let _ = peer.send(msg.clone());
        }
    }

    pub fn broadcast_vote(&self, vote: Vote) {
        let peers = self.peers.read();
        let msg = NetworkMessage::Vote(vote);
        for peer in peers.values() {
            let _ = peer.send(msg.clone());
        }
    }

    pub fn broadcast_evidence(&self, evidence: EquivocationEvidence) {
        let peers = self.peers.read();
        let msg = NetworkMessage::Evidence(evidence);
        for peer in peers.values() {
            let _ = peer.send(msg.clone());
        }
    }

    pub fn broadcast(&self, msg: NetworkMessage) {
        let peers = self.peers.read();
        for peer in peers.values() {
            let _ = peer.send(msg.clone());
        }
    }

    /// Connects to a target peer over TCP with mutual cryptographic authentication.
    pub async fn dial_peer(
        &self,
        addr_str: &str,
        current_height: u64,
        latest_hash: Hash32,
    ) -> Result<(), NetworkError> {
        let addr = self.resource_manager.validate_socket_addr(addr_str)?;
        let slot = Arc::clone(&self.outbound_slots)
            .try_acquire_owned()
            .map_err(|_| {
                NetworkError::ConnectionFailed("outbound connection limit reached".into())
            })?;

        let timeout_dur = self.resource_manager.limits().handshake_timeout;
        let stream = tokio::time::timeout(timeout_dur, TcpStream::connect(addr))
            .await
            .map_err(|_| {
                NetworkError::ConnectionFailed(format!("connection timed out to {addr_str}"))
            })?
            .map_err(|e| {
                NetworkError::ConnectionFailed(format!("failed to connect to {addr_str}: {e}"))
            })?;

        let listen_addr = if self.config.p2p_port > 0 {
            Some(format!(
                "{}:{}",
                self.config.listen_addr, self.config.p2p_port
            ))
        } else {
            None
        };

        Self::handle_outbound_connection(
            stream,
            Arc::clone(&self.identity),
            self.genesis,
            self.chain_id.clone(),
            current_height,
            latest_hash,
            Arc::clone(&self.peers),
            Arc::clone(&self.known_addresses),
            self.resource_manager.clone(),
            addr_str.to_string(),
            listen_addr,
            self.inbound_tx_tx.clone(),
            self.inbound_block_tx.clone(),
            self.inbound_vote_tx.clone(),
            self.inbound_proposal_tx.clone(),
            self.inbound_evidence_tx.clone(),
            Arc::clone(&self.block_fetch_fn),
            Arc::clone(&self.proposal_fetch_fn),
            Arc::clone(&self.proposal_response_tx),
            self.config.clone(),
            ShutdownSignal::new(&self.shutdown),
            slot,
        )
        .await
    }

    /// Starts the P2P listener daemon in background.
    pub async fn start(
        &self,
        initial_height: u64,
        initial_hash: Hash32,
    ) -> Result<(), NetworkError> {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let listen_addr = format!("{}:{}", self.config.listen_addr, self.config.p2p_port);
        let listener = match TcpListener::bind(&listen_addr).await {
            Ok(listener) => listener,
            Err(error) => {
                self.is_running.store(false, Ordering::SeqCst);
                return Err(NetworkError::ConnectionFailed(format!(
                    "failed to bind TCP listener on {listen_addr}: {error}"
                )));
            }
        };

        info!(
            peer_id = %self.local_peer_id,
            listen_addr = %listen_addr,
            "P2P Network Service listening for authenticated connections"
        );

        let peers = Arc::clone(&self.peers);
        let known_addresses = Arc::clone(&self.known_addresses);
        let identity = Arc::clone(&self.identity);
        let chain_id = self.chain_id.clone();
        let genesis = self.genesis;
        let is_running = Arc::clone(&self.is_running);
        let tx_in = self.inbound_tx_tx.clone();
        let block_in = self.inbound_block_tx.clone();
        let vote_in = self.inbound_vote_tx.clone();
        let proposal_in = self.inbound_proposal_tx.clone();
        let evidence_in = self.inbound_evidence_tx.clone();
        let fetch_fn = Arc::clone(&self.block_fetch_fn);
        let prop_fetch = Arc::clone(&self.proposal_fetch_fn);
        let prop_resp = Arc::clone(&self.proposal_response_tx);
        let inbound_slots = Arc::clone(&self.inbound_slots);
        let resource_manager = self.resource_manager.clone();
        let config = self.config.clone();
        let mut listener_shutdown = ShutdownSignal::new(&self.shutdown);
        let crawler_shutdown = ShutdownSignal::new(&self.shutdown);

        self.update_finalized_tip(initial_height, initial_hash);
        let finalized_tip = Arc::clone(&self.finalized_tip);

        // Spawn Inbound TCP Listener Loop
        tokio::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                let accepted = tokio::select! {
                    biased;
                    _ = wait_for_shutdown(&mut listener_shutdown) => break,
                    result = listener.accept() => result,
                };
                match accepted {
                    Ok((stream, remote_addr)) => {
                        let Ok(slot) = Arc::clone(&inbound_slots).try_acquire_owned() else {
                            drop(stream);
                            continue;
                        };
                        debug!(addr = %remote_addr, "Accepted incoming P2P connection");
                        let p = Arc::clone(&peers);
                        let k = Arc::clone(&known_addresses);
                        let ident = Arc::clone(&identity);
                        let c_id = chain_id.clone();
                        let tx_in = tx_in.clone();
                        let block_in = block_in.clone();
                        let vote_in = vote_in.clone();
                        let proposal_in = proposal_in.clone();
                        let evidence_in = evidence_in.clone();
                        let fetch_fn = Arc::clone(&fetch_fn);
                        let p_fetch = Arc::clone(&prop_fetch);
                        let p_resp = Arc::clone(&prop_resp);
                        let res_mgr = resource_manager.clone();
                        let cfg = config.clone();

                        let connection_shutdown = listener_shutdown.clone();
                        let (current_height, current_hash) = *finalized_tip.read();
                        tokio::spawn(async move {
                            let _connection_slot = slot;
                            let connection = Self::handle_inbound_connection(
                                stream,
                                ident,
                                genesis,
                                c_id,
                                current_height,
                                current_hash,
                                p,
                                k,
                                res_mgr,
                                tx_in,
                                block_in,
                                vote_in,
                                proposal_in,
                                evidence_in,
                                fetch_fn,
                                p_fetch,
                                p_resp,
                                cfg,
                                connection_shutdown,
                            );
                            if let Err(e) = connection.await {
                                debug!("Inbound connection closed: {e}");
                            }
                        });
                    }
                    Err(err) => {
                        warn!("TCP listener accept error: {err}");
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });

        // Dial bootstrap peers in background
        for peer_addr in &self.config.bootstrap_peers {
            let addr = peer_addr.clone();
            let _ = self.dial_peer(&addr, initial_height, initial_hash).await;
        }

        // Spawn dynamic peer discovery crawler loop
        let crawler = self.clone();
        tokio::spawn(async move {
            crawler.run_peer_discovery_loop(crawler_shutdown).await;
        });

        Ok(())
    }

    async fn run_peer_discovery_loop(&self, mut shutdown: ShutdownSignal) {
        let mut interval = tokio::time::interval_at(
            tokio::time::Instant::now() + Duration::from_millis(500),
            Duration::from_millis(500),
        );
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        while self.is_running.load(Ordering::SeqCst) {
            tokio::select! {
                biased;
                _ = wait_for_shutdown(&mut shutdown) => break,
                _ = interval.tick() => {},
            }

            if !self.is_running.load(Ordering::SeqCst) {
                break;
            }

            // Periodically request peer discovery from connected peers
            {
                let connected = self.peers.read();
                for peer in connected.values() {
                    let _ = peer.send(NetworkMessage::PeerDiscoveryRequest);
                }
            }

            // Dial candidate peers if we need more outbound connections
            let connected_count = self.connected_peers_count();
            if connected_count < self.config.max_outbound_peers {
                let needed = self.config.max_outbound_peers - connected_count;
                let candidate_addrs = {
                    let known = self.known_addresses.read();
                    let connected = self.peers.read();
                    let connected_addrs: HashSet<String> =
                        connected.values().map(|p| p.remote_addr.clone()).collect();
                    let my_p2p_port = self.config.p2p_port;

                    known
                        .iter()
                        .filter(|addr| {
                            if connected_addrs.contains(*addr) {
                                return false;
                            }
                            if let Ok(parsed) = addr.parse::<SocketAddr>() {
                                if parsed.port() == my_p2p_port
                                    && (parsed.ip().is_loopback() || parsed.ip().is_unspecified())
                                {
                                    return false;
                                }
                            }
                            true
                        })
                        .take(needed)
                        .cloned()
                        .collect::<Vec<_>>()
                };

                let (height, hash) = *self.finalized_tip.read();
                for addr in candidate_addrs {
                    let _ = self.dial_peer(&addr, height, hash).await;
                }
            }
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.shutdown
            .send_modify(|generation| *generation = generation.wrapping_add(1));
        self.peers.write().clear();
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_inbound_connection(
        mut stream: TcpStream,
        identity: Arc<NetworkIdentity>,
        genesis: Hash32,
        chain_id: String,
        current_height: u64,
        latest_hash: Hash32,
        peers: Arc<RwLock<HashMap<PeerId, PeerHandle>>>,
        known: Arc<RwLock<HashSet<String>>>,
        resource_manager: ResourceManager,
        inbound_tx: mpsc::Sender<Transaction>,
        inbound_block: mpsc::Sender<Block>,
        inbound_vote: mpsc::Sender<Vote>,
        inbound_proposal: mpsc::Sender<SignedProposal>,
        inbound_evidence: mpsc::Sender<EquivocationEvidence>,
        block_fetch_fn: BlockFetchFn,
        proposal_fetch_fn: Arc<RwLock<Option<ProposalFetchFn>>>,
        proposal_response_tx: Arc<RwLock<Option<mpsc::Sender<ProposalResponseMsg>>>>,
        config: NetworkConfig,
        mut shutdown: ShutdownSignal,
    ) -> Result<(), NetworkError> {
        let remote_socket = stream
            .peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_default();
        let remote_ip = stream
            .peer_addr()
            .map(|a| a.ip())
            .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

        let handshake_permit = resource_manager.acquire_handshake_permit(remote_ip)?;
        let timeout_dur = resource_manager.limits().handshake_timeout;
        let local_id = identity.peer_id().clone();

        let handshake_work = async {
            let auth_channel =
                AuthenticatedChannel::respond(&mut stream, &identity, &chain_id, &genesis).await?;

            let remote_id = auth_channel.remote_peer_id.clone();
            if resource_manager.is_peer_banned(&remote_id) {
                return Err(NetworkError::PeerBanned(format!(
                    "peer {remote_id} is banned"
                )));
            }

            let (server_r, server_w) = stream.into_split();
            let (mut encrypted_reader, mut encrypted_writer) =
                auth_channel.split_io(server_r, server_w, config.max_message_size_bytes);

            let remote_msg = encrypted_reader.read_message().await?;
            let (remote_height, remote_hash, listen_addr) = match remote_msg {
                NetworkMessage::Handshake {
                    peer_id: claimed_id,
                    chain_id: peer_chain_id,
                    genesis: peer_genesis,
                    protocol_version,
                    height,
                    latest_block_hash,
                    listen_addr,
                    ..
                } => {
                    if protocol_version != CURRENT_PROTOCOL_VERSION {
                        return Err(NetworkError::ProtocolMismatch(format!(
                            "unsupported protocol version: {protocol_version}"
                        )));
                    }
                    if peer_chain_id != chain_id {
                        return Err(NetworkError::HandshakeFailed(format!(
                            "chain ID mismatch: {peer_chain_id} != {chain_id}"
                        )));
                    }
                    if peer_genesis != genesis
                        && genesis != Hash32::ZERO
                        && peer_genesis != Hash32::ZERO
                    {
                        return Err(NetworkError::HandshakeFailed(format!(
                            "genesis mismatch: {peer_genesis} != {genesis}"
                        )));
                    }
                    if claimed_id != remote_id {
                        return Err(NetworkError::SecurityViolation(format!(
                            "claimed peer ID {claimed_id} does not match authenticated peer ID {remote_id}"
                        )));
                    }

                    // Reject duplicate peer before sending HandshakeAck
                    let preferred_direction = local_id > remote_id;
                    {
                        let connected = peers.read();
                        if let Some(existing) = connected.get(&remote_id) {
                            if existing.preferred_direction || !preferred_direction {
                                return Err(NetworkError::ConnectionFailed(
                                    "duplicate connected peer ID".into(),
                                ));
                            }
                        }
                    }

                    (height, latest_block_hash, listen_addr)
                }
                _ => {
                    return Err(NetworkError::HandshakeFailed(
                        "expected Handshake message".into(),
                    ))
                }
            };

            let ack = NetworkMessage::HandshakeAck {
                peer_id: local_id.clone(),
                chain_id: chain_id.clone(),
                genesis,
                protocol_version: CURRENT_PROTOCOL_VERSION,
                capabilities: 1,
                height: current_height,
                latest_block_hash: latest_hash,
            };
            encrypted_writer.write_message(&ack).await?;

            drop(handshake_permit);

            Ok::<_, NetworkError>((
                remote_id,
                remote_height,
                remote_hash,
                listen_addr,
                encrypted_reader,
                encrypted_writer,
            ))
        };

        let (
            remote_id,
            remote_height,
            remote_hash,
            reported_addr,
            encrypted_reader,
            encrypted_writer,
        ) = tokio::select! {
            biased;
            _ = wait_for_shutdown(&mut shutdown) => return Err(NetworkError::ConnectionFailed("network stopped".into())),
            result = tokio::time::timeout(timeout_dur, handshake_work) => {
                result.map_err(|_| NetworkError::HandshakeFailed("handshake timed out".into()))??
            }
        };

        let preferred_direction = local_id > remote_id;
        let remote_addr = if let Some(reported) = reported_addr {
            if let Ok(parsed) = reported.parse::<SocketAddr>() {
                parsed.to_string()
            } else {
                remote_socket
            }
        } else {
            remote_socket
        };

        Self::run_authenticated_connection_loop(
            encrypted_reader,
            encrypted_writer,
            remote_id,
            remote_addr,
            preferred_direction,
            remote_height,
            remote_hash,
            current_height,
            peers,
            known,
            resource_manager,
            inbound_tx,
            inbound_block,
            inbound_vote,
            inbound_proposal,
            inbound_evidence,
            block_fetch_fn,
            proposal_fetch_fn,
            proposal_response_tx,
            config.max_peer_queue_bytes,
            shutdown,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_outbound_connection(
        mut stream: TcpStream,
        identity: Arc<NetworkIdentity>,
        genesis: Hash32,
        chain_id: String,
        current_height: u64,
        latest_hash: Hash32,
        peers: Arc<RwLock<HashMap<PeerId, PeerHandle>>>,
        known: Arc<RwLock<HashSet<String>>>,
        resource_manager: ResourceManager,
        remote_addr: String,
        local_listen_addr: Option<String>,
        inbound_tx: mpsc::Sender<Transaction>,
        inbound_block: mpsc::Sender<Block>,
        inbound_vote: mpsc::Sender<Vote>,
        inbound_proposal: mpsc::Sender<SignedProposal>,
        inbound_evidence: mpsc::Sender<EquivocationEvidence>,
        block_fetch_fn: BlockFetchFn,
        proposal_fetch_fn: Arc<RwLock<Option<ProposalFetchFn>>>,
        proposal_response_tx: Arc<RwLock<Option<mpsc::Sender<ProposalResponseMsg>>>>,
        config: NetworkConfig,
        mut shutdown: ShutdownSignal,
        slot: OwnedSemaphorePermit,
    ) -> Result<(), NetworkError> {
        let timeout_dur = resource_manager.limits().handshake_timeout;
        let local_id = identity.peer_id().clone();

        let handshake_work = async {
            let auth_channel =
                AuthenticatedChannel::initiate(&mut stream, &identity, &chain_id, &genesis).await?;

            let remote_id = auth_channel.remote_peer_id.clone();
            if resource_manager.is_peer_banned(&remote_id) {
                return Err(NetworkError::PeerBanned(format!(
                    "peer {remote_id} is banned"
                )));
            }

            let (client_r, client_w) = stream.into_split();
            let (mut encrypted_reader, mut encrypted_writer) =
                auth_channel.split_io(client_r, client_w, config.max_message_size_bytes);

            let handshake = NetworkMessage::Handshake {
                peer_id: local_id.clone(),
                chain_id: chain_id.clone(),
                genesis,
                protocol_version: CURRENT_PROTOCOL_VERSION,
                capabilities: 1,
                height: current_height,
                latest_block_hash: latest_hash,
                listen_addr: local_listen_addr,
            };
            encrypted_writer.write_message(&handshake).await?;

            let ack_msg = encrypted_reader.read_message().await?;
            let (remote_height, remote_hash) = match ack_msg {
                NetworkMessage::HandshakeAck {
                    peer_id: claimed_id,
                    chain_id: peer_chain_id,
                    genesis: peer_genesis,
                    protocol_version,
                    height,
                    latest_block_hash,
                    ..
                } => {
                    if protocol_version != CURRENT_PROTOCOL_VERSION {
                        return Err(NetworkError::ProtocolMismatch(format!(
                            "unsupported protocol version: {protocol_version}"
                        )));
                    }
                    if peer_chain_id != chain_id {
                        return Err(NetworkError::HandshakeFailed(format!(
                            "chain ID mismatch: {peer_chain_id} != {chain_id}"
                        )));
                    }
                    if peer_genesis != genesis
                        && genesis != Hash32::ZERO
                        && peer_genesis != Hash32::ZERO
                    {
                        return Err(NetworkError::HandshakeFailed(format!(
                            "genesis mismatch: {peer_genesis} != {genesis}"
                        )));
                    }
                    if claimed_id != remote_id {
                        return Err(NetworkError::SecurityViolation(format!(
                            "claimed peer ID {claimed_id} does not match authenticated peer ID {remote_id}"
                        )));
                    }

                    (height, latest_block_hash)
                }
                _ => {
                    return Err(NetworkError::HandshakeFailed(
                        "expected HandshakeAck message".into(),
                    ))
                }
            };

            Ok::<_, NetworkError>((
                remote_id,
                remote_height,
                remote_hash,
                encrypted_reader,
                encrypted_writer,
            ))
        };

        let (remote_id, remote_height, remote_hash, encrypted_reader, encrypted_writer) = tokio::select! {
            biased;
            _ = wait_for_shutdown(&mut shutdown) => return Err(NetworkError::ConnectionFailed("network stopped".into())),
            result = tokio::time::timeout(timeout_dur, handshake_work) => {
                result.map_err(|_| NetworkError::HandshakeFailed("handshake timed out".into()))??
            }
        };

        let preferred_direction = local_id < remote_id;
        tokio::spawn(async move {
            let _connection_slot = slot;
            Self::run_authenticated_connection_loop(
                encrypted_reader,
                encrypted_writer,
                remote_id,
                remote_addr,
                preferred_direction,
                remote_height,
                remote_hash,
                current_height,
                peers,
                known,
                resource_manager,
                inbound_tx,
                inbound_block,
                inbound_vote,
                inbound_proposal,
                inbound_evidence,
                block_fetch_fn,
                proposal_fetch_fn,
                proposal_response_tx,
                config.max_peer_queue_bytes,
                shutdown,
            )
            .await
        });
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_authenticated_connection_loop<R, W>(
        mut encrypted_reader: EncryptedReader<R>,
        mut encrypted_writer: EncryptedWriter<W>,
        remote_id: PeerId,
        remote_addr: String,
        preferred_direction: bool,
        height: u64,
        hash: Hash32,
        current_height: u64,
        peers: Arc<RwLock<HashMap<PeerId, PeerHandle>>>,
        known_addresses: Arc<RwLock<HashSet<String>>>,
        resource_manager: ResourceManager,
        inbound_tx: mpsc::Sender<Transaction>,
        inbound_block: mpsc::Sender<Block>,
        inbound_vote: mpsc::Sender<Vote>,
        inbound_proposal: mpsc::Sender<SignedProposal>,
        inbound_evidence: mpsc::Sender<EquivocationEvidence>,
        block_fetch_fn: BlockFetchFn,
        proposal_fetch_fn: Arc<RwLock<Option<ProposalFetchFn>>>,
        proposal_response_tx: Arc<RwLock<Option<mpsc::Sender<ProposalResponseMsg>>>>,
        max_peer_queue_bytes: usize,
        mut shutdown: ShutdownSignal,
    ) -> Result<(), NetworkError>
    where
        R: tokio::io::AsyncRead + Unpin + Send + 'static,
        W: tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        let (outbound_tx, mut outbound_rx) = mpsc::channel::<NetworkMessage>(16);
        let byte_budget = Arc::new(OutboundByteBudget::new(max_peer_queue_bytes));

        let (disconnect, mut disconnected) = watch::channel(false);
        let handle = PeerHandle {
            peer_id: remote_id.clone(),
            remote_addr: remote_addr.clone(),
            height,
            latest_block_hash: hash,
            score: PeerScore::default(),
            sender: outbound_tx.clone(),
            byte_budget,
            preferred_direction,
            disconnect,
        };

        // Simultaneous dial collision resolution
        {
            let mut connected = peers.write();
            if let Some(existing) = connected.get(&remote_id) {
                if existing.preferred_direction || !preferred_direction {
                    return Err(NetworkError::ConnectionFailed(
                        "duplicate connected peer ID".into(),
                    ));
                }
                existing.disconnect.send_replace(true);
            }
            connected.insert(remote_id.clone(), handle);
        }
        info!(peer = %remote_id, "Authenticated P2P peer connected successfully");

        // Catch-up on connect if peer is ahead
        if height > current_height {
            let _ = outbound_tx.try_send(NetworkMessage::GetBlocksRequest {
                from_height: current_height + 1,
                to_height: height,
            });
        }

        let p_clone = Arc::clone(&peers);
        let r_id = remote_id.clone();

        // Writer task
        let mut write_task = tokio::spawn(async move {
            while let Some(msg) = outbound_rx.recv().await {
                if encrypted_writer.write_message(&msg).await.is_err() {
                    break;
                }
            }
        });

        // Reader loop
        let res_mgr = resource_manager.clone();
        let read_work = async {
            let mut last_req_sec = Instant::now();
            let mut reqs_this_sec = 0usize;
            const MAX_PROPOSAL_REQS_PER_SEC: usize = 20;

            loop {
                let msg = match encrypted_reader.read_message().await {
                    Ok(msg) => msg,
                    Err(e) => {
                        debug!("Peer {r_id} read error: {e}");
                        let mut p_lock = p_clone.write();
                        if let Some(peer) = p_lock.get_mut(&r_id) {
                            peer.score.record_invalid_message();
                            if peer.score.is_banned {
                                res_mgr.ban_peer(r_id.clone());
                            }
                        }
                        break;
                    }
                };

                match msg {
                    NetworkMessage::TxGossip(tx) => {
                        let _ = inbound_tx.try_send(tx);
                    }
                    NetworkMessage::BlockGossip(block) => {
                        let _ = inbound_block.try_send(block);
                    }
                    NetworkMessage::Ping { nonce } => {
                        let pong = NetworkMessage::Pong { nonce };
                        let _ = p_clone.read().get(&r_id).map(|p| p.send(pong));
                    }
                    NetworkMessage::Vote(vote) => {
                        let _ = inbound_vote.try_send(vote);
                    }
                    NetworkMessage::Proposal(proposal) => {
                        let _ = inbound_proposal.try_send(proposal);
                    }
                    NetworkMessage::Evidence(evidence) => {
                        let _ = inbound_evidence.try_send(evidence);
                    }
                    NetworkMessage::ProposalRequest {
                        request_id,
                        genesis,
                        height,
                        round,
                        block_hash,
                    } => {
                        if last_req_sec.elapsed() >= Duration::from_secs(1) {
                            last_req_sec = Instant::now();
                            reqs_this_sec = 0;
                        }
                        reqs_this_sec += 1;
                        if reqs_this_sec <= MAX_PROPOSAL_REQS_PER_SEC {
                            if let Some(fetcher) = proposal_fetch_fn.read().as_ref() {
                                if let Some(proposal) = fetcher(genesis, height, round, block_hash)
                                {
                                    let resp = NetworkMessage::ProposalResponse {
                                        request_id,
                                        genesis,
                                        height,
                                        round,
                                        proposal: Box::new(proposal),
                                    };
                                    let _ = outbound_tx.try_send(resp);
                                }
                            }
                        } else {
                            warn!(
                                peer = %r_id,
                                "exceeded per-peer proposal request rate limit; dropping request"
                            );
                        }
                    }
                    NetworkMessage::ProposalResponse {
                        request_id,
                        genesis,
                        height,
                        round,
                        proposal,
                    } => {
                        if let Some(tx) = proposal_response_tx.read().as_ref() {
                            let _ = tx.try_send(ProposalResponseMsg {
                                peer_id: r_id.clone(),
                                request_id,
                                genesis,
                                height,
                                round,
                                proposal,
                            });
                        }
                    }
                    NetworkMessage::GetBlocksRequest {
                        from_height,
                        to_height,
                    } => {
                        let blocks = if from_height <= to_height {
                            block_fetch_fn(from_height, from_height)
                        } else {
                            Vec::new()
                        };
                        let _ = outbound_tx.try_send(NetworkMessage::GetBlocksResponse { blocks });
                    }
                    NetworkMessage::GetBlocksResponse { blocks } => {
                        let last_height = blocks.last().map(|block| block.header.height);
                        for b in blocks {
                            if inbound_block.send(b).await.is_err() {
                                break;
                            }
                        }
                        if let Some(last) = last_height.filter(|last| *last < height) {
                            let _ = outbound_tx.try_send(NetworkMessage::GetBlocksRequest {
                                from_height: last + 1,
                                to_height: height,
                            });
                        }
                    }
                    NetworkMessage::PeerDiscoveryRequest => {
                        let mut candidate_peers = Vec::new();
                        {
                            let known = known_addresses.read();
                            for addr in known.iter() {
                                if candidate_peers.len() >= 50 {
                                    break;
                                }
                                if !addr.is_empty() {
                                    candidate_peers.push(addr.clone());
                                }
                            }
                        }
                        if candidate_peers.len() < 50 {
                            let connected = peers.read();
                            for p in connected.values() {
                                if candidate_peers.len() >= 50 {
                                    break;
                                }
                                if !p.remote_addr.is_empty()
                                    && !candidate_peers.contains(&p.remote_addr)
                                {
                                    candidate_peers.push(p.remote_addr.clone());
                                }
                            }
                        }
                        let _ = outbound_tx.try_send(NetworkMessage::PeerDiscoveryResponse {
                            peers: candidate_peers,
                        });
                    }
                    NetworkMessage::PeerDiscoveryResponse { peers: discovered } => {
                        let mut known = known_addresses.write();
                        for addr in discovered {
                            if addr.len() <= 128
                                && addr.parse::<SocketAddr>().is_ok()
                                && known.len() < 1024
                            {
                                known.insert(addr);
                            }
                        }
                    }
                    NetworkMessage::Handshake { .. }
                    | NetworkMessage::HandshakeAck { .. }
                    | NetworkMessage::Pong { .. } => {}
                }
            }
        };

        tokio::select! {
            biased;
            _ = wait_for_shutdown(&mut shutdown) => {},
            _ = disconnected.changed() => {},
            _ = read_work => {},
            _ = &mut write_task => {},
        }

        let mut connected = peers.write();
        if connected
            .get(&remote_id)
            .is_some_and(|peer| peer.sender.same_channel(&outbound_tx))
        {
            connected.remove(&remote_id);
        }
        drop(connected);
        write_task.abort();
        info!(peer = %remote_id, "P2P Peer disconnected");

        Ok(())
    }
}

// Stop generations cannot be erased by a quick stop/start sequence.
#[derive(Clone)]
struct ShutdownSignal {
    receiver: watch::Receiver<u64>,
    generation: u64,
}

impl ShutdownSignal {
    fn new(sender: &watch::Sender<u64>) -> Self {
        let receiver = sender.subscribe();
        let generation = *receiver.borrow();
        Self {
            receiver,
            generation,
        }
    }
}

async fn wait_for_shutdown(shutdown: &mut ShutdownSignal) {
    loop {
        if *shutdown.receiver.borrow_and_update() != shutdown.generation {
            return;
        }
        if shutdown.receiver.changed().await.is_err() {
            return;
        }
    }
}
