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

/// Active validator set with deterministic weighted proposer selection.
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
        let mut addresses = std::collections::HashSet::new();
        let mut total_voting_power = 0u64;
        for validator in &validators {
            if validator.voting_power == 0 || !addresses.insert(validator.address) {
                return Err(ConsensusError::InvalidValidatorSet(
                    "zero power or duplicate validator".into(),
                ));
            }
            total_voting_power = total_voting_power
                .checked_add(validator.voting_power)
                .filter(|power| *power <= i64::MAX as u64 / 4)
                .ok_or_else(|| {
                    ConsensusError::InvalidValidatorSet(
                        "voting power exceeds safe proposer arithmetic bound".into(),
                    )
                })?;
        }
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

    pub fn from_canonical(
        validators: Vec<sprax_types::CanonicalValidator>,
    ) -> Result<Self, ConsensusError> {
        Self::new(
            validators
                .into_iter()
                .map(|v| Validator::new(v.address, v.public_key, v.voting_power))
                .collect(),
        )
    }

    pub fn commitment(&self) -> Result<sprax_types::Hash32, ConsensusError> {
        let validators: Vec<_> = self
            .validators
            .iter()
            .map(|v| sprax_types::CanonicalValidator {
                address: v.address,
                public_key: v.public_key.clone(),
                voting_power: v.voting_power,
            })
            .collect();
        let encoded = serde_json::to_vec(&validators)
            .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?;
        Ok(sprax_crypto::Hasher::sha256(&encoded))
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
        ((u128::from(self.total_voting_power) * 2 / 3) + 1) as u64
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

    /// Selects a weighted proposer from immutable consensus coordinates. Every node,
    /// including a node returning after downtime, derives the same validator without
    /// relying on process-local priority state. Rejection sampling avoids modulo bias.
    pub fn select_proposer_for_round(
        &self,
        genesis: sprax_types::Hash32,
        height: u64,
        round: u32,
    ) -> Result<Validator, ConsensusError> {
        let commitment = self.commitment()?;
        let mut seed = Vec::with_capacity(128);
        seed.extend_from_slice(b"sprax/proposer/v1");
        seed.extend_from_slice(genesis.as_bytes());
        seed.extend_from_slice(&height.to_be_bytes());
        seed.extend_from_slice(&round.to_be_bytes());
        seed.extend_from_slice(commitment.as_bytes());

        let total = u128::from(self.total_voting_power);
        let limit = u128::MAX - u128::from(u128::MAX % total);
        let mut counter = 0u32;
        let ticket = loop {
            seed.extend_from_slice(&counter.to_be_bytes());
            let hash = sprax_crypto::Hasher::sha256(&seed);
            seed.truncate(seed.len() - 4);
            let candidate = u128::from_be_bytes(
                hash.as_bytes()[..16]
                    .try_into()
                    .expect("fixed-size hash prefix"),
            );
            if candidate < limit {
                break candidate % total;
            }
            counter = counter.checked_add(1).ok_or_else(|| {
                ConsensusError::InvalidValidatorSet("proposer draw counter exhausted".into())
            })?;
        };

        let mut cumulative = 0u128;
        for validator in &self.validators {
            cumulative += u128::from(validator.voting_power);
            if ticket < cumulative {
                return Ok(validator.clone());
            }
        }
        Err(ConsensusError::InvalidValidatorSet(
            "proposer ticket falls outside total voting power".into(),
        ))
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
