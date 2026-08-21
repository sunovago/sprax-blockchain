use crate::{
    config::NetworkConfig,
    error::NetworkError,
    message::NetworkMessage,
    peer::{PeerId, PeerScore},
};
use parking_lot::RwLock;
use sprax_consensus::{EquivocationEvidence, Vote};
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
    sync::mpsc,
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
    sender: mpsc::UnboundedSender<NetworkMessage>,
}

impl PeerHandle {
    pub fn send(&self, msg: NetworkMessage) -> Result<(), NetworkError> {
        self.sender
            .send(msg)
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
    peers: Arc<RwLock<HashMap<PeerId, PeerHandle>>>,
    known_addresses: Arc<RwLock<HashSet<String>>>,
    is_running: Arc<AtomicBool>,
    inbound_tx_tx: mpsc::UnboundedSender<Transaction>,
    inbound_block_tx: mpsc::UnboundedSender<Block>,
    inbound_vote_tx: mpsc::UnboundedSender<Vote>,
    inbound_proposal_tx: mpsc::UnboundedSender<(u64, u32, Block)>,
    inbound_evidence_tx: mpsc::UnboundedSender<EquivocationEvidence>,
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
        inbound_tx_tx: mpsc::UnboundedSender<Transaction>,
        inbound_block_tx: mpsc::UnboundedSender<Block>,
        inbound_vote_tx: mpsc::UnboundedSender<Vote>,
        inbound_proposal_tx: mpsc::UnboundedSender<(u64, u32, Block)>,
        inbound_evidence_tx: mpsc::UnboundedSender<EquivocationEvidence>,
        block_fetch_fn: BlockFetchFn,
    ) -> Self {
        let mut known = HashSet::new();
        for peer in &config.bootstrap_peers {
            known.insert(peer.clone());
        }

        Self {
            local_peer_id,
            chain_id,
            config,
            peers: Arc::new(RwLock::new(HashMap::new())),
            known_addresses: Arc::new(RwLock::new(known)),
            is_running: Arc::new(AtomicBool::new(false)),
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
    pub fn broadcast_proposal(&self, height: u64, round: u32, block: Block) {
        let peers = self.peers.read();
        let msg = NetworkMessage::Proposal {
            height,
            round,
            block,
        };
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
        let listener = TcpListener::bind(&listen_addr).await.map_err(|e| {
            NetworkError::ConnectionFailed(format!(
                "failed to bind TCP listener on {listen_addr}: {e}"
            ))
        })?;

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

        // Spawn Inbound TCP Listener Loop
        tokio::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                match listener.accept().await {
                    Ok((stream, remote_addr)) => {
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

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_inbound_connection(
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
                            )
                            .await
                            {
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
        inbound_tx: mpsc::UnboundedSender<Transaction>,
        inbound_block: mpsc::UnboundedSender<Block>,
        inbound_vote: mpsc::UnboundedSender<Vote>,
        inbound_proposal: mpsc::UnboundedSender<(u64, u32, Block)>,
        inbound_evidence: mpsc::UnboundedSender<EquivocationEvidence>,
        block_fetch_fn: BlockFetchFn,
    ) -> Result<(), NetworkError> {
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
                    known.write().insert(addr);
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
        inbound_tx: mpsc::UnboundedSender<Transaction>,
        inbound_block: mpsc::UnboundedSender<Block>,
        inbound_vote: mpsc::UnboundedSender<Vote>,
        inbound_proposal: mpsc::UnboundedSender<(u64, u32, Block)>,
        inbound_evidence: mpsc::UnboundedSender<EquivocationEvidence>,
        block_fetch_fn: BlockFetchFn,
    ) -> Result<(), NetworkError> {
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
        let ack_msg = tokio::time::timeout(Duration::from_secs(5), Self::read_frame(&mut stream))
            .await
            .map_err(|_| NetworkError::HandshakeFailed("handshake ack timed out".into()))??;
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

        tokio::spawn(Self::run_connection_loop(
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
        ));
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
        inbound_tx: mpsc::UnboundedSender<Transaction>,
        inbound_block: mpsc::UnboundedSender<Block>,
        inbound_vote: mpsc::UnboundedSender<Vote>,
        inbound_proposal: mpsc::UnboundedSender<(u64, u32, Block)>,
        inbound_evidence: mpsc::UnboundedSender<EquivocationEvidence>,
        block_fetch_fn: BlockFetchFn,
    ) -> Result<(), NetworkError> {
        let (mut reader, mut writer) = stream.into_split();
        let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<NetworkMessage>();

        let handle = PeerHandle {
            peer_id: remote_id.clone(),
            remote_addr: "".to_string(),
            height,
            latest_block_hash: hash,
            score: PeerScore::default(),
            sender: outbound_tx.clone(),
        };

        peers.write().insert(remote_id.clone(), handle);
        info!(peer = %remote_id, "P2P Peer connected successfully");

        // Catch-up on connect: if this peer is ahead of us, ask for the blocks we're missing.
        if height > current_height {
            let _ = outbound_tx.send(NetworkMessage::GetBlocksRequest {
                from_height: current_height + 1,
                to_height: height,
            });
        }

        let p_clone = Arc::clone(&peers);
        let r_id = remote_id.clone();

        // Writer task
        let write_task = tokio::spawn(async move {
            while let Some(msg) = outbound_rx.recv().await {
                if let Ok(framed) = msg.encode() {
                    if writer.write_all(&framed).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Reader loop
        let read_res = async {
            loop {
                let mut len_bytes = [0u8; 4];
                if reader.read_exact(&mut len_bytes).await.is_err() {
                    break;
                }
                let len = u32::from_be_bytes(len_bytes) as usize;
                if len > 10 * 1024 * 1024 {
                    break; // Message too large
                }
                let mut buf = vec![0u8; len];
                if reader.read_exact(&mut buf).await.is_err() {
                    break;
                }
                if let Ok(msg) = NetworkMessage::decode(&buf) {
                    match msg {
                        NetworkMessage::TxGossip(tx) => {
                            let _ = inbound_tx.send(tx);
                        }
                        NetworkMessage::BlockGossip(block) => {
                            let _ = inbound_block.send(block);
                        }
                        NetworkMessage::Ping { nonce } => {
                            let pong = NetworkMessage::Pong { nonce };
                            let _ = p_clone.read().get(&r_id).map(|p| p.send(pong));
                        }
                        NetworkMessage::Vote(vote) => {
                            let _ = inbound_vote.send(vote);
                        }
                        NetworkMessage::Proposal {
                            height,
                            round,
                            block,
                        } => {
                            let _ = inbound_proposal.send((height, round, block));
                        }
                        NetworkMessage::Evidence(evidence) => {
                            let _ = inbound_evidence.send(evidence);
                        }
                        NetworkMessage::GetBlocksRequest {
                            from_height,
                            to_height,
                        } => {
                            let blocks = block_fetch_fn(from_height, to_height);
                            let _ = outbound_tx.send(NetworkMessage::GetBlocksResponse { blocks });
                        }
                        NetworkMessage::GetBlocksResponse { blocks } => {
                            for b in blocks {
                                let _ = inbound_block.send(b);
                            }
                        }
                        NetworkMessage::PeerDiscoveryRequest => {
                            let known_peers: Vec<String> = peers
                                .read()
                                .values()
                                .map(|p| p.remote_addr.to_string())
                                .collect();
                            let _ = outbound_tx
                                .send(NetworkMessage::PeerDiscoveryResponse { peers: known_peers });
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
        }
        .await;

        peers.write().remove(&remote_id);
        write_task.abort();
        info!(peer = %remote_id, "P2P Peer disconnected");

        Ok(read_res)
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
        let mut buf = vec![0u8; len];
        stream
            .read_exact(&mut buf)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
        NetworkMessage::decode(&buf)
    }
}
