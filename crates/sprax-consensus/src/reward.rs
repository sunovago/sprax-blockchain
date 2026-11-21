use crate::{error::ConsensusError, staking::StakingKeeper};
use serde::{Deserialize, Serialize};
use sprax_types::{Address, Amount};
use std::collections::HashMap;
use std::path::Path;

/// Parameters governing block rewards, transaction fee sharing, and commission deductions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewardParams {
    /// Fixed base reward minted per finalized block (e.g., 2 SPRX).
    pub base_block_reward: Amount,
    /// Percentage (in basis points, 10,000 bps = 100%) allocated as a bonus to the block proposer.
    pub proposer_bonus_bps: u64,
    /// Community pool allocation (in basis points).
    pub community_tax_bps: u64,
}

impl Default for RewardParams {
    fn default() -> Self {
        Self {
            base_block_reward: Amount::from_sprx_whole(2).expect("valid 2 SPRX"),
            proposer_bonus_bps: 1_000, // 10.00%
            community_tax_bps: 200,    // 2.00%
        }
    }
}

/// Audit receipt for a block's reward distribution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockRewardDistribution {
    pub height: u64,
    pub proposer: Address,
    pub total_pot: Amount,
    pub community_tax: Amount,
    pub proposer_bonus: Amount,
    pub validator_commissions: HashMap<Address, Amount>,
    pub delegator_rewards: HashMap<(Address, Address), Amount>,
    pub remainder: Amount,
}

/// Storage record for serializing reward keeper state to JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RewardKeeperRecord {
    params: RewardParams,
    community_pool: Amount,
    validator_commissions: HashMap<Address, Amount>,
    delegator_rewards: Vec<DelegatorRewardEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DelegatorRewardEntry {
    delegator: Address,
    validator: Address,
    amount: Amount,
}

/// Computes (a * b) / denom with full 256-bit intermediate precision, preventing
/// u128 multiplication overflow when multiplying large 18-decimal fixed-point amounts.
#[must_use]
pub fn mul_div_u128(a: u128, b: u128, denom: u128) -> Option<u128> {
    if denom == 0 {
        return None;
    }
    if let Some(prod) = a.checked_mul(b) {
        return Some(prod / denom);
    }

    let a_lo = a as u64 as u128;
    let a_hi = a >> 64;
    let b_lo = b as u64 as u128;
    let b_hi = b >> 64;

    let p0 = a_lo * b_lo;
    let p1 = a_lo * b_hi;
    let p2 = a_hi * b_lo;
    let p3 = a_hi * b_hi;

    let p0_hi = p0 >> 64;
    let p1_lo = p1 as u64 as u128;
    let p1_hi = p1 >> 64;
    let p2_lo = p2 as u64 as u128;
    let p2_hi = p2 >> 64;

    let mid_lo = p0_hi + p1_lo + p2_lo;
    let lo = (mid_lo << 64) | (p0 as u64 as u128);
    let hi = p3 + p1_hi + p2_hi + (mid_lo >> 64);

    if hi >= denom || denom >= (1u128 << 127) {
        return None;
    }

    let mut rem = hi;
    let mut quot = 0u128;

    for i in (0..128).rev() {
        let bit = (lo >> i) & 1;
        rem = (rem << 1) | bit;
        if rem >= denom {
            rem -= denom;
            quot |= 1u128 << i;
        }
    }

    Some(quot)
}

/// In-memory manager for validator and delegator rewards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardKeeper {
    params: RewardParams,
    community_pool: Amount,
    validator_commissions: HashMap<Address, Amount>,
    delegator_rewards: HashMap<(Address, Address), Amount>,
}

impl Default for RewardKeeper {
    fn default() -> Self {
        Self::new(RewardParams::default())
    }
}

impl RewardKeeper {
    #[must_use]
    pub fn new(params: RewardParams) -> Self {
        Self {
            params,
            community_pool: Amount::ZERO,
            validator_commissions: HashMap::new(),
            delegator_rewards: HashMap::new(),
        }
    }

    #[must_use]
    pub fn params(&self) -> &RewardParams {
        &self.params
    }

    #[must_use]
    pub fn community_pool(&self) -> Amount {
        self.community_pool
    }

    #[must_use]
    pub fn get_operator_commission(&self, validator: &Address) -> Amount {
        self.validator_commissions
            .get(validator)
            .copied()
            .unwrap_or(Amount::ZERO)
    }

