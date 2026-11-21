use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Invalid peer ID: {0}")]
    InvalidPeerId(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

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

    #[error("Network identity error: {0}")]
    IdentityError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Security violation: {0}")]
    SecurityViolation(String),

    #[error("Resource budget exceeded: {0}")]
    BudgetExceeded(String),

    #[error("Protocol mismatch: {0}")]
    ProtocolMismatch(String),

    #[error("Noise transport error: {0}")]
    NoiseError(String),

    #[error("Frame encoding/decoding error: {0}")]
    FrameError(String),
}
