use crate::{error::NetworkError, message::NetworkMessage, peer::PeerId};
use parking_lot::RwLock;
use sprax_types::{Block, Transaction};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use tokio::sync::mpsc;
use tracing::info;

/// Virtual In-Memory Node Endpoint within a Testnet Swarm.
#[derive(Debug, Clone)]
pub struct SwarmNodeEndpoint {
    pub peer_id: PeerId,
    pub tx_sender: mpsc::UnboundedSender<NetworkMessage>,
    pub inbound_tx_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<Transaction>>>,
    pub inbound_block_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<Block>>>,
}

/// Swarm Hub managing a local multi-node virtual testnet mesh.
#[derive(Debug, Default)]
pub struct SwarmHub {
    nodes: Arc<RwLock<HashMap<PeerId, mpsc::UnboundedSender<NetworkMessage>>>>,
    partitions: Arc<RwLock<HashSet<(PeerId, PeerId)>>>,
    delayed_peers: Arc<RwLock<HashSet<PeerId>>>,
}

impl SwarmHub {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new node in the swarm mesh and returns its endpoint channels.
    pub fn register_node(&self, peer_id: PeerId) -> SwarmNodeEndpoint {
        let (node_in_tx, mut node_in_rx) = mpsc::unbounded_channel::<NetworkMessage>();
        let (inbound_tx_tx, inbound_tx_rx) = mpsc::unbounded_channel::<Transaction>();
        let (inbound_block_tx, inbound_block_rx) = mpsc::unbounded_channel::<Block>();

        self.nodes
            .write()
            .insert(peer_id.clone(), node_in_tx.clone());

        // Spawn packet processing dispatcher
        let p_id = peer_id.clone();
        tokio::spawn(async move {
            while let Some(msg) = node_in_rx.recv().await {
                match msg {
                    NetworkMessage::TxGossip(tx) => {
                        let _ = inbound_tx_tx.send(tx);
                    }
                    NetworkMessage::BlockGossip(block) => {
                        let _ = inbound_block_tx.send(block);
                    }
                    _ => {}
                }
            }
            info!(node = %p_id, "Swarm endpoint dispatcher terminated");
        });

        SwarmNodeEndpoint {
            peer_id,
            tx_sender: node_in_tx,
            inbound_tx_rx: Arc::new(tokio::sync::Mutex::new(inbound_tx_rx)),
            inbound_block_rx: Arc::new(tokio::sync::Mutex::new(inbound_block_rx)),
        }
    }

    /// Unregisters or disconnects a node from the swarm (simulating node shutdown).
    pub fn unregister_node(&self, peer_id: &PeerId) {
        self.nodes.write().remove(peer_id);
    }

    /// Creates a network partition blocking communication between two peers.
    pub fn partition_peers(&self, p1: PeerId, p2: PeerId) {
        let mut parts = self.partitions.write();
        parts.insert((p1.clone(), p2.clone()));
        parts.insert((p2, p1));
    }

    /// Heals a partition allowing communication between two peers again.
    pub fn heal_partition(&self, p1: &PeerId, p2: &PeerId) {
        let mut parts = self.partitions.write();
        parts.remove(&(p1.clone(), p2.clone()));
        parts.remove(&(p2.clone(), p1.clone()));
    }

    /// Sets artificial delay on messages destined to a peer.
    pub fn set_delay(&self, peer: PeerId) {
        self.delayed_peers.write().insert(peer);
    }

    /// Removes artificial delay on messages.
    pub fn clear_delay(&self, peer: &PeerId) {
        self.delayed_peers.write().remove(peer);
    }

    /// Broadcasts a network message from sender to all other swarm nodes, respecting partitions and delays.
    pub async fn broadcast(
        &self,
        from: &PeerId,
        msg: NetworkMessage,
    ) -> Result<usize, NetworkError> {
        let nodes = self.nodes.read().clone();
        let partitions = self.partitions.read().clone();
        let delayed = self.delayed_peers.read().clone();

        let mut delivered = 0;
        for (target_id, sender) in nodes {
            if target_id == *from {
                continue;
            }
            // Check if partitioned
            if partitions.contains(&(from.clone(), target_id.clone())) {
                continue; // Dropped by network partition
            }

            let msg_clone = msg.clone();
            let is_delayed = delayed.contains(&target_id);

            if is_delayed {
                let s = sender.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    let _ = s.send(msg_clone);
                });
            } else {
                let _ = sender.send(msg_clone);
            }
            delivered += 1;
        }

        Ok(delivered)
    }
}
