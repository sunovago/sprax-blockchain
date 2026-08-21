use crate::error::ConsensusError;
use serde::{Deserialize, Serialize};
use sprax_types::Address;

/// Active consensus validator descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    pub address: Address,
    pub public_key: Vec<u8>,
    pub voting_power: u64,
    pub proposer_priority: i64,
}

impl Validator {
    pub fn new(address: Address, public_key: Vec<u8>, voting_power: u64) -> Self {
        Self {
            address,
            public_key,
            voting_power,
            proposer_priority: 0,
        }
    }
}

/// Active validator set with deterministic weighted round-robin proposer selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorSet {
    validators: Vec<Validator>,
    total_voting_power: u64,
}

impl ValidatorSet {
    pub fn new(validators: Vec<Validator>) -> Result<Self, ConsensusError> {
        if validators.is_empty() {
            return Err(ConsensusError::InvalidValidatorSet(
                "validator set cannot be empty".into(),
            ));
        }
        let total_voting_power: u64 = validators.iter().map(|v| v.voting_power).sum();
        if total_voting_power == 0 {
            return Err(ConsensusError::InvalidValidatorSet(
                "total voting power cannot be zero".into(),
            ));
        }

        let mut set = Self {
            validators,
            total_voting_power,
        };
        // Normalize and sort lexicographically by address for deterministic behavior
        set.validators.sort_by_key(|a| a.address);
        Ok(set)
    }

    #[must_use]
    pub fn total_voting_power(&self) -> u64 {
        self.total_voting_power
    }

    #[must_use]
    pub fn validators(&self) -> &[Validator] {
        &self.validators
    }

    /// Mutable access to the validator list — used by [`crate::BftConsensusEngine::set_validator_set`]
    /// to carry `proposer_priority` forward across a refresh (see its docs for why).
    pub fn validators_mut(&mut self) -> &mut [Validator] {
        &mut self.validators
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.validators.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    /// Consensus Quorum Threshold: Q = floor(2W/3) + 1.
    #[must_use]
    pub fn quorum_threshold(&self) -> u64 {
        (self.total_voting_power * 2 / 3) + 1
    }

    /// Checks if a given voting power sum satisfies consensus quorum.
    #[must_use]
    pub fn has_quorum(&self, voting_power: u64) -> bool {
        voting_power >= self.quorum_threshold()
    }

    /// Selects the next block proposer using Deterministic Weighted Round-Robin (DWRR).
    pub fn select_proposer(&mut self) -> Validator {
        let total_power = self.total_voting_power as i64;

        // 1. Increment priority of each validator by their voting power
        for val in &mut self.validators {
            val.proposer_priority = val
                .proposer_priority
                .saturating_add(val.voting_power as i64);
        }

        // 2. Select validator with maximum priority (tie-break by sorted address)
        let max_idx = self
            .validators
            .iter()
            .enumerate()
            .max_by_key(|(_, val)| val.proposer_priority)
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        // 3. Decrement selected validator's priority by total voting power
        self.validators[max_idx].proposer_priority = self.validators[max_idx]
            .proposer_priority
            .saturating_sub(total_power);

        self.validators[max_idx].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_set_quorum() {
        let v1 = Validator::new(Address::new([1u8; 20]), vec![1; 32], 50);
        let v2 = Validator::new(Address::new([2u8; 20]), vec![2; 32], 30);
        let v3 = Validator::new(Address::new([3u8; 20]), vec![3; 32], 20);

        let set = ValidatorSet::new(vec![v1, v2, v3]).unwrap();
        assert_eq!(set.total_voting_power(), 100);
        assert_eq!(set.quorum_threshold(), 67);

        assert!(!set.has_quorum(66));
        assert!(set.has_quorum(67));
        assert!(set.has_quorum(100));
    }

    #[test]
    fn test_deterministic_weighted_proposer_selection() {
        let v1 = Validator::new(Address::new([1u8; 20]), vec![1; 32], 60);
        let v2 = Validator::new(Address::new([2u8; 20]), vec![2; 32], 40);

        let mut set = ValidatorSet::new(vec![v1.clone(), v2.clone()]).unwrap();

        let mut p_counts = std::collections::HashMap::new();
        for _ in 0..10 {
            let proposer = set.select_proposer();
            *p_counts.entry(proposer.address).or_insert(0) += 1;
        }

        assert_eq!(p_counts.get(&v1.address), Some(&6));
        assert_eq!(p_counts.get(&v2.address), Some(&4));
    }
}
