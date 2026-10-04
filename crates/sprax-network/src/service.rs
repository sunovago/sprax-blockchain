use crate::{
    config::NetworkConfig,
    error::NetworkError,
    message::NetworkMessage,
    peer::{PeerId, PeerScore},
};
use parking_lot::RwLock;
use sprax_consensus::{EquivocationEvidence, SignedProposal, Vote};
use sprax_types::{Block, Hash32, Transaction};
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
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
    sender: mpsc::Sender<NetworkMessage>,
}

impl PeerHandle {
    pub fn send(&self, msg: NetworkMessage) -> Result<(), NetworkError> {
        self.sender
            .try_send(msg)
            .map_err(|e| NetworkError::ConnectionFailed(format!("failed to send message: {e}")))
    }
}

pub type BlockFetchFn = Arc<dyn Fn(u64, u64) -> Vec<Block> + Send + Sync>;

/// Master P2P Network Service managing peer connections, discovery, and gossip.
#[derive(Clone)]
pub struct P2pService {
    local_peer_id: PeerId,
    chain_id: String,
    config: NetworkConfig,
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
}

impl std::fmt::Debug for P2pService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("P2pService")
            .field("local_peer_id", &self.local_peer_id)
            .field("chain_id", &self.chain_id)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl P2pService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        local_peer_id: PeerId,
        chain_id: String,
        config: NetworkConfig,
        inbound_tx_tx: mpsc::Sender<Transaction>,
        inbound_block_tx: mpsc::Sender<Block>,
        inbound_vote_tx: mpsc::Sender<Vote>,
        inbound_proposal_tx: mpsc::Sender<SignedProposal>,
        inbound_evidence_tx: mpsc::Sender<EquivocationEvidence>,
        block_fetch_fn: BlockFetchFn,
    ) -> Self {
        let inbound_slots = Arc::new(Semaphore::new(config.max_inbound_peers.min(1024)));
        let outbound_slots = Arc::new(Semaphore::new(config.max_outbound_peers.min(1024)));
        let mut known = HashSet::new();
        for peer in &config.bootstrap_peers {
            known.insert(peer.clone());
        }

        Self {
            local_peer_id,
            chain_id,
            config,
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
        }
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

    /// Broadcasts a newly submitted transaction to all connected peers.
    pub fn broadcast_tx(&self, tx: Transaction) {
        let peers = self.peers.read();
        let msg = NetworkMessage::TxGossip(tx);
        for peer in peers.values() {
            let _ = peer.send(msg.clone());
        }
    }

    /// Broadcasts a newly produced/committed block to all connected peers.
    pub fn broadcast_block(&self, block: Block) {
        let peers = self.peers.read();
        let msg = NetworkMessage::BlockGossip(block);
        for peer in peers.values() {
            let _ = peer.send(msg.clone());
        }
    }

    /// Broadcasts a block proposal for the given (height, round) to all connected peers.
    pub fn broadcast_proposal(&self, proposal: SignedProposal) {
        let peers = self.peers.read();
        let msg = NetworkMessage::Proposal(proposal);
        for peer in peers.values() {
            let _ = peer.send(msg.clone());
        }
    }

    /// Broadcasts a signed BFT vote (prevote or precommit) to all connected peers.
    pub fn broadcast_vote(&self, vote: Vote) {
        let peers = self.peers.read();
        let msg = NetworkMessage::Vote(vote);
        for peer in peers.values() {
            let _ = peer.send(msg.clone());
        }
    }

    /// Broadcasts double-sign evidence so every honest peer can slash the offending validator locally.
    pub fn broadcast_evidence(&self, evidence: EquivocationEvidence) {
        let peers = self.peers.read();
        let msg = NetworkMessage::Evidence(evidence);
        for peer in peers.values() {
            let _ = peer.send(msg.clone());
        }
    }

    /// Connects to a target peer over TCP.
    pub async fn dial_peer(
        &self,
        addr_str: &str,
        current_height: u64,
        latest_hash: Hash32,
    ) -> Result<(), NetworkError> {
        let slot = Arc::clone(&self.outbound_slots)
            .try_acquire_owned()
            .map_err(|_| {
                NetworkError::ConnectionFailed("outbound connection limit reached".into())
            })?;
        let addr: SocketAddr = addr_str.parse().map_err(|e| {
            NetworkError::InvalidAddress(format!("invalid socket address '{addr_str}': {e}"))
        })?;

        let stream = tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(addr))
            .await
            .map_err(|_| {
                NetworkError::ConnectionFailed(format!("connection timed out to {addr_str}"))
            })?
            .map_err(|e| {
                NetworkError::ConnectionFailed(format!("failed to connect to {addr_str}: {e}"))
            })?;

        Self::handle_outbound_connection(
            stream,
            self.local_peer_id.clone(),
            self.chain_id.clone(),
            current_height,
            latest_hash,
            Arc::clone(&self.peers),
            Arc::clone(&self.known_addresses),
            self.inbound_tx_tx.clone(),
            self.inbound_block_tx.clone(),
            self.inbound_vote_tx.clone(),
            self.inbound_proposal_tx.clone(),
            self.inbound_evidence_tx.clone(),
            Arc::clone(&self.block_fetch_fn),
            self.config.max_message_size_bytes,
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
            "P2P Network Service listening for incoming connections"
        );

        let peers = Arc::clone(&self.peers);
        let known_addresses = Arc::clone(&self.known_addresses);
        let local_peer_id = self.local_peer_id.clone();
        let chain_id = self.chain_id.clone();
        let is_running = Arc::clone(&self.is_running);
        let tx_in = self.inbound_tx_tx.clone();
        let block_in = self.inbound_block_tx.clone();
        let vote_in = self.inbound_vote_tx.clone();
        let proposal_in = self.inbound_proposal_tx.clone();
        let evidence_in = self.inbound_evidence_tx.clone();
        let fetch_fn = Arc::clone(&self.block_fetch_fn);
        let inbound_slots = Arc::clone(&self.inbound_slots);
        let max_message_size = self.config.max_message_size_bytes;
        let mut shutdown = ShutdownSignal::new(&self.shutdown);

        // Spawn Inbound TCP Listener Loop
        tokio::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                let accepted = tokio::select! {
                    biased;
                    _ = wait_for_shutdown(&mut shutdown) => break,
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
                        let l_id = local_peer_id.clone();
                        let c_id = chain_id.clone();
                        let tx_in = tx_in.clone();
                        let block_in = block_in.clone();
                        let vote_in = vote_in.clone();
                        let proposal_in = proposal_in.clone();
                        let evidence_in = evidence_in.clone();
                        let fetch_fn = Arc::clone(&fetch_fn);

                        let connection_shutdown = shutdown.clone();
                        tokio::spawn(async move {
                            let _connection_slot = slot;
                            let connection = Self::handle_inbound_connection(
                                stream,
                                l_id,
                                c_id,
                                initial_height,
                                initial_hash,
                                p,
                                k,
                                tx_in,
                                block_in,
                                vote_in,
                                proposal_in,
                                evidence_in,
                                fetch_fn,
                                max_message_size,
                                connection_shutdown.clone(),
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

        Ok(())
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
        local_id: PeerId,
        chain_id: String,
        current_height: u64,
        latest_hash: Hash32,
        peers: Arc<RwLock<HashMap<PeerId, PeerHandle>>>,
        known: Arc<RwLock<HashSet<String>>>,
        inbound_tx: mpsc::Sender<Transaction>,
        inbound_block: mpsc::Sender<Block>,
        inbound_vote: mpsc::Sender<Vote>,
        inbound_proposal: mpsc::Sender<SignedProposal>,
        inbound_evidence: mpsc::Sender<EquivocationEvidence>,
        block_fetch_fn: BlockFetchFn,
        max_message_size: usize,
        mut shutdown: ShutdownSignal,
    ) -> Result<(), NetworkError> {
        let handshake_work = async {
            let remote_msg =
                tokio::time::timeout(Duration::from_secs(5), Self::read_frame(&mut stream))
                    .await
                    .map_err(|_| NetworkError::HandshakeFailed("handshake timed out".into()))??;
            let (remote_id, remote_height, remote_hash) = match remote_msg {
                NetworkMessage::Handshake {
                    peer_id,
                    chain_id: peer_chain_id,
                    height,
                    latest_block_hash,
                    listen_addr,
                } => {
                    if peer_chain_id != chain_id {
                        return Err(NetworkError::HandshakeFailed(format!(
                            "chain ID mismatch: {peer_chain_id} != {chain_id}"
                        )));
                    }
                    if let Some(addr) = listen_addr {
                        if addr.len() <= 128 && addr.parse::<SocketAddr>().is_ok() {
                            let mut known = known.write();
                            if known.len() < 1024 {
                                known.insert(addr);
                            }
                        }
                    }
                    (peer_id, height, latest_block_hash)
                }
                _ => {
                    return Err(NetworkError::HandshakeFailed(
                        "expected Handshake message".into(),
                    ))
                }
            };

            // Send HandshakeAck
            let ack = NetworkMessage::HandshakeAck {
                peer_id: local_id.clone(),
                chain_id: chain_id.clone(),
                height: current_height,
                latest_block_hash: latest_hash,
            };
            Self::write_frame(&mut stream, &ack).await?;
            Ok::<_, NetworkError>((remote_id, remote_height, remote_hash))
        };
        let (remote_id, remote_height, remote_hash) = tokio::select! {
            biased;
            _ = wait_for_shutdown(&mut shutdown) => return Err(NetworkError::ConnectionFailed("network stopped".into())),
            result = handshake_work => result?,
        };

        Self::run_connection_loop(
            stream,
            remote_id,
            remote_height,
            remote_hash,
            current_height,
            peers,
            inbound_tx,
            inbound_block,
            inbound_vote,
            inbound_proposal,
            inbound_evidence,
            block_fetch_fn,
            max_message_size,
            shutdown,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_outbound_connection(
        mut stream: TcpStream,
        local_id: PeerId,
        chain_id: String,
        current_height: u64,
        latest_hash: Hash32,
        peers: Arc<RwLock<HashMap<PeerId, PeerHandle>>>,
        _known: Arc<RwLock<HashSet<String>>>,
        inbound_tx: mpsc::Sender<Transaction>,
        inbound_block: mpsc::Sender<Block>,
        inbound_vote: mpsc::Sender<Vote>,
        inbound_proposal: mpsc::Sender<SignedProposal>,
        inbound_evidence: mpsc::Sender<EquivocationEvidence>,
        block_fetch_fn: BlockFetchFn,
        max_message_size: usize,
        mut shutdown: ShutdownSignal,
        slot: OwnedSemaphorePermit,
    ) -> Result<(), NetworkError> {
        let handshake_work = async {
            // Send Handshake
            let handshake = NetworkMessage::Handshake {
                peer_id: local_id,
                chain_id: chain_id.clone(),
                height: current_height,
                latest_block_hash: latest_hash,
                listen_addr: None,
            };
            Self::write_frame(&mut stream, &handshake).await?;

            // Read HandshakeAck (bounded — an unresponsive peer must not hang the dialer forever)
            let ack_msg =
                tokio::time::timeout(Duration::from_secs(5), Self::read_frame(&mut stream))
                    .await
                    .map_err(|_| {
                        NetworkError::HandshakeFailed("handshake ack timed out".into())
                    })??;
            let (remote_id, remote_height, remote_hash) = match ack_msg {
                NetworkMessage::HandshakeAck {
                    peer_id,
                    chain_id: peer_chain_id,
                    height,
                    latest_block_hash,
                } => {
                    if peer_chain_id != chain_id {
                        return Err(NetworkError::HandshakeFailed(format!(
                            "chain ID mismatch: {peer_chain_id} != {chain_id}"
                        )));
                    }
                    (peer_id, height, latest_block_hash)
                }
                _ => {
                    return Err(NetworkError::HandshakeFailed(
                        "expected HandshakeAck message".into(),
                    ))
                }
            };

            Ok::<_, NetworkError>((remote_id, remote_height, remote_hash))
        };
        let (remote_id, remote_height, remote_hash) = tokio::select! {
            biased;
            _ = wait_for_shutdown(&mut shutdown) => return Err(NetworkError::ConnectionFailed("network stopped".into())),
            result = handshake_work => result?,
        };

        tokio::spawn(async move {
            let _connection_slot = slot;
            Self::run_connection_loop(
                stream,
                remote_id,
                remote_height,
                remote_hash,
                current_height,
                peers,
                inbound_tx,
                inbound_block,
                inbound_vote,
                inbound_proposal,
                inbound_evidence,
                block_fetch_fn,
                max_message_size,
                shutdown,
            )
            .await
        });
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_connection_loop(
        stream: TcpStream,
        remote_id: PeerId,
        height: u64,
        hash: Hash32,
        current_height: u64,
        peers: Arc<RwLock<HashMap<PeerId, PeerHandle>>>,
        inbound_tx: mpsc::Sender<Transaction>,
        inbound_block: mpsc::Sender<Block>,
        inbound_vote: mpsc::Sender<Vote>,
        inbound_proposal: mpsc::Sender<SignedProposal>,
        inbound_evidence: mpsc::Sender<EquivocationEvidence>,
        block_fetch_fn: BlockFetchFn,
        max_message_size: usize,
        mut shutdown: ShutdownSignal,
    ) -> Result<(), NetworkError> {
        let (mut reader, mut writer) = stream.into_split();
        let (outbound_tx, mut outbound_rx) = mpsc::channel::<NetworkMessage>(8);

        let handle = PeerHandle {
            peer_id: remote_id.clone(),
            remote_addr: "".to_string(),
            height,
            latest_block_hash: hash,
            score: PeerScore::default(),
            sender: outbound_tx.clone(),
        };

        // Do not let a duplicate self-asserted ID replace the live connection's routing entry.
        {
            let mut connected = peers.write();
            if connected.contains_key(&remote_id) {
                return Err(NetworkError::ConnectionFailed(
                    "duplicate connected peer ID".into(),
                ));
            }
            connected.insert(remote_id.clone(), handle);
        }
        info!(peer = %remote_id, "P2P Peer connected successfully");

        // Catch-up on connect: if this peer is ahead of us, ask for the blocks we're missing.
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
                if let Ok(framed) = msg.encode_bounded(max_message_size) {
                    if writer.write_all(&framed).await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        // Reader loop
        let read_work = async {
            loop {
                let mut len_bytes = [0u8; 4];
                if reader.read_exact(&mut len_bytes).await.is_err() {
                    break;
                }
                let len = u32::from_be_bytes(len_bytes) as usize;
                if len == 0 || len > max_message_size {
                    break; // Message too large
                }
                let mut buf = vec![0u8; len];
                if reader.read_exact(&mut buf).await.is_err() {
                    break;
                }
                if let Ok(msg) = NetworkMessage::decode(&buf) {
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
                        NetworkMessage::GetBlocksRequest {
                            from_height,
                            to_height,
                        } => {
                            // Serve one block per page; an untrusted range cannot allocate the archive.
                            let blocks = if from_height <= to_height {
                                block_fetch_fn(from_height, from_height)
                            } else {
                                Vec::new()
                            };
                            let _ =
                                outbound_tx.try_send(NetworkMessage::GetBlocksResponse { blocks });
                        }
                        NetworkMessage::GetBlocksResponse { blocks } => {
                            let last_height = blocks.last().map(|block| block.header.height);
                            for b in blocks {
                                // Apply backpressure so catch-up never drops a required predecessor.
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
                            let known_peers: Vec<String> = peers
                                .read()
                                .values()
                                .map(|p| p.remote_addr.to_string())
                                .collect();
                            let _ = outbound_tx.try_send(NetworkMessage::PeerDiscoveryResponse {
                                peers: known_peers,
                            });
                        }
                        NetworkMessage::PeerDiscoveryResponse { peers: discovered } => {
                            info!(count = discovered.len(), "Received peer discovery list");
                        }
                        NetworkMessage::Handshake { .. }
                        | NetworkMessage::HandshakeAck { .. }
                        | NetworkMessage::Pong { .. } => {}
                    }
                }
            }
        };
        tokio::select! {
            biased;
            _ = wait_for_shutdown(&mut shutdown) => {},
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

    async fn write_frame(stream: &mut TcpStream, msg: &NetworkMessage) -> Result<(), NetworkError> {
        let framed = msg.encode()?;
        stream
            .write_all(&framed)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
        Ok(())
    }

    async fn read_frame(stream: &mut TcpStream) -> Result<NetworkMessage, NetworkError> {
        let mut len_bytes = [0u8; 4];
        stream
            .read_exact(&mut len_bytes)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        if len == 0 || len > 4096 {
            return Err(NetworkError::HandshakeFailed(
                "handshake frame exceeds the 4 KiB limit".into(),
            ));
        }
        let mut buf = vec![0u8; len];
        stream
            .read_exact(&mut buf)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
        NetworkMessage::decode(&buf)
    }
}

// Stop generations cannot be erased by a quick stop/start sequence. Every old task
// keeps its original generation even if it first polls after the next start.
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
