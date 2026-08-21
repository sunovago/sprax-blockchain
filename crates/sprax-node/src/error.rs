use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Storage initialization error: {0}")]
    StorageError(String),

    #[error("Service runtime error: {0}")]
    RuntimeError(String),

    #[error("Node shutdown failed: {0}")]
    ShutdownError(String),
}
