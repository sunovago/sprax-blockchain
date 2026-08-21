use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Out of gas: limit {limit}, consumed {consumed}")]
    OutOfGas { limit: u64, consumed: u64 },

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Insufficient funds: balance {balance}, required {required}")]
    InsufficientFunds { balance: String, required: String },

    #[error("Invalid nonce: account nonce {account_nonce}, tx nonce {tx_nonce}")]
    InvalidNonce { account_nonce: u64, tx_nonce: u64 },

    #[error("Module error ({module}): {reason}")]
    ModuleError { module: String, reason: String },

    #[error("State error: {0}")]
    StateError(String),

    #[error("Execution reverted: {0}")]
    ExecutionReverted(String),
}
