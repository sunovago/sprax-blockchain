use sprax_core::{ChainLedger, GenesisConfig};
use sprax_core::{GenesisValidator, StateAccessor};
use sprax_storage::{MemKVStore, StateCommitment};
use sprax_types::Amount;

#[test]
fn genesis_self_stake_is_funded_and_supply_is_conserved() {
    let mut genesis = GenesisConfig::default_development();
    let operator = genesis.accounts[0].address;
    let stake = Amount::from_sprx_whole(100_000).unwrap();
    genesis.validators.push(GenesisValidator {
        operator_address: operator,
        consensus_pubkey: vec![7; 32],
        self_stake: stake,
        moniker: "validator".into(),
    });
    let store = MemKVStore::new();
    genesis.initialize_state(&store).unwrap();
    let account = StateAccessor::get_account(&store, &operator).unwrap();
    let delegation = StateAccessor::get_delegation(&store, &operator, &operator).unwrap();
    let validator = StateAccessor::get_validator_stake(&store, &operator).unwrap();
    assert_eq!(
        account.balance.checked_add(delegation.balance).unwrap(),
        genesis.accounts[0].initial_balance
    );
    assert_eq!(delegation.balance, stake);
    assert_eq!(delegation.shares, stake);
    assert_eq!(validator.tokens, stake);
    let issued = genesis.accounts.iter().fold(Amount::ZERO, |sum, account| {
        sum.checked_add(account.initial_balance).unwrap()
    });
    assert_eq!(
        StateAccessor::get_supply_state(&store)
            .unwrap()
            .circulating_supply,
        issued
    );
    genesis.validators[0].self_stake = Amount::from_sprx_whole(1_000_001).unwrap();
    let rejected = MemKVStore::new();
    let root = rejected.compute_root().unwrap();
    assert!(genesis.initialize_state(&rejected).is_err());
    assert_eq!(rejected.compute_root().unwrap(), root);
    genesis.validators[0].self_stake = stake;
    genesis.validators.push(genesis.validators[0].clone());
    assert!(genesis.initialize_state(&rejected).is_err());
    assert_eq!(rejected.compute_root().unwrap(), root);
}

#[test]
fn reopening_rejects_changed_genesis_and_preserves_existing_state() {
    let genesis = GenesisConfig::default_development();
    let store = MemKVStore::new();
    let mut ledger =
        ChainLedger::init_from_genesis_with_store(genesis.clone(), store.clone()).unwrap();
    ledger.mine_block(genesis.accounts[0].address).unwrap();
    let root = store.compute_root().unwrap();
    drop(ledger);
    let mut modified = genesis.clone();
    modified.consensus_params.min_gas_price_atto += 1;
    assert!(ChainLedger::open_or_init(store.clone(), || Ok(modified)).is_err());
    assert!(ChainLedger::init_from_genesis_with_store(genesis.clone(), store.clone()).is_err());
    assert_eq!(store.compute_root().unwrap(), root);
    let reopened = ChainLedger::open_or_init(store, || Ok(genesis)).unwrap();
    assert_eq!(reopened.height(), 1);
    assert_eq!(reopened.state_root().unwrap(), root);
}

#[test]
fn invalid_allocations_do_not_persist_partial_genesis() {
    let mut genesis = GenesisConfig::default_development();
    genesis.accounts.push(genesis.accounts[0].clone());
    let store = MemKVStore::new();
    let root = store.compute_root().unwrap();
    assert!(ChainLedger::init_from_genesis_with_store(genesis, store.clone()).is_err());
    assert_eq!(store.compute_root().unwrap(), root);
}
