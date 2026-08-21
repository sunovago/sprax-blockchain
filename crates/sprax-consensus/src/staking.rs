use crate::{
    error::ConsensusError,
    evidence::EquivocationEvidence,
    validator::{Validator, ValidatorSet},
};
use serde::{Deserialize, Serialize};
use sprax_types::{Address, Amount};
use std::collections::HashMap;

/// Lifecycle status of a validator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorStatus {
    /// Actively participating in BFT consensus and receiving block rewards.
    Active,
    /// Temporarily or permanently excluded from consensus due to downtime or equivocation.
    Jailed,
    /// Undergoing unbonding period; tokens cannot yet be withdrawn.
    Unbonding,
    /// Inactive and fully unbonded.
    Unbonded,
}

/// On-chain validator metadata and identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorDescription {
    pub moniker: String,
    pub identity: String,
    pub website: String,
    pub details: String,
}

/// Validator commission rate configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommissionRates {
    pub rate: f64,            // e.g. 0.05 (5%)
    pub max_rate: f64,        // e.g. 0.20 (20%)
    pub max_change_rate: f64, // e.g. 0.01 (1% per update)
}

impl Default for CommissionRates {
    fn default() -> Self {
        Self {
            rate: 0.05,
            max_rate: 0.20,
            max_change_rate: 0.01,
        }
    }
}

/// Full on-chain validator staking state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StakingValidator {
    pub operator_address: Address,
    pub consensus_pubkey: Vec<u8>,
    pub description: ValidatorDescription,
    pub status: ValidatorStatus,
    pub tokens: Amount,
    pub delegator_shares: Amount,
    pub min_self_delegation: Amount,
    pub unbonding_height: Option<u64>,
    pub commission: CommissionRates,
    pub jailed_until: Option<u64>,
    pub missed_blocks_count: u64,
    pub is_tombstoned: bool,
}

/// Delegation relationship between an account and a validator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delegation {
    pub delegator_address: Address,
    pub validator_address: Address,
    pub shares: Amount,
    pub balance: Amount,
}

/// Maturing unbonding entry in queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnbondingEntry {
    pub delegator_address: Address,
    pub validator_address: Address,
    pub creation_height: u64,
    pub completion_height: u64,
    pub amount: Amount,
}

/// Consensus & Staking Parameter configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StakingParams {
    pub max_validators: usize,
    pub unbonding_period_blocks: u64,
    pub double_sign_slash_fraction: f64, // 0.05 (5%)
    pub downtime_slash_fraction: f64,    // 0.0001 (0.01%)
    pub downtime_jail_blocks: u64,       // 600 blocks
    pub min_self_stake: Amount,          // 1,000 SPRX
    pub missed_blocks_threshold: u64,    // 50 blocks
}

impl Default for StakingParams {
    fn default() -> Self {
        Self {
            max_validators: 100,
            unbonding_period_blocks: 10, // 10 blocks in devnet / 201600 in mainnet
            double_sign_slash_fraction: 0.05,
            downtime_slash_fraction: 0.0001,
            downtime_jail_blocks: 600,
            min_self_stake: Amount::from_sprx_whole(1_000).unwrap(),
            missed_blocks_threshold: 50,
        }
    }
}

/// In-memory staking and validator set state manager.
#[derive(Debug, Clone)]
pub struct StakingKeeper {
    params: StakingParams,
    validators: HashMap<Address, StakingValidator>,
    delegations: HashMap<(Address, Address), Delegation>, // (delegator, validator) -> Delegation
    unbonding_queue: Vec<UnbondingEntry>,
}

impl Default for StakingKeeper {
    fn default() -> Self {
        Self::new(StakingParams::default())
    }
}

impl StakingKeeper {
    #[must_use]
    pub fn new(params: StakingParams) -> Self {
        Self {
            params,
            validators: HashMap::new(),
            delegations: HashMap::new(),
            unbonding_queue: Vec::new(),
        }
    }

    #[must_use]
    pub fn params(&self) -> &StakingParams {
        &self.params
    }

