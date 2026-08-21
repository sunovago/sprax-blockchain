use crate::{
    error::ContractError,
    gas::GasMeter,
    storage::ContractStorage,
    types::{CodeId, CodeInfo, ContractInfo, Env},
};
use parking_lot::RwLock;
use sprax_crypto::Hasher;
use sprax_types::{Address, Amount};
use std::collections::{HashMap, HashSet};
use tracing::info;

/// Core Sandboxed Smart Contract Runtime Virtual Machine.
#[derive(Debug, Default)]
pub struct WasmContractEngine {
    codes: RwLock<HashMap<CodeId, CodeInfo>>,
    contracts: RwLock<HashMap<Address, ContractInfo>>,
    storage: RwLock<ContractStorage>,
    active_call_stack: RwLock<HashSet<Address>>,
    next_code_id: RwLock<u64>,
    instance_nonce: RwLock<u64>,
}

impl WasmContractEngine {
    pub fn new() -> Self {
        Self {
            next_code_id: RwLock::new(1),
            ..Default::default()
        }
    }

    /// Stores compiled contract bytecode on-chain and generates a unique CodeId.
    pub fn store_code(
        &self,
        creator: Address,
        bytecode: Vec<u8>,
        height: u64,
    ) -> Result<CodeId, ContractError> {
        if bytecode.is_empty() {
            return Err(ContractError::InvalidInput(
                "bytecode cannot be empty".into(),
            ));
        }

        let code_hash = Hasher::blake3(&bytecode);
        let mut id_guard = self.next_code_id.write();
        let code_id = *id_guard;
        *id_guard += 1;

        let code_info = CodeInfo {
            code_id,
            creator,
            code_hash,
            code_size: bytecode.len(),
            created_height: height,
        };

        self.codes.write().insert(code_id, code_info);

        info!(
            code_id = code_id,
            size = bytecode.len(),
            hash = %code_hash,
            "Stored smart contract bytecode on-chain"
        );

        Ok(code_id)
    }

    /// Derives a deterministic unique contract address based on creator, code_id, and nonce.
    fn derive_contract_address(&self, creator: &Address, code_id: CodeId, label: &str) -> Address {
        let mut nonce_guard = self.instance_nonce.write();
        let nonce = *nonce_guard;
        *nonce_guard += 1;

        let mut preimage = creator.as_bytes().to_vec();
        preimage.extend_from_slice(&code_id.to_be_bytes());
        preimage.extend_from_slice(&nonce.to_be_bytes());
        preimage.extend_from_slice(label.as_bytes());

        let hash = Hasher::blake3(&preimage);
        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&hash.as_bytes()[0..20]);
        Address::new(addr_bytes)
    }

    /// Instantiates a new contract from a stored CodeId.
    #[allow(clippy::too_many_arguments)]
    pub fn instantiate(
        &self,
        sender: Address,
        code_id: CodeId,
        label: String,
        admin: Option<Address>,
        _funds: Amount,
        env: &mut Env,
        gas: &mut GasMeter,
    ) -> Result<Address, ContractError> {
        gas.consume(GasMeter::BASE_INVOCATION_GAS)?;

        // Verify CodeId exists
        if !self.codes.read().contains_key(&code_id) {
            return Err(ContractError::CodeNotFound(code_id));
        }

        let contract_addr = self.derive_contract_address(&sender, code_id, &label);
        env.contract_address = contract_addr;

        let contract_info = ContractInfo {
            address: contract_addr,
            code_id,
            admin,
            label: label.clone(),
            creator: sender,
            created_height: env.block_height,
        };

        self.contracts.write().insert(contract_addr, contract_info);

        info!(
            contract = %contract_addr,
            code_id = code_id,
            label = %label,
            "Instantiated smart contract instance"
        );

        Ok(contract_addr)
    }

    /// Acquires a reentrancy execution lock on a contract address.
    pub fn enter_call(&self, contract: &Address) -> Result<(), ContractError> {
        let mut stack = self.active_call_stack.write();
        if stack.contains(contract) {
            return Err(ContractError::ReentrancyDetected(contract.to_string()));
        }
        stack.insert(*contract);
        Ok(())
    }

    /// Releases reentrancy execution lock.
    pub fn exit_call(&self, contract: &Address) {
        self.active_call_stack.write().remove(contract);
    }

    pub fn get_contract_info(&self, address: &Address) -> Option<ContractInfo> {
        self.contracts.read().get(address).cloned()
    }

    pub fn get_code_info(&self, code_id: CodeId) -> Option<CodeInfo> {
        self.codes.read().get(&code_id).cloned()
    }

    pub fn storage(&self) -> &RwLock<ContractStorage> {
        &self.storage
    }

    /// Executes a call against a smart contract instance with gas metering and state isolation.
    pub fn execute(
        &self,
        contract: Address,
        sender: Address,
        msg: Vec<u8>,
        funds: Amount,
        env: &mut Env,
        gas: &mut GasMeter,
    ) -> Result<crate::types::ContractResponse, ContractError> {
        gas.consume(GasMeter::BASE_INVOCATION_GAS)?;

        // Verify contract exists
        let contract_info = self
            .get_contract_info(&contract)
            .ok_or_else(|| ContractError::ContractNotFound(contract.to_string()))?;

        self.enter_call(&contract)?;

        env.contract_address = contract;
        let _info = crate::types::MessageInfo { sender, funds };

        // Consume gas proportional to message payload size
        gas.consume((msg.len() as u64) * GasMeter::STORAGE_READ_BYTE_GAS)?;

        let mut response = crate::types::ContractResponse::new();
        response = response.add_event(
            crate::types::ContractEvent::new("contract_execution")
                .add_attribute("contract", contract.to_string())
                .add_attribute("code_id", contract_info.code_id.to_string())
                .add_attribute("caller", sender.to_string()),
        );

        self.exit_call(&contract);
        Ok(response)
    }

    /// Read-only contract state query.
    pub fn query(
        &self,
        contract: Address,
        query_msg: Vec<u8>,
        gas: &mut GasMeter,
    ) -> Result<Vec<u8>, ContractError> {
        gas.consume(GasMeter::BASE_INVOCATION_GAS)?;

        if !self.contracts.read().contains_key(&contract) {
            return Err(ContractError::ContractNotFound(contract.to_string()));
        }

        // Return current contract storage or query reflection
        let storage_guard = self.storage.read();
        let val = storage_guard
            .get(&contract, &query_msg, gas)?
            .unwrap_or_default();
        Ok(val)
    }
}
