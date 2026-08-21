use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("Transaction not found: {0}")]
    TxNotFound(String),

    #[error("Address not found: {0}")]
    AddressNotFound(String),

    #[error("Validator not found: {0}")]
    ValidatorNotFound(String),

    #[error("State inconsistency detected: {0}")]
    StateInconsistency(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid query parameter: {0}")]
    InvalidQuery(String),
}