    /// Registers a new validator candidate with self-stake.
    pub fn register_validator(
        &mut self,
        operator_address: Address,
        consensus_pubkey: Vec<u8>,
        description: ValidatorDescription,
        self_stake: Amount,
        commission: CommissionRates,
    ) -> Result<(), ConsensusError> {
        if self.validators.contains_key(&operator_address) {
            return Err(ConsensusError::InvalidValidatorSet(
                "validator already registered".into(),
            ));
        }
        if self_stake < self.params.min_self_stake {
            return Err(ConsensusError::InvalidValidatorSet(format!(
                "self stake {} is less than minimum required {}",
                self_stake, self.params.min_self_stake
            )));
        }

        let validator = StakingValidator {
            operator_address,
            consensus_pubkey,
            description,
            status: ValidatorStatus::Active,
            tokens: self_stake,
            delegator_shares: self_stake,
            min_self_delegation: self_stake,
            unbonding_height: None,
            commission,
            jailed_until: None,
            missed_blocks_count: 0,
            is_tombstoned: false,
        };

        self.validators.insert(operator_address, validator);

        // Record initial self-delegation
        let delegation = Delegation {
            delegator_address: operator_address,
            validator_address: operator_address,
            shares: self_stake,
            balance: self_stake,
        };
        self.delegations
            .insert((operator_address, operator_address), delegation);

        Ok(())
    }

    /// Delegates native tokens to an active validator.
    pub fn delegate(
        &mut self,
        delegator: Address,
        validator_addr: Address,
        amount: Amount,
    ) -> Result<(), ConsensusError> {
        let val = self.validators.get_mut(&validator_addr).ok_or_else(|| {
            ConsensusError::InvalidValidatorSet("target validator not found".into())
        })?;

        if val.is_tombstoned || val.status == ValidatorStatus::Unbonded {
            return Err(ConsensusError::InvalidValidatorSet(
                "cannot delegate to tombstoned or unbonded validator".into(),
            ));
        }

        val.tokens = val
            .tokens
            .checked_add(amount)
            .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?;
        val.delegator_shares = val
            .delegator_shares
            .checked_add(amount)
            .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?;

        let entry = self
            .delegations
            .entry((delegator, validator_addr))
            .or_insert(Delegation {
                delegator_address: delegator,
                validator_address: validator_addr,
                shares: Amount::ZERO,
                balance: Amount::ZERO,
            });

        entry.shares = entry
            .shares
            .checked_add(amount)
            .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?;
        entry.balance = entry
            .balance
            .checked_add(amount)
            .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?;

        Ok(())
    }

    /// Initiates undelegation and places tokens in the unbonding queue.
    pub fn undelegate(
        &mut self,
        delegator: Address,
        validator_addr: Address,
        amount: Amount,
        current_height: u64,
    ) -> Result<UnbondingEntry, ConsensusError> {
        let entry = self
            .delegations
            .get_mut(&(delegator, validator_addr))
            .ok_or_else(|| ConsensusError::InvalidValidatorSet("delegation not found".into()))?;

        if entry.balance < amount {
            return Err(ConsensusError::InvalidValidatorSet(
                "undelegation amount exceeds delegated balance".into(),
            ));
        }

        entry.balance = entry
            .balance
            .checked_sub(amount)
            .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?;
        entry.shares = entry
            .shares
            .checked_sub(amount)
            .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?;

        if let Some(val) = self.validators.get_mut(&validator_addr) {
            val.tokens = val
                .tokens
                .checked_sub(amount)
                .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?;
            val.delegator_shares = val
                .delegator_shares
                .checked_sub(amount)
                .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?;
        }

        let unbonding = UnbondingEntry {
            delegator_address: delegator,
            validator_address: validator_addr,
            creation_height: current_height,
            completion_height: current_height + self.params.unbonding_period_blocks,
            amount,
        };

        self.unbonding_queue.push(unbonding.clone());
        Ok(unbonding)
    }

