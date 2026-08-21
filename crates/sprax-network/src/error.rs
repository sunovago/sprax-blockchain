use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Invalid peer ID: {0}")]
    InvalidPeerId(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),

    #[error("Peer banned due to low score: {0}")]
    PeerBanned(String),

    #[error("Topic error: {0}")]
    TopicError(String),

    #[error("Message serialization error: {0}")]
    SerializationError(String),

    #[error("Rate limit exceeded for peer: {0}")]
    RateLimitExceeded(String),
}
