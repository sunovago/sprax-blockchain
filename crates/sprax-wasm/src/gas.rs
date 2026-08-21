use crate::error::ContractError;

/// Precise metered gas counter for contract operations.
#[derive(Debug, Clone)]
pub struct GasMeter {
    limit: u64,
    consumed: u64,
}

impl GasMeter {
    pub const BASE_INVOCATION_GAS: u64 = 2_000;
    pub const STORAGE_READ_BASE_GAS: u64 = 100;
    pub const STORAGE_READ_BYTE_GAS: u64 = 1;
    pub const STORAGE_WRITE_BASE_GAS: u64 = 500;
    pub const STORAGE_WRITE_BYTE_GAS: u64 = 2;

    pub fn new(limit: u64) -> Self {
        Self { limit, consumed: 0 }
    }

    #[must_use]
    pub fn limit(&self) -> u64 {
        self.limit
    }

    #[must_use]
    pub fn consumed(&self) -> u64 {
        self.consumed
    }

    #[must_use]
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.consumed)
    }

    /// Consumes gas, returning an error if limit is exceeded.
    pub fn consume(&mut self, amount: u64) -> Result<(), ContractError> {
        let new_consumed = self.consumed.saturating_add(amount);
        if new_consumed > self.limit {
            return Err(ContractError::OutOfGas {
                limit: self.limit,
                consumed: new_consumed,
            });
        }
        self.consumed = new_consumed;
        Ok(())
    }

    /// Consumes storage read gas based on key and value lengths.
    pub fn consume_read(&mut self, bytes_len: usize) -> Result<(), ContractError> {
        let cost = Self::STORAGE_READ_BASE_GAS + (bytes_len as u64 * Self::STORAGE_READ_BYTE_GAS);
        self.consume(cost)
    }

    /// Consumes storage write gas based on key and value lengths.
    pub fn consume_write(&mut self, bytes_len: usize) -> Result<(), ContractError> {
        let cost = Self::STORAGE_WRITE_BASE_GAS + (bytes_len as u64 * Self::STORAGE_WRITE_BYTE_GAS);
        self.consume(cost)
    }
}
