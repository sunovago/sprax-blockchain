use crate::{state::CanonicalValidatorStatus, CoreError, GenesisConfig, StateAccessor};
use sprax_storage::ReadonlyKVStore;
use sprax_types::{Amount, CanonicalValidator};

/// Version-one protocol limit.
pub const MAX_ACTIVE_VALIDATORS: usize = 100;

/// Minimum self-stake required to be eligible for the active set (1,000 SPRX).
pub const MIN_SELF_STAKE: Amount = Amount::from_atto(1_000_000_000_000_000_000_000); // 1,000 SPRX

pub fn canonical_validator_set<S: ReadonlyKVStore>(
    genesis: &GenesisConfig,
    store: &S,
) -> Result<Vec<CanonicalValidator>, CoreError> {
    let records = StateAccessor::scan_all_validators(store)?;

    let mut bonded = Vec::new();
    if !records.is_empty() {
        for record in records {
            // Exclude jailed or tombstoned validators
            if record.status != CanonicalValidatorStatus::Active || record.is_tombstoned {
                continue;
            }
            let stake = StateAccessor::get_validator_stake(store, &record.operator_address)?;
            if stake.tokens >= MIN_SELF_STAKE
                || (!stake.tokens.is_zero()
                    && genesis
                        .validators
                        .iter()
                        .any(|v| v.operator_address == record.operator_address))
            {
                bonded.push((
                    stake.tokens,
                    record.operator_address,
                    record.consensus_pubkey,
                ));
            }
        }
    } else {
        // Fallback for genesis validators if records not yet scanned
        for registered in &genesis.validators {
            let stake = StateAccessor::get_validator_stake(store, &registered.operator_address)?;
            if !stake.tokens.is_zero() {
                bonded.push((
                    stake.tokens,
                    registered.operator_address,
                    registered.consensus_pubkey.clone(),
                ));
            }
        }
    }

    // Sort descending by token weight, then deterministically by operator address
    bonded.sort_by(|(a, addr_a, _), (b, addr_b, _)| b.cmp(a).then_with(|| addr_a.cmp(addr_b)));
    bonded.truncate(MAX_ACTIVE_VALIDATORS);

    let mut total = 0u64;
    let mut validators = Vec::with_capacity(bonded.len());
    for (tokens, operator_address, consensus_pubkey) in bonded {
        let power = u64::try_from((tokens.as_atto() / sprax_types::ATTO_SPRX_PER_SPRX).max(1))
            .map_err(|_| CoreError::StateError("validator voting power overflow".into()))?;
        total = total
            .checked_add(power)
            .filter(|sum| *sum <= i64::MAX as u64 / 4)
            .ok_or_else(|| {
                CoreError::StateError("validator total voting power exceeds protocol bound".into())
            })?;
        validators.push(CanonicalValidator {
            address: operator_address,
            public_key: consensus_pubkey,
            voting_power: power,
        });
    }

    validators.sort_by_key(|v| v.address);
    Ok(validators)
}

pub fn validator_set_hash(
    validators: &[CanonicalValidator],
) -> Result<sprax_types::Hash32, CoreError> {
    let encoded =
        serde_json::to_vec(validators).map_err(|e| CoreError::StateError(e.to_string()))?;
    Ok(sprax_crypto::Hasher::sha256(&encoded))
}
