use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContractError {
    #[error("Unauthorized caller: {0}")]
    Unauthorized(String),

    #[error("Contract not found at address: {0}")]
    ContractNotFound(String),

    #[error("Code ID not found: {0}")]
    CodeNotFound(u64),

    #[error("Insufficient funds: balance {balance}, required {required}")]
    InsufficientFunds { balance: String, required: String },

    #[error("Out of gas: limit {limit}, consumed {consumed}")]
    OutOfGas { limit: u64, consumed: u64 },

    #[error("Reentrancy detected: contract {0} already active on call stack")]
    ReentrancyDetected(String),

    #[error("Arithmetic overflow/underflow: {0}")]
    ArithmeticError(String),

    #[error("Invalid contract input: {0}")]
    InvalidInput(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Contract state storage error: {0}")]
    StorageError(String),
}