    /// Slashes validator for double-signing (equivocation): 5% stake slashed, permanently tombstoned.
    pub fn slash_equivocation(
        &mut self,
        evidence: &EquivocationEvidence,
    ) -> Result<Amount, ConsensusError> {
        if !evidence.is_valid_equivocation() {
            return Err(ConsensusError::InvalidValidatorSet(
                "invalid equivocation evidence".into(),
            ));
        }

        let val = self
            .validators
            .get_mut(&evidence.validator_address)
            .ok_or_else(|| ConsensusError::InvalidValidatorSet("validator not found".into()))?;

        if val.is_tombstoned {
            return Ok(Amount::ZERO); // Already slashed and tombstoned
        }

        // 5% slash
        let slashed_atto =
            ((val.tokens.as_atto() as f64) * self.params.double_sign_slash_fraction) as u128;
        let slashed_amount = Amount::from_atto(slashed_atto);

        val.tokens = val
            .tokens
            .checked_sub(slashed_amount)
            .unwrap_or(Amount::ZERO);
        val.status = ValidatorStatus::Jailed;
        val.is_tombstoned = true;

        Ok(slashed_amount)
    }

    /// Slashes validator for downtime / liveness failure: 0.01% slashed, jailed for duration.
    pub fn slash_downtime(
        &mut self,
        validator_addr: &Address,
        current_height: u64,
    ) -> Result<Amount, ConsensusError> {
        let val = self
            .validators
            .get_mut(validator_addr)
            .ok_or_else(|| ConsensusError::InvalidValidatorSet("validator not found".into()))?;

        if val.status == ValidatorStatus::Jailed || val.is_tombstoned {
            return Ok(Amount::ZERO);
        }

        val.missed_blocks_count += 1;
        if val.missed_blocks_count >= self.params.missed_blocks_threshold {
            let slashed_atto =
                ((val.tokens.as_atto() as f64) * self.params.downtime_slash_fraction) as u128;
            let slashed_amount = Amount::from_atto(slashed_atto);

            val.tokens = val
                .tokens
                .checked_sub(slashed_amount)
                .unwrap_or(Amount::ZERO);
            val.status = ValidatorStatus::Jailed;
            val.jailed_until = Some(current_height + self.params.downtime_jail_blocks);
            val.missed_blocks_count = 0;

            return Ok(slashed_amount);
        }

        Ok(Amount::ZERO)
    }

    /// Unjails a previously jailed validator after jail duration expires.
    pub fn unjail(
        &mut self,
        validator_addr: &Address,
        current_height: u64,
    ) -> Result<(), ConsensusError> {
        let val = self
            .validators
            .get_mut(validator_addr)
            .ok_or_else(|| ConsensusError::InvalidValidatorSet("validator not found".into()))?;

        if val.is_tombstoned {
            return Err(ConsensusError::InvalidValidatorSet(
                "tombstoned validator cannot be unjailed".into(),
            ));
        }

        if let Some(until) = val.jailed_until {
            if current_height < until {
                return Err(ConsensusError::InvalidValidatorSet(format!(
                    "jail duration active until block #{until} (current: #{current_height})"
                )));
            }
        }

        val.status = ValidatorStatus::Active;
        val.jailed_until = None;
        val.missed_blocks_count = 0;

        Ok(())
    }

    /// Derives the current active `ValidatorSet` for CometBFT consensus (Top N by stake).
    pub fn get_active_validator_set(&self) -> Result<ValidatorSet, ConsensusError> {
        let mut active: Vec<&StakingValidator> = self
            .validators
            .values()
            .filter(|v| v.status == ValidatorStatus::Active && !v.tokens.is_zero())
            .collect();

        // Sort descending by token weight
        active.sort_by_key(|b| std::cmp::Reverse(b.tokens));
        active.truncate(self.params.max_validators);

        let consensus_validators: Vec<Validator> = active
            .into_iter()
            .map(|v| {
                // Voting power in millions of atto-SPRX units (or integer SPRX)
                let voting_power = (v.tokens.as_atto() / 1_000_000_000_000_000_000).max(1) as u64;
                Validator::new(v.operator_address, v.consensus_pubkey.clone(), voting_power)
            })
            .collect();

        ValidatorSet::new(consensus_validators)
    }

