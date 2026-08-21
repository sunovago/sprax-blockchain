use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Key not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("State commitment mismatch: expected {expected}, computed {computed}")]
    CommitmentMismatch { expected: String, computed: String },

    #[error("Batch write failed: {0}")]
    BatchError(String),

    #[error("Transaction poisoned or lock error: {0}")]
    LockError(String),
}
