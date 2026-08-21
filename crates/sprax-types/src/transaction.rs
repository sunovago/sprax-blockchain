use crate::{address::Address, amount::Amount, chain_id::ChainId, error::TypeError, hash::Hash32};
use serde::{Deserialize, Serialize};

/// Type of signature algorithm used by the transaction signer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    Ed25519,
    Secp256k1,
}

/// Transaction message payload variants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxMessage {
    /// Native asset transfer
    Transfer { to: Address, amount: Amount },
    /// Staking delegation to a validator
    Delegate { validator: Address, amount: Amount },
    /// Staking unbonding from a validator
    Unbond { validator: Address, amount: Amount },
    /// Smart contract execution payload
    ContractCall { contract: Address, data: Vec<u8> },
    /// Generic extensible payload
    Generic { type_url: String, payload: Vec<u8> },
}

/// Fee and gas specification for transaction execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxFee {
    /// Total fee offered (in atto-SPRX)
    pub amount: Amount,
    /// Maximum gas limit authorized
    pub gas_limit: u64,
    /// Priority fee tip (in atto-SPRX)
    pub priority_fee: Amount,
}

impl Default for TxFee {
    fn default() -> Self {
        Self {
            amount: Amount::from_atto(500_000_000_000_000), // 0.0005 SPRX
            gas_limit: 200_000,
            priority_fee: Amount::ZERO,
        }
    }
}

/// Core transaction body containing operations and metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxBody {
    pub chain_id: ChainId,
    pub sender: Address,
    pub nonce: u64,
    pub messages: Vec<TxMessage>,
    pub fee: TxFee,
    pub memo: String,
    pub timeout_height: u64,
}

impl TxBody {
    /// Serializes the transaction body to canonical JSON bytes for signing.
    pub fn sign_bytes(&self) -> Result<Vec<u8>, TypeError> {
        serde_json::to_vec(self)
            .map_err(|e| TypeError::InvalidTransaction(format!("serialization error: {e}")))
    }
}

/// Complete signed transaction envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub body: TxBody,
    pub key_type: KeyType,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}

impl Transaction {
    pub fn new(
        body: TxBody,
        key_type: KeyType,
        public_key: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<Self, TypeError> {
        if body.messages.is_empty() {
            return Err(TypeError::InvalidTransaction(
                "transaction must contain at least one message".into(),
            ));
        }
        if public_key.is_empty() {
            return Err(TypeError::InvalidTransaction(
                "public key cannot be empty".into(),
            ));
        }
        if signature.is_empty() {
            return Err(TypeError::InvalidTransaction(
                "signature cannot be empty".into(),
            ));
        }
        Ok(Self {
            body,
            key_type,
            public_key,
            signature,
        })
    }

    /// Serializes the transaction body to canonical JSON bytes for signature creation/verification.
    pub fn sign_bytes(&self) -> Result<Vec<u8>, TypeError> {
        serde_json::to_vec(&self.body)
            .map_err(|e| TypeError::InvalidTransaction(format!("serialization error: {e}")))
    }
}

/// Transaction receipt generated after state execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxReceipt {
    pub tx_hash: Hash32,
    pub height: u64,
    pub success: bool,
    pub gas_used: u64,
    pub error_message: Option<String>,
    pub return_data: Vec<u8>,
}