    #[must_use]
    pub fn get_delegator_reward(&self, delegator: &Address, validator: &Address) -> Amount {
        self.delegator_rewards
            .get(&(*delegator, *validator))
            .copied()
            .unwrap_or(Amount::ZERO)
    }

    /// Claims accumulated commission for a validator operator.
    pub fn claim_operator_commission(
        &mut self,
        validator: &Address,
    ) -> Result<Amount, ConsensusError> {
        let entry = self
            .validator_commissions
            .get_mut(validator)
            .ok_or_else(|| {
                ConsensusError::InvalidValidatorSet("no commission rewards found".into())
            })?;
        let claimed = *entry;
        *entry = Amount::ZERO;
        Ok(claimed)
    }

    /// Claims accumulated rewards for a delegator from a specific validator.
    pub fn claim_delegator_reward(
        &mut self,
        delegator: &Address,
        validator: &Address,
    ) -> Result<Amount, ConsensusError> {
        let entry = self
            .delegator_rewards
            .get_mut(&(*delegator, *validator))
            .ok_or_else(|| {
                ConsensusError::InvalidValidatorSet("no delegator rewards found".into())
            })?;
        let claimed = *entry;
        *entry = Amount::ZERO;
        Ok(claimed)
    }

    /// Distributes rewards for a finalized block among proposer, committers, and delegators.
    ///
    /// Mathematical Invariants:
    /// 1. pot = base_block_reward + tx_fees
    /// 2. community_tax = (pot * community_tax_bps) / 10,000
    /// 3. proposer_bonus = (pot * proposer_bonus_bps) / 10,000
    /// 4. committer_pot = pot - community_tax - proposer_bonus
    /// 5. Committer shares divided proportionally by voting stake.
    /// 6. Validator commission deducted from each committer share.
    /// 7. Remainder from divisions strictly credited to community pool or operator so that
    ///    total distributed tokens identically equals `pot`.
    pub fn distribute_block_rewards(
        &mut self,
        height: u64,
        proposer: Address,
        tx_fees: Amount,
        committers: &[Address],
        staking_keeper: &StakingKeeper,
    ) -> Result<BlockRewardDistribution, ConsensusError> {
        let total_pot = self
            .params
            .base_block_reward
            .checked_add(tx_fees)
            .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?;

        if total_pot.is_zero() {
            return Ok(BlockRewardDistribution {
                height,
                proposer,
                total_pot: Amount::ZERO,
                community_tax: Amount::ZERO,
                proposer_bonus: Amount::ZERO,
                validator_commissions: HashMap::new(),
                delegator_rewards: HashMap::new(),
                remainder: Amount::ZERO,
            });
        }

        let pot_atto = total_pot.as_atto();

        // 1. Community tax
        let community_tax_atto = pot_atto
            .checked_mul(self.params.community_tax_bps as u128)
            .ok_or_else(|| ConsensusError::InvalidValidatorSet("community tax overflow".into()))?
            / 10_000;

        // 2. Proposer bonus
        let proposer_bonus_atto = pot_atto
            .checked_mul(self.params.proposer_bonus_bps as u128)
            .ok_or_else(|| ConsensusError::InvalidValidatorSet("proposer bonus overflow".into()))?
            / 10_000;

        // 3. Pool for committers
        let committer_pool_atto = pot_atto
            .checked_sub(community_tax_atto + proposer_bonus_atto)
            .ok_or_else(|| {
                ConsensusError::InvalidValidatorSet("reward pool subtraction underflow".into())
            })?;

        // Proposer gets bonus credited to operator commission
        let current_proposer_comm = self
            .validator_commissions
            .entry(proposer)
            .or_insert(Amount::ZERO);
        *current_proposer_comm = current_proposer_comm
            .checked_add(Amount::from_atto(proposer_bonus_atto))
            .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?;

        // Map and sum committer token stakes
        let mut committer_stakes: Vec<(Address, u128)> = Vec::new();
        let mut total_committer_tokens_atto = 0u128;

        for addr in committers {
            if let Some(val) = staking_keeper.get_validator(addr) {
                let stake = val.tokens.as_atto();
                if stake > 0 {
                    committer_stakes.push((*addr, stake));
                    total_committer_tokens_atto = total_committer_tokens_atto
                        .checked_add(stake)
                        .ok_or_else(|| {
                            ConsensusError::InvalidValidatorSet(
                                "total committer stake overflow".into(),
                            )
                        })?;
                }
            }
        }

        let mut round_val_commissions: HashMap<Address, Amount> = HashMap::new();
        let mut round_del_rewards: HashMap<(Address, Address), Amount> = HashMap::new();
        let mut total_committer_distributed_atto = 0u128;

        if total_committer_tokens_atto > 0 {
            for (val_addr, stake_atto) in &committer_stakes {
                let val_share_atto = mul_div_u128(
                    committer_pool_atto,
                    *stake_atto,
                    total_committer_tokens_atto,
                )
                .ok_or_else(|| {
                    ConsensusError::InvalidValidatorSet("share calculation overflow".into())
                })?;

                total_committer_distributed_atto = total_committer_distributed_atto
                    .checked_add(val_share_atto)
                    .ok_or_else(|| {
                        ConsensusError::InvalidValidatorSet(
                            "committer share addition overflow".into(),
                        )
                    })?;

                let val = staking_keeper.get_validator(val_addr).ok_or_else(|| {
                    ConsensusError::InvalidValidatorSet("committer validator not found".into())
                })?;

                // Validator commission
                let commission_bps = val.commission.rate_bps as u128;
                let commission_atto =
                    mul_div_u128(val_share_atto, commission_bps, 10_000).unwrap_or(0);

                let delegator_pool_atto = val_share_atto.saturating_sub(commission_atto);

                // Distribute among delegators
                let total_shares_atto = val.delegator_shares.as_atto();
                let mut sum_del_atto = 0u128;

                if total_shares_atto > 0 {
                    let delegations = staking_keeper.get_validator_delegations(val_addr);
                    for delegation in &delegations {
                        let del_share_atto = delegation.shares.as_atto();
                        let del_addr = delegation.delegator_address;
                        if del_share_atto > 0 {
                            let reward_atto = mul_div_u128(
                                delegator_pool_atto,
                                del_share_atto,
                                total_shares_atto,
                            )
                            .unwrap_or(0);

                            if reward_atto > 0 {
                                sum_del_atto = sum_del_atto
                                    .checked_add(reward_atto)
                                    .unwrap_or(sum_del_atto);

                                let entry = self
                                    .delegator_rewards
                                    .entry((del_addr, *val_addr))
                                    .or_insert(Amount::ZERO);
                                *entry =
                                    entry.checked_add(Amount::from_atto(reward_atto)).map_err(
                                        |e| ConsensusError::InvalidValidatorSet(e.to_string()),
                                    )?;

                                round_del_rewards
                                    .insert((del_addr, *val_addr), Amount::from_atto(reward_atto));
                            }
                        }
                    }
                }

                // Any delegator rounding remainder goes to operator commission
                let del_remainder_atto = delegator_pool_atto.saturating_sub(sum_del_atto);
                let total_val_comm_atto = commission_atto
                    .checked_add(del_remainder_atto)
                    .unwrap_or(commission_atto);

                let comm_entry = self
                    .validator_commissions
                    .entry(*val_addr)
                    .or_insert(Amount::ZERO);
                *comm_entry = comm_entry
                    .checked_add(Amount::from_atto(total_val_comm_atto))
                    .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?;

                round_val_commissions.insert(*val_addr, Amount::from_atto(total_val_comm_atto));
            }
        }

        // Remainder from committer distribution division goes to community pool
        let committer_remainder_atto = committer_pool_atto
            .checked_sub(total_committer_distributed_atto)
            .unwrap_or(committer_pool_atto);

        let total_community_tax_atto = community_tax_atto
            .checked_add(committer_remainder_atto)
            .unwrap_or(community_tax_atto);

        self.community_pool = self
            .community_pool
            .checked_add(Amount::from_atto(total_community_tax_atto))
            .map_err(|e| ConsensusError::InvalidValidatorSet(e.to_string()))?;

        Ok(BlockRewardDistribution {
            height,
            proposer,
            total_pot,
            community_tax: Amount::from_atto(total_community_tax_atto),
            proposer_bonus: Amount::from_atto(proposer_bonus_atto),
            validator_commissions: round_val_commissions,
            delegator_rewards: round_del_rewards,
            remainder: Amount::from_atto(committer_remainder_atto),
        })
    }