    pub fn get_validator(&self, addr: &Address) -> Option<&StakingValidator> {
        self.validators.get(addr)
    }

    pub fn get_delegation(&self, delegator: &Address, validator: &Address) -> Option<&Delegation> {
        self.delegations.get(&(*delegator, *validator))
    }

    pub fn get_delegator_delegations(&self, delegator: Address) -> Vec<Delegation> {
        self.delegations
            .values()
            .filter(|d| d.delegator_address == delegator)
            .cloned()
            .collect()
    }

    pub fn get_unbonding_entries(
        &self,
        delegator: &Address,
    ) -> Vec<crate::staking::UnbondingEntry> {
        self.unbonding_queue
            .iter()
            .filter(|u| u.delegator_address == *delegator)
            .cloned()
            .collect()
    }

    /// Persists full staking state (validators, delegations, unbonding queue) to a JSON file,
    /// mirroring `GenesisConfig::save_to_file`'s convention. Deliberately JSON rather than the
    /// RocksDB/redb state store: validator-set data is small and bounded by `max_validators`,
    /// unlike per-account ledger state.
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), ConsensusError> {
        let record = StakingKeeperRecord {
            params: self.params.clone(),
            validators: self.validators.clone(),
            // HashMap keys must serialize as JSON strings; (Address, Address) tuple keys can't,
            // so persist delegations as a flat list and rebuild the keyed map on load.
            delegations: self.delegations.values().cloned().collect(),
            unbonding_queue: self.unbonding_queue.clone(),
        };
        let json_str = serde_json::to_string_pretty(&record)
            .map_err(|e| ConsensusError::InvalidValidatorSet(format!("json encode error: {e}")))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ConsensusError::InvalidValidatorSet(format!("fs error: {e}")))?;
        }
        std::fs::write(path, json_str)
            .map_err(|e| ConsensusError::InvalidValidatorSet(format!("fs write error: {e}")))?;
        Ok(())
    }

    /// Loads staking state previously written by [`Self::save_to_file`].
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, ConsensusError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConsensusError::InvalidValidatorSet(format!("fs read error: {e}")))?;
        let record: StakingKeeperRecord = serde_json::from_str(&content)
            .map_err(|e| ConsensusError::InvalidValidatorSet(format!("json parse error: {e}")))?;
        let delegations = record
            .delegations
            .into_iter()
            .map(|d| ((d.delegator_address, d.validator_address), d))
            .collect();
        Ok(Self {
            params: record.params,
            validators: record.validators,
            delegations,
            unbonding_queue: record.unbonding_queue,
        })
    }
}

