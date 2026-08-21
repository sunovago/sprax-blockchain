use serde::{Deserialize, Serialize};
use sprax_types::{Address, Amount, Hash32};

pub type CodeId = u64;

/// Stored bytecode metadata and hash commitment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeInfo {
    pub code_id: CodeId,
    pub creator: Address,
    pub code_hash: Hash32,
    pub code_size: usize,
    pub created_height: u64,
}

/// On-chain instantiated contract instance metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractInfo {
    pub address: Address,
    pub code_id: CodeId,
    pub admin: Option<Address>,
    pub label: String,
    pub creator: Address,
    pub created_height: u64,
}

/// Execution environment contextual parameters passed to contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Env {
    pub block_height: u64,
    pub block_timestamp_unix: u64,
    pub chain_id: String,
    pub contract_address: Address,
}

/// Transaction caller message info.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageInfo {
    pub sender: Address,
    pub funds: Amount,
}

/// Typed event emitted during contract execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractEvent {
    pub event_type: String,
    pub attributes: Vec<(String, String)>,
}

impl ContractEvent {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            attributes: Vec::new(),
        }
    }

    pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push((key.into(), value.into()));
        self
    }
}

/// Sub-message for cross-contract calling or native fund transfers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractSubMessage {
    BankSend {
        to: Address,
        amount: Amount,
    },
    ExecuteContract {
        contract: Address,
        msg: Vec<u8>,
        funds: Amount,
    },
}

/// Result returned from contract instantiation or execution.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractResponse {
    pub data: Option<Vec<u8>>,
    pub events: Vec<ContractEvent>,
    pub messages: Vec<ContractSubMessage>,
}

impl ContractResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_event(mut self, event: ContractEvent) -> Self {
        self.events.push(event);
        self
    }

    pub fn add_message(mut self, msg: ContractSubMessage) -> Self {
        self.messages.push(msg);
        self
    }

    pub fn set_data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }
}
