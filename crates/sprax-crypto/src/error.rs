use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Key derivation error: {0}")]
    KeyDerivationError(String),

    #[error("Invalid mnemonic phrase: {0}")]
    InvalidMnemonic(String),

    #[error("Randomness generation error: {0}")]
    RngError(String),
}
