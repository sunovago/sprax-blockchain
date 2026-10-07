use crate::{CoreError, GenesisConfig, StateAccessor};
use sprax_storage::ReadonlyKVStore;
use sprax_types::CanonicalValidator;

/// Protocol active-set limit, applied after canonical registry eligibility checks.
pub const MAX_ACTIVE_VALIDATORS: usize = 100;

pub fn canonical_validator_set<S: ReadonlyKVStore>(
    genesis: &GenesisConfig,
    store: &S,
) -> Result<Vec<CanonicalValidator>, CoreError> {
    let mut bonded = Vec::with_capacity(genesis.validators.len());
    for registered in crate::validator_lifecycle::registry(store)? {
        if registered.jailed || registered.tombstoned {
            continue;
        }
        let self_bond = StateAccessor::get_delegation(
            store,
            &registered.operator_address,
            &registered.operator_address,
        )?;
        if self_bond.balance < genesis.consensus_params.validator_policy.min_self_stake {
            continue;
        }
        let stake = StateAccessor::get_validator_stake(store, &registered.operator_address)?;
        if !stake.tokens.is_zero() {
            bonded.push((stake.tokens, registered));
        }
    }
    bonded.sort_by(|(a, av), (b, bv)| {
        b.cmp(a)
            .then_with(|| av.operator_address.cmp(&bv.operator_address))
    });
    bonded.truncate(MAX_ACTIVE_VALIDATORS);
    let mut total = 0u64;
    let mut validators = Vec::with_capacity(bonded.len());
    for (tokens, registered) in bonded {
        let power = u64::try_from((tokens.as_atto() / sprax_types::ATTO_SPRX_PER_SPRX).max(1))
            .map_err(|_| CoreError::StateError("validator voting power overflow".into()))?;
        total = total
            .checked_add(power)
            .filter(|sum| *sum <= i64::MAX as u64 / 4)
            .ok_or_else(|| {
                CoreError::StateError("validator total voting power exceeds protocol bound".into())
            })?;
        validators.push(CanonicalValidator {
            address: registered.operator_address,
            public_key: registered.consensus_pubkey.clone(),
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
