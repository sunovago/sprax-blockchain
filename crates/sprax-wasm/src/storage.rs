use crate::{error::ContractError, gas::GasMeter};
use sprax_types::Address;
use std::collections::HashMap;

/// Sandboxed contract-scoped Key-Value storage layer.
#[derive(Debug, Clone, Default)]
pub struct ContractStorage {
    // contract_address -> (key -> value)
    state: HashMap<Address, HashMap<Vec<u8>, Vec<u8>>>,
}

impl ContractStorage {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads a key from contract storage with metered gas consumption.
    pub fn get(
        &self,
        contract: &Address,
        key: &[u8],
        gas: &mut GasMeter,
    ) -> Result<Option<Vec<u8>>, ContractError> {
        let val = self
            .state
            .get(contract)
            .and_then(|map| map.get(key).cloned());

        let len = key.len() + val.as_ref().map(|v| v.len()).unwrap_or(0);
        gas.consume_read(len)?;

        Ok(val)
    }

    /// Writes a key-value pair to contract storage with metered gas consumption.
    pub fn set(
        &mut self,
        contract: &Address,
        key: Vec<u8>,
        value: Vec<u8>,
        gas: &mut GasMeter,
    ) -> Result<(), ContractError> {
        let len = key.len() + value.len();
        gas.consume_write(len)?;

        self.state.entry(*contract).or_default().insert(key, value);
        Ok(())
    }

    /// Removes a key from contract storage.
    pub fn remove(
        &mut self,
        contract: &Address,
        key: &[u8],
        gas: &mut GasMeter,
    ) -> Result<(), ContractError> {
        gas.consume(GasMeter::STORAGE_WRITE_BASE_GAS)?;
        if let Some(map) = self.state.get_mut(contract) {
            map.remove(key);
        }
        Ok(())
    }

    /// Read-only get without gas metering for external state queries.
    pub fn get_raw(&self, contract: &Address, key: &[u8]) -> Option<Vec<u8>> {
        self.state
            .get(contract)
            .and_then(|map| map.get(key).cloned())
    }

    /// Read-only set without gas for internal tests / genesis seeding.
    pub fn set_raw(&mut self, contract: Address, key: Vec<u8>, value: Vec<u8>) {
        self.state.entry(contract).or_default().insert(key, value);
    }
}