/// On-disk representation for [`StakingKeeper::save_to_file`] / [`StakingKeeper::load_from_file`].
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StakingKeeperRecord {
    params: StakingParams,
    validators: HashMap<Address, StakingValidator>,
    delegations: Vec<Delegation>,
    unbonding_queue: Vec<UnbondingEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vote::{Vote, VoteType};
    use sprax_types::Hash32;

    #[test]
    fn test_validator_registration_and_delegation() {
        let mut keeper = StakingKeeper::default();
        let val_addr = Address::new([1u8; 20]);
        let del_addr = Address::new([2u8; 20]);

        let desc = ValidatorDescription {
            moniker: "Genesis-Val-1".into(),
            identity: "".into(),
            website: "https://sprax.io".into(),
            details: "Official testnet node".into(),
        };

        // Self stake 5,000 SPRX
        let self_stake = Amount::from_sprx_whole(5_000).unwrap();
        keeper
            .register_validator(
                val_addr,
                vec![1; 32],
                desc,
                self_stake,
                CommissionRates::default(),
            )
            .unwrap();

        let val = keeper.get_validator(&val_addr).unwrap();
        assert_eq!(val.status, ValidatorStatus::Active);
        assert_eq!(val.tokens, self_stake);

        // Delegator delegates 2,000 SPRX
        let del_amount = Amount::from_sprx_whole(2_000).unwrap();
        keeper.delegate(del_addr, val_addr, del_amount).unwrap();

        let val_after = keeper.get_validator(&val_addr).unwrap();
        assert_eq!(val_after.tokens, Amount::from_sprx_whole(7_000).unwrap());

        let del = keeper.get_delegation(&del_addr, &val_addr).unwrap();
        assert_eq!(del.balance, del_amount);

        let val_set = keeper.get_active_validator_set().unwrap();
        assert_eq!(val_set.len(), 1);
        assert_eq!(val_set.total_voting_power(), 7000);
    }

    #[test]
    fn test_equivocation_slashing_and_tombstoning() {
        let mut keeper = StakingKeeper::default();
        let val_addr = Address::new([1u8; 20]);

        keeper
            .register_validator(
                val_addr,
                vec![1; 32],
                ValidatorDescription {
                    moniker: "Val-1".into(),
                    identity: "".into(),
                    website: "".into(),
                    details: "".into(),
                },
                Amount::from_sprx_whole(100_000).unwrap(),
                CommissionRates::default(),
            )
            .unwrap();

        let evidence = EquivocationEvidence {
            validator_address: val_addr,
            height: 42,
            round: 0,
            vote_a: Vote::new(
                VoteType::Precommit,
                42,
                0,
                Some(Hash32::new([1u8; 32])),
                val_addr,
                vec![1; 64],
            ),
            vote_b: Vote::new(
                VoteType::Precommit,
                42,
                0,
                Some(Hash32::new([2u8; 32])),
                val_addr,
                vec![2; 64],
            ),
        };

        let slashed = keeper.slash_equivocation(&evidence).unwrap();
        assert_eq!(slashed, Amount::from_sprx_whole(5_000).unwrap()); // 5% of 100,000

        let val_after = keeper.get_validator(&val_addr).unwrap();
        assert_eq!(val_after.status, ValidatorStatus::Jailed);
        assert!(val_after.is_tombstoned);
        assert_eq!(val_after.tokens, Amount::from_sprx_whole(95_000).unwrap());
    }

    #[test]
    fn test_staking_keeper_json_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("staking.json");

        let mut keeper = StakingKeeper::default();
        let val_addr = Address::new([7u8; 20]);
        let del_addr = Address::new([8u8; 20]);

        keeper
            .register_validator(
                val_addr,
                vec![7; 32],
                ValidatorDescription {
                    moniker: "Val-7".into(),
                    identity: "".into(),
                    website: "".into(),
                    details: "".into(),
                },
                Amount::from_sprx_whole(10_000).unwrap(),
                CommissionRates::default(),
            )
            .unwrap();
        keeper
            .delegate(del_addr, val_addr, Amount::from_sprx_whole(500).unwrap())
            .unwrap();

        // Tombstone the validator via equivocation so the round-trip also covers jailed state.
        let evidence = EquivocationEvidence {
            validator_address: val_addr,
            height: 10,
            round: 0,
            vote_a: Vote::new(
                VoteType::Precommit,
                10,
                0,
                Some(Hash32::new([1u8; 32])),
                val_addr,
                vec![1; 64],
            ),
            vote_b: Vote::new(
                VoteType::Precommit,
                10,
                0,
                Some(Hash32::new([2u8; 32])),
                val_addr,
                vec![2; 64],
            ),
        };
        keeper.slash_equivocation(&evidence).unwrap();

        keeper.save_to_file(&path).unwrap();
        let reloaded = StakingKeeper::load_from_file(&path).unwrap();

        let original_val = keeper.get_validator(&val_addr).unwrap();
        let reloaded_val = reloaded.get_validator(&val_addr).unwrap();
        assert_eq!(reloaded_val.tokens, original_val.tokens);
        assert_eq!(reloaded_val.status, ValidatorStatus::Jailed);
        assert!(reloaded_val.is_tombstoned);

        let reloaded_del = reloaded.get_delegation(&del_addr, &val_addr).unwrap();
        assert_eq!(reloaded_del.balance, Amount::from_sprx_whole(500).unwrap());
    }
}
