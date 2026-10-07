//! Canonical, transaction-driven validator registration and equivocation penalties.
//! All mutations are called through the transaction/block overlays, never from gossip.
use crate::{gas::GasMeter, CoreError, GenesisConfig, StateAccessor};
use serde::{Deserialize, Serialize};
use sprax_crypto::{Ed25519Keypair, Hasher};
use sprax_storage::{KVStore, ReadonlyKVStore};
use sprax_types::{Address, Amount, CanonicalValidator, Hash32};

const REGISTRY: &[u8] = b"staking/registry/v1";
const POLICY: &[u8] = b"staking/policy/v1";
pub const MAX_REGISTERED_VALIDATORS: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorPolicy {
    pub double_sign_slash_bps: u16,
    pub min_self_stake: Amount,
    pub jail_blocks: u64,
}
impl Default for ValidatorPolicy {
    fn default() -> Self {
        Self {
            double_sign_slash_bps: 500,
            min_self_stake: Amount::ONE_SPRX,
            jail_blocks: 10,
        }
    }
}
impl ValidatorPolicy {
    pub fn validate(&self) -> Result<(), CoreError> {
        if !(1..=10000).contains(&self.double_sign_slash_bps)
            || self.min_self_stake.is_zero()
            || self.jail_blocks == 0
        {
            return Err(error("invalid validator policy"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorRecord {
    pub operator_address: Address,
    pub consensus_pubkey: Vec<u8>,
    pub moniker: String,
    pub registered_height: u64,
    pub jailed: bool,
    pub jailed_until: u64,
    pub tombstoned: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlashRecord {
    pub evidence_hash: Hash32,
    pub infraction_height: u64,
    pub processed_height: u64,
    pub burned: Amount,
}
fn error(message: &str) -> CoreError {
    CoreError::StateError(message.into())
}
fn encode(value: &impl Serialize) -> Result<Vec<u8>, CoreError> {
    serde_json::to_vec(value).map_err(|e| error(&e.to_string()))
}
fn put(store: &impl KVStore, key: &[u8], value: &impl Serialize) -> Result<(), CoreError> {
    store
        .set(key, &encode(value)?)
        .map_err(|e| error(&e.to_string()))
}
fn read<T: serde::de::DeserializeOwned>(
    store: &impl ReadonlyKVStore,
    key: &[u8],
) -> Result<T, CoreError> {
    let bytes = store
        .get(key)
        .map_err(|e| error(&e.to_string()))?
        .ok_or_else(|| error("missing canonical staking state"))?;
    serde_json::from_slice(&bytes).map_err(|e| error(&e.to_string()))
}
pub fn registry(store: &impl ReadonlyKVStore) -> Result<Vec<ValidatorRecord>, CoreError> {
    read(store, REGISTRY)
}
pub fn policy(store: &impl ReadonlyKVStore) -> Result<ValidatorPolicy, CoreError> {
    read(store, POLICY)
}
pub(crate) fn initialize(store: &impl KVStore, genesis: &GenesisConfig) -> Result<(), CoreError> {
    if genesis.validators.len() > MAX_REGISTERED_VALIDATORS {
        return Err(error("validator registry capacity exceeded"));
    }
    let mut records = Vec::new();
    for v in &genesis.validators {
        if v.self_stake < genesis.consensus_params.validator_policy.min_self_stake
            || v.moniker.len() > 128
        {
            return Err(error("invalid genesis validator self-stake or moniker"));
        }
        records.push(ValidatorRecord {
            operator_address: v.operator_address,
            consensus_pubkey: v.consensus_pubkey.clone(),
            moniker: v.moniker.clone(),
            registered_height: 0,
            jailed: false,
            jailed_until: 0,
            tombstoned: false,
        });
    }
    records.sort_by_key(|r| r.operator_address);
    put(store, REGISTRY, &records)?;
    put(store, POLICY, &genesis.consensus_params.validator_policy)
}
pub fn registration_sign_bytes(
    genesis: Hash32,
    operator: Address,
    nonce: u64,
    public_key: &[u8],
) -> Result<Vec<u8>, CoreError> {
    encode(&(
        "sprax/validator-registration/v1",
        genesis,
        operator,
        nonce,
        public_key,
    ))
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn register(
    store: &impl KVStore,
    operator: Address,
    nonce: u64,
    height: u64,
    key: &[u8],
    proof: &[u8],
    self_stake: Amount,
    moniker: &str,
) -> Result<(), CoreError> {
    if key.len() != 32
        || proof.len() != 64
        || moniker.len() > 128
        || self_stake < policy(store)?.min_self_stake
    {
        return Err(error("invalid validator registration"));
    }
    let derived = Hasher::blake3(key);
    if operator.as_bytes() != &derived.as_bytes()[..20] {
        return Err(error(
            "consensus key must derive the operator address in this protocol version",
        ));
    }
    let mut records = registry(store)?;
    if records.len() >= MAX_REGISTERED_VALIDATORS
        || records
            .iter()
            .any(|v| v.operator_address == operator || v.consensus_pubkey == key)
    {
        return Err(error("operator/key already registered or registry full"));
    }
    let identity = store
        .get(GenesisConfig::IDENTITY_KEY)
        .map_err(|e| error(&e.to_string()))?
        .ok_or_else(|| error("missing genesis identity"))?;
    let genesis = Hash32::new(
        identity
            .try_into()
            .map_err(|_| error("invalid genesis identity"))?,
    );
    Ed25519Keypair::verify(
        key,
        &registration_sign_bytes(genesis, operator, nonce, key)?,
        proof,
    )
    .map_err(|_| error("invalid consensus-key possession proof"))?;
    records.push(ValidatorRecord {
        operator_address: operator,
        consensus_pubkey: key.to_vec(),
        moniker: moniker.into(),
        registered_height: height,
        jailed: false,
        jailed_until: 0,
        tombstoned: false,
    });
    records.sort_by_key(|r| r.operator_address);
    put(store, REGISTRY, &records)?;
    let mut stake = StateAccessor::get_validator_stake(store, &operator)?;
    stake.tokens = stake
        .tokens
        .checked_add(self_stake)
        .map_err(|e| error(&e.to_string()))?;
    StateAccessor::set_validator_stake(store, &operator, &stake)?;
    let mut delegation = StateAccessor::get_delegation(store, &operator, &operator)?;
    delegation.balance = delegation
        .balance
        .checked_add(self_stake)
        .map_err(|e| error(&e.to_string()))?;
    delegation.shares = delegation.balance;
    StateAccessor::set_delegation(store, &operator, &operator, &delegation)
}
pub(crate) fn ensure_delegatable(
    store: &impl ReadonlyKVStore,
    operator: Address,
    delegator: Address,
) -> Result<(), CoreError> {
    let record = registry(store)?
        .into_iter()
        .find(|v| v.operator_address == operator)
        .ok_or_else(|| error("validator not registered"))?;
    if record.tombstoned || (record.jailed && operator != delegator) {
        return Err(error("validator cannot receive new delegation"));
    }
    Ok(())
}
pub(crate) fn jail(
    store: &impl KVStore,
    operator: Address,
    height: u64,
    unjail: bool,
) -> Result<(), CoreError> {
    let config = policy(store)?;
    let mut records = registry(store)?;
    let record = records
        .iter_mut()
        .find(|v| v.operator_address == operator)
        .ok_or_else(|| error("validator not registered"))?;
    if record.tombstoned {
        return Err(error("tombstone is permanent"));
    }
    if unjail {
        if !record.jailed
            || height < record.jailed_until
            || StateAccessor::get_delegation(store, &operator, &operator)?.balance
                < config.min_self_stake
        {
            return Err(error("validator not eligible for unjail"));
        }
        record.jailed = false;
        record.jailed_until = 0;
    } else {
        record.jailed = true;
        record.jailed_until = height
            .checked_add(config.jail_blocks)
            .ok_or_else(|| error("jail height overflow"))?;
    }
    put(store, REGISTRY, &records)
}
pub(crate) fn check_self_bond(
    store: &impl KVStore,
    operator: Address,
    height: u64,
) -> Result<(), CoreError> {
    let record = registry(store)?
        .into_iter()
        .find(|v| v.operator_address == operator);
    if record.is_some_and(|v| !v.jailed && !v.tombstoned)
        && StateAccessor::get_delegation(store, &operator, &operator)?.balance
            < policy(store)?.min_self_stake
    {
        jail(store, operator, height, false)?;
    }
    Ok(())
}
fn history_key(height: u64) -> Vec<u8> {
    [b"staking/history/".as_slice(), &height.to_be_bytes()].concat()
}
pub(crate) fn record_active_set(
    store: &impl KVStore,
    height: u64,
    period: u64,
    validators: &[CanonicalValidator],
) -> Result<(), CoreError> {
    put(store, &history_key(height), &validators)?;
    if let Some(expired) = height.checked_sub(period) {
        store
            .delete(&history_key(expired))
            .map_err(|e| error(&e.to_string()))?;
    }
    Ok(())
}
pub fn slash_record(
    store: &impl ReadonlyKVStore,
    operator: Address,
) -> Result<SlashRecord, CoreError> {
    read(
        store,
        &[b"staking/slash/".as_slice(), operator.as_bytes()].concat(),
    )
}
fn fraction(amount: Amount, bps: u16) -> Amount {
    let value = amount.as_atto();
    let rate = u128::from(bps);
    Amount::from_atto((value / 10000) * rate + ((value % 10000) * rate) / 10000)
}
pub(crate) fn slash(
    store: &impl KVStore,
    encoded: &[u8],
    height: u64,
    period: u64,
    gas: &mut GasMeter,
) -> Result<SlashRecord, CoreError> {
    if encoded.len() > 4096 {
        return Err(error("evidence exceeds size limit"));
    }
    let mut evidence: sprax_consensus::EquivocationEvidence =
        serde_json::from_slice(encoded).map_err(|e| error(&e.to_string()))?;
    if evidence.height == 0
        || evidence.height >= height
        || height - evidence.height >= period
        || !evidence.is_valid_equivocation()
        || evidence.vote_a.signature.len() != 64
        || evidence.vote_b.signature.len() != 64
    {
        return Err(error("invalid or expired equivocation evidence"));
    }
    let identity = store
        .get(GenesisConfig::IDENTITY_KEY)
        .map_err(|e| error(&e.to_string()))?
        .ok_or_else(|| error("missing genesis identity"))?;
    if evidence.vote_a.genesis.as_bytes().as_slice() != identity.as_slice() {
        return Err(error("evidence genesis mismatch"));
    }
    let historical: Vec<CanonicalValidator> = read(store, &history_key(evidence.height))?;
    let signer = historical
        .iter()
        .find(|v| v.address == evidence.validator_address)
        .ok_or_else(|| error("evidence signer was not active at infraction height"))?;
    evidence
        .verify_signatures(&signer.public_key)
        .map_err(|e| error(&e.to_string()))?;
    let mut records = registry(store)?;
    let record = records
        .iter_mut()
        .find(|v| v.operator_address == evidence.validator_address)
        .ok_or_else(|| error("unknown evidence signer"))?;
    if record.tombstoned {
        return Err(error("validator already tombstoned; no repeated penalty"));
    }
    let rate = policy(store)?.double_sign_slash_bps;
    let mut burned = Amount::ZERO;
    let mut bonded_burned = Amount::ZERO;
    let operator = evidence.validator_address;
    for (_, address) in store
        .scan_prefix(&StateAccessor::delegation_prefix(&operator))
        .map_err(|e| error(&e.to_string()))?
    {
        gas.consume_gas(1500)?;
        let delegator = Address::new(
            address
                .try_into()
                .map_err(|_| error("invalid delegation index"))?,
        );
        let mut delegation = StateAccessor::get_delegation(store, &delegator, &operator)?;
        let penalty = fraction(delegation.balance, rate);
        delegation.balance = delegation
            .balance
            .checked_sub(penalty)
            .map_err(|e| error(&e.to_string()))?;
        delegation.shares = delegation.balance;
        StateAccessor::set_delegation(store, &delegator, &operator, &delegation)?;
        bonded_burned = bonded_burned
            .checked_add(penalty)
            .map_err(|e| error(&e.to_string()))?;
    }
    let mut stake = StateAccessor::get_validator_stake(store, &operator)?;
    stake.tokens = stake
        .tokens
        .checked_sub(bonded_burned)
        .map_err(|e| error(&e.to_string()))?;
    StateAccessor::set_validator_stake(store, &operator, &stake)?;
    burned = burned
        .checked_add(bonded_burned)
        .map_err(|e| error(&e.to_string()))?;
    for (_, key) in store
        .scan_prefix(&StateAccessor::unbonding_prefix(&operator))
        .map_err(|e| error(&e.to_string()))?
    {
        gas.consume_gas(1500)?;
        let mut entry: crate::state::UnbondingRecord = read(store, &key)?;
        if entry.creation_height >= evidence.height {
            let penalty = fraction(entry.amount, rate);
            entry.amount = entry
                .amount
                .checked_sub(penalty)
                .map_err(|e| error(&e.to_string()))?;
            put(store, &key, &entry)?;
            burned = burned
                .checked_add(penalty)
                .map_err(|e| error(&e.to_string()))?;
        }
    }
    let mut supply = StateAccessor::get_supply_state(store)?;
    supply.circulating_supply = supply
        .circulating_supply
        .checked_sub(burned)
        .map_err(|e| error(&e.to_string()))?;
    supply.total_burned = supply
        .total_burned
        .checked_add(burned)
        .map_err(|e| error(&e.to_string()))?;
    StateAccessor::set_supply_state(store, &supply)?;
    record.jailed = true;
    record.tombstoned = true;
    record.jailed_until = u64::MAX;
    put(store, REGISTRY, &records)?;
    if encode(&evidence.vote_a)? > encode(&evidence.vote_b)? {
        std::mem::swap(&mut evidence.vote_a, &mut evidence.vote_b);
    }
    let receipt = SlashRecord {
        evidence_hash: Hasher::sha256(&encode(&evidence)?),
        infraction_height: evidence.height,
        processed_height: height,
        burned,
    };
    put(
        store,
        &[b"staking/slash/".as_slice(), operator.as_bytes()].concat(),
        &receipt,
    )?;
    Ok(receipt)
}
