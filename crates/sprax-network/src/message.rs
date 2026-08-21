use crate::peer::PeerId;
use serde::{Deserialize, Serialize};
use sprax_consensus::{EquivocationEvidence, Vote};
use sprax_types::{Block, Hash32, Transaction};

/// Canonical P2P Network Protocol Messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Initial handshake sent upon establishing a TCP connection.
    Handshake {
        peer_id: PeerId,
        chain_id: String,
        height: u64,
        latest_block_hash: Hash32,
        listen_addr: Option<String>,
    },
    /// Handshake acknowledgment responding with local peer metadata.
    HandshakeAck {
        peer_id: PeerId,
        chain_id: String,
        height: u64,
        latest_block_hash: Hash32,
    },
    /// Heartbeat ping.
    Ping { nonce: u64 },
    /// Heartbeat pong reply.
    Pong { nonce: u64 },
    /// Gossip of a new uncommitted transaction.
    TxGossip(Transaction),
    /// Gossip of a newly produced and committed block.
    BlockGossip(Block),
    /// Request a range of historical blocks for state synchronization.
    GetBlocksRequest { from_height: u64, to_height: u64 },
    /// Response delivering a batch of blocks for catch-up synchronization.
    GetBlocksResponse { blocks: Vec<Block> },
    /// Request known active peer addresses.
    PeerDiscoveryRequest,
    /// Response containing active peer network addresses.
    PeerDiscoveryResponse { peers: Vec<String> },
    /// Block proposal broadcast by the round's selected proposer.
    Proposal {
        height: u64,
        round: u32,
        block: Block,
    },
    /// A signed BFT prevote or precommit attestation.
    Vote(Vote),
    /// Double-sign (equivocation) evidence, gossiped so every honest node can slash locally.
    Evidence(EquivocationEvidence),
}

impl NetworkMessage {
    /// Encodes the message to JSON bytes with length-prefix framing.
    pub fn encode(&self) -> Result<Vec<u8>, crate::error::NetworkError> {
        let json_bytes = serde_json::to_vec(self)
            .map_err(|e| crate::error::NetworkError::SerializationError(e.to_string()))?;
        let len = (json_bytes.len() as u32).to_be_bytes();
        let mut framed = Vec::with_capacity(4 + json_bytes.len());
        framed.extend_from_slice(&len);
        framed.extend_from_slice(&json_bytes);
        Ok(framed)
    }

    /// Decodes a message from raw payload bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, crate::error::NetworkError> {
        serde_json::from_slice(bytes)
            .map_err(|e| crate::error::NetworkError::SerializationError(e.to_string()))
    }
}
