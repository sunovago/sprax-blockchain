use crate::error::CoreError;
use serde::{Deserialize, Serialize};

/// Deterministic gas costs for VM and storage operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GasConfig {
    pub base_tx_cost: u64,
    pub signature_verify_cost: u64,
    pub storage_read_byte_cost: u64,
    pub storage_write_byte_cost: u64,
    pub transfer_cost: u64,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            base_tx_cost: 21_000,
            signature_verify_cost: 10_000,
            storage_read_byte_cost: 10,
            storage_write_byte_cost: 100,
            transfer_cost: 10_000,
        }
    }
}

/// Dynamic Gas Meter tracking consumption within transaction bounds.
#[derive(Debug, Clone)]
pub struct GasMeter {
    limit: u64,
    consumed: u64,
}

impl GasMeter {
    #[must_use]
    pub const fn new(limit: u64) -> Self {
        Self { limit, consumed: 0 }
    }

    #[must_use]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    #[must_use]
    pub const fn consumed(&self) -> u64 {
        self.consumed
    }

    #[must_use]
    pub const fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.consumed)
    }

    pub fn consume_gas(&mut self, amount: u64) -> Result<(), CoreError> {
        let new_consumed = self.consumed.saturating_add(amount);
        if new_consumed > self.limit {
            return Err(CoreError::OutOfGas {
                limit: self.limit,
                consumed: new_consumed,
            });
        }
        self.consumed = new_consumed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_meter() {
        let mut meter = GasMeter::new(100_000);
        assert_eq!(meter.remaining(), 100_000);

        meter.consume_gas(30_000).unwrap();
        assert_eq!(meter.consumed(), 30_000);
        assert_eq!(meter.remaining(), 70_000);

        meter.consume_gas(70_000).unwrap();
        assert_eq!(meter.consumed(), 100_000);

        assert!(meter.consume_gas(1).is_err());
    }
}
