use sprax_core::{ChainLedger, GenesisConfig};
use sprax_storage::{MemKVStore, StateCommitment};

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
