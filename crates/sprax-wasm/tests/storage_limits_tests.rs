use cosmwasm_std::Order;
use cosmwasm_vm::Storage;
use sprax_storage::MemKVStore;
use sprax_types::Address;
use sprax_wasm::backend::{ChainStorage, MAX_ITERATORS, MAX_STORAGE_VALUE_BYTES};

#[test]
fn storage_limits_and_iterator_isolation_are_enforced() {
    let store = MemKVStore::new();
    let mut first = ChainStorage::new(store.clone(), Address::new([1; 20]), false);
    let mut other = ChainStorage::new(store, Address::new([2; 20]), false);
    first.set(b"a", b"one").0.unwrap();
    first.set(b"b", b"two").0.unwrap();
    other.set(b"a", b"private-to-other-contract").0.unwrap();
    assert!(first
        .set(b"too-big", &vec![0; MAX_STORAGE_VALUE_BYTES + 1])
        .0
        .is_err());
    assert!(first.get(b"too-big").0.unwrap().is_none());
    let id = first.scan(Some(b"b"), None, Order::Descending).0.unwrap();
    assert_eq!(
        first.next(id).0.unwrap(),
        Some((b"b".to_vec(), b"two".to_vec()))
    );
    assert!(first.next(id).0.unwrap().is_none());
    assert!(first.next(id).0.unwrap().is_none());
    for _ in 0..MAX_ITERATORS {
        first.scan(None, None, Order::Ascending).0.unwrap();
    }
    assert!(first.scan(None, None, Order::Ascending).0.is_err());
    let mut readonly = ChainStorage::new(MemKVStore::new(), Address::ZERO, true);
    assert!(readonly.set(b"a", b"b").0.is_err());
    assert!(readonly.remove(b"a").0.is_err());
}