    /// Persists reward keeper state to JSON file.
    pub fn save_to_file(&self, path: &Path) -> Result<(), ConsensusError> {
        let delegator_rewards = self
            .delegator_rewards
            .iter()
            .map(|((del, val), amount)| DelegatorRewardEntry {
                delegator: *del,
                validator: *val,
                amount: *amount,
            })
            .collect();

        let record = RewardKeeperRecord {
            params: self.params.clone(),
            community_pool: self.community_pool,
            validator_commissions: self.validator_commissions.clone(),
            delegator_rewards,
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

    /// Loads reward keeper state from JSON file.
    pub fn load_from_file(path: &Path) -> Result<Self, ConsensusError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConsensusError::InvalidValidatorSet(format!("fs read error: {e}")))?;
        let record: RewardKeeperRecord = serde_json::from_str(&content)
            .map_err(|e| ConsensusError::InvalidValidatorSet(format!("json parse error: {e}")))?;
        let delegator_rewards = record
            .delegator_rewards
            .into_iter()
            .map(|e| ((e.delegator, e.validator), e.amount))
            .collect();

        Ok(Self {
            params: record.params,
            community_pool: record.community_pool,
            validator_commissions: record.validator_commissions,
            delegator_rewards,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::staking::{CommissionRates, ValidatorDescription};

    #[test]
    fn test_block_reward_distribution_and_claims() {
        let mut staking = StakingKeeper::default();
        let val1 = Address::new([1u8; 20]);
        let val2 = Address::new([2u8; 20]);
        let del1 = Address::new([3u8; 20]);

        let desc = ValidatorDescription {
            moniker: "V1".into(),
            identity: "".into(),
            website: "".into(),
            details: "".into(),
        };

        let comm = CommissionRates {
            rate_bps: 1_000, // 10% commission
            max_rate_bps: 2_000,
            max_change_rate_bps: 100,
        };

        // Val 1: 10,000 SPRX
        staking
            .register_validator(
                val1,
                vec![1; 32],
                desc.clone(),
                Amount::from_sprx_whole(10_000).unwrap(),
                comm.clone(),
            )
            .unwrap();

        // Val 2: 10,000 SPRX
        staking
            .register_validator(
                val2,
                vec![2; 32],
                desc,
                Amount::from_sprx_whole(10_000).unwrap(),
                comm,
            )
            .unwrap();

        // Del 1 delegates 10,000 SPRX to Val 1
        staking
            .delegate(del1, val1, Amount::from_sprx_whole(10_000).unwrap())
            .unwrap();

        let mut reward_keeper = RewardKeeper::new(RewardParams::default());
        let tx_fees = Amount::from_sprx_whole(1).unwrap(); // 1 SPRX in tx fees
                                                           // Total pot = 2 (base) + 1 (fees) = 3 SPRX

        let distribution = reward_keeper
            .distribute_block_rewards(1, val1, tx_fees, &[val1, val2], &staking)
            .unwrap();

        assert_eq!(distribution.total_pot, Amount::from_sprx_whole(3).unwrap());
        // Proposer bonus is 10% of 3 SPRX = 0.3 SPRX
        assert_eq!(
            distribution.proposer_bonus,
            Amount::from_atto(300_000_000_000_000_000)
        );

        // Community pool tax is 2% of 3 SPRX = 0.06 SPRX + rounding remainder
        assert!(reward_keeper.community_pool().as_atto() >= 60_000_000_000_000_000);

        // Val 1 operator commission should be > 0 (proposer bonus + validator commission)
        let comm_val1 = reward_keeper.get_operator_commission(&val1);
        assert!(comm_val1.as_atto() > 0);

        // Delegator reward for Del 1 should be > 0
        let del_reward = reward_keeper.get_delegator_reward(&del1, &val1);
        assert!(del_reward.as_atto() > 0);

        // Claim delegator reward
        let claimed = reward_keeper.claim_delegator_reward(&del1, &val1).unwrap();
        assert_eq!(claimed, del_reward);
        assert_eq!(
            reward_keeper.get_delegator_reward(&del1, &val1),
            Amount::ZERO
        );

        // Claim commission
        let claimed_comm = reward_keeper.claim_operator_commission(&val1).unwrap();
        assert_eq!(claimed_comm, comm_val1);
        assert_eq!(reward_keeper.get_operator_commission(&val1), Amount::ZERO);
    }
}
