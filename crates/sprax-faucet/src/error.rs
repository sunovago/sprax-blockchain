use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FaucetError {
    #[error("Rate limit exceeded for address/IP: retry in {retry_after_secs} seconds")]
    RateLimitExceeded { retry_after_secs: u64 },

    #[error("Requested amount ({requested}) exceeds maximum allowed payout ({max_allowed})")]
    AmountExceedsLimit {
        requested: String,
        max_allowed: String,
    },

    #[error("Invalid recipient address: {0}")]
    InvalidAddress(String),

    #[error("Insufficient faucet pool balance")]
    InsufficientFunds,

    #[error("Transaction execution/submission failed: {0}")]
    SubmissionError(String),
}
