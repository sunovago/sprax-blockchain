use thiserror::Error;

/// Errors arising from core type validation and conversions.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum TypeError {
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),

    #[error("Invalid hash length: expected {expected} bytes, found {found}")]
    InvalidHashLength { expected: usize, found: usize },

    #[error("Hex decoding error: {0}")]
    HexDecodeError(String),

    #[error("Invalid chain ID format: '{0}' (expected lowercase alphanumeric with hyphens, e.g. sprax-mainnet-1)")]
    InvalidChainId(String),

    #[error("Arithmetic overflow in amount operation: {0}")]
    AmountOverflow(String),

    #[error("Arithmetic underflow in amount operation: {0}")]
    AmountUnderflow(String),

    #[error("Invalid amount format: {0}")]
    InvalidAmount(String),

    #[error("Invalid denomination string: '{0}'")]
    InvalidDenomination(String),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Invalid block structure: {0}")]
    InvalidBlock(String),
}
