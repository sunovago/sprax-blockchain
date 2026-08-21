use crate::error::NetworkError;
use serde::{Deserialize, Serialize};
use sprax_types::Hash32;
use std::fmt;

/// Operational role of a network peer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    FullNode,
    Sentry,
    Validator,
    Archive,
    LightClient,
}

/// Cryptographic Peer Identifier derived from public key.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PeerId(pub String);

impl PeerId {
    pub fn new(id: impl Into<String>) -> Result<Self, NetworkError> {
        let s = id.into();
        if s.is_empty() || s.len() > 128 {
            return Err(NetworkError::InvalidPeerId(
                "peer ID must be between 1 and 128 characters".into(),
            ));
        }
        Ok(Self(s))
    }

    #[must_use]
    pub fn from_pubkey_hash(hash: &Hash32) -> Self {
        Self(format!("12D3KooW{}", hex::encode(&hash.as_bytes()[0..16])))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PeerId({})", self.0)
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Reputation and scoring profile of a connected peer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeerScore {
    pub score: f64,
    pub successful_deliveries: u64,
    pub invalid_messages: u64,
    pub is_banned: bool,
}

impl Default for PeerScore {
    fn default() -> Self {
        Self {
            score: 0.0,
            successful_deliveries: 0,
            invalid_messages: 0,
            is_banned: false,
        }
    }
}

impl PeerScore {
    pub fn record_valid_delivery(&mut self) {
        self.successful_deliveries += 1;
        self.score += 1.0;
    }

    pub fn record_invalid_message(&mut self) {
        self.invalid_messages += 1;
        self.score -= 50.0;
        if self.score < -100.0 {
            self.is_banned = true;
        }
    }
}
