use crate::peer::PeerId;
use serde::{Deserialize, Serialize};
use sprax_consensus::{EquivocationEvidence, SignedProposal, Vote};
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
    Proposal(SignedProposal),
    /// A signed BFT prevote or precommit attestation.
    Vote(Vote),
    /// Double-sign (equivocation) evidence, gossiped so every honest node can slash locally.
    Evidence(EquivocationEvidence),
}

impl NetworkMessage {
    /// Encodes the message to JSON bytes with length-prefix framing.
    pub fn encode(&self) -> Result<Vec<u8>, crate::error::NetworkError> {
        self.encode_bounded(u32::MAX as usize)
    }

    /// Bounds serialization itself, so an oversized outgoing message cannot allocate its
    /// entire JSON representation before the transport checks its length.
    pub fn encode_bounded(&self, limit: usize) -> Result<Vec<u8>, crate::error::NetworkError> {
        struct Payload {
            bytes: Vec<u8>,
            limit: usize,
        }
        impl std::io::Write for Payload {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if bytes.len() > self.limit.saturating_sub(self.bytes.len() - 4) {
                    return Err(std::io::Error::other(
                        "network payload exceeds configured limit",
                    ));
                }
                self.bytes.extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut payload = Payload {
            bytes: vec![0; 4],
            limit: limit.min(u32::MAX as usize),
        };
        serde_json::to_writer(&mut payload, self)
            .map_err(|e| crate::error::NetworkError::SerializationError(e.to_string()))?;
        let len = (payload.bytes.len() - 4) as u32;
        payload.bytes[..4].copy_from_slice(&len.to_be_bytes());
        Ok(payload.bytes)
    }

    /// Decodes a message from raw payload bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, crate::error::NetworkError> {
        serde_json::from_slice(bytes)
            .map_err(|e| crate::error::NetworkError::SerializationError(e.to_string()))
    }
}
