#![cfg(feature = "redb-store")]
use sprax_storage::{KVStore, MemKVStore, OverlayStore, ReadonlyKVStore, RedbStore};

fn verify_bounds(store: impl KVStore + Clone) {
    store.set(b"a/1", b"1234").unwrap();
    store.set(b"a/2", b"5678").unwrap();
    store.set(b"z/1", &[0; 128]).unwrap();
    let expected = store.scan_range(b"a/", Some(b"b/")).unwrap();
    assert_eq!(
        store.scan_range_bounded(b"a/", Some(b"b/"), 2, 14).unwrap(),
        expected
    );
    assert!(store.scan_range_bounded(b"a/", Some(b"b/"), 1, 14).is_err());
    assert!(store.scan_range_bounded(b"a/", Some(b"b/"), 2, 13).is_err());
    let overlay = OverlayStore::new(store);
    overlay.set(b"a/2", b"other").unwrap();
    overlay.set(b"a/3", b"new").unwrap();
    assert!(overlay
        .scan_range_bounded(b"a/", Some(b"b/"), 2, 100)
        .is_err());
    assert_eq!(
        overlay
            .scan_range_bounded(b"a/", Some(b"b/"), 3, 100)
            .unwrap(),
        overlay.scan_range(b"a/", Some(b"b/")).unwrap()
    );
}

#[test]
fn memory_and_disk_enforce_the_same_scan_bounds() {
    verify_bounds(MemKVStore::new());
    let dir = tempfile::tempdir().unwrap();
    verify_bounds(RedbStore::open(&dir.path().join("store.redb")).unwrap());
}
