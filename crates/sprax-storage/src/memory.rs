use crate::{
    error::StorageError,
    roots::compute_flat_root,
    traits::{BatchOperation, ChainMetaStore, KVStore, ReadonlyKVStore, StateCommitment},
};
use parking_lot::RwLock;
use sprax_types::Hash32;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, Default)]
pub struct MemKVStore {
    inner: Arc<RwLock<BTreeMap<Vec<u8>, Vec<u8>>>>,
    meta: Arc<RwLock<BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl MemKVStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(BTreeMap::new())),
            meta: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }
}

impl ReadonlyKVStore for MemKVStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let guard = self.inner.read();
        Ok(guard.get(key).cloned())
    }
}

impl KVStore for MemKVStore {
    fn set(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let mut guard = self.inner.write();
        guard.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<(), StorageError> {
        let mut guard = self.inner.write();
        guard.remove(key);
        Ok(())
    }

    fn write_batch(&self, batch: BatchOperation) -> Result<(), StorageError> {
        let mut guard = self.inner.write();
        for (k, v) in batch.puts {
            guard.insert(k, v);
        }
        for k in batch.deletes {
            guard.remove(&k);
        }
        Ok(())
    }
}

impl StateCommitment for MemKVStore {
    /// Computes deterministic Blake3 Merkle Root over sorted key-values.
    fn compute_root(&self) -> Result<Hash32, StorageError> {
        let guard = self.inner.read();
        let pairs = guard.iter().map(|(k, v)| (k.as_slice(), v.as_slice()));
        Ok(compute_flat_root(pairs))
    }
}

fn u64_key(prefix: &[u8], height: u64) -> Vec<u8> {
    let mut k = Vec::with_capacity(prefix.len() + 8);
    k.extend_from_slice(prefix);
    k.extend_from_slice(&height.to_be_bytes());
    k
}

fn hash_key(prefix: &[u8], hash: Hash32) -> Vec<u8> {
    let mut k = Vec::with_capacity(prefix.len() + 32);
    k.extend_from_slice(prefix);
    k.extend_from_slice(hash.as_bytes());
    k
}

const HEIGHT_KEY: &[u8] = b"meta:height";
const BLOCK_PREFIX: &[u8] = b"blk:";
const BLOCK_HASH_PREFIX: &[u8] = b"blkhash:";
const TX_PREFIX: &[u8] = b"tx:";

impl ChainMetaStore for MemKVStore {
    fn put_height(&self, height: u64) -> Result<(), StorageError> {
        self.meta
            .write()
            .insert(HEIGHT_KEY.to_vec(), height.to_be_bytes().to_vec());
        Ok(())
    }

    fn get_height(&self) -> Result<Option<u64>, StorageError> {
        Ok(self.meta.read().get(HEIGHT_KEY).map(|bytes| {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(bytes);
            u64::from_be_bytes(arr)
        }))
    }

    fn put_block(&self, height: u64, block_bytes: &[u8]) -> Result<(), StorageError> {
        self.meta
            .write()
            .insert(u64_key(BLOCK_PREFIX, height), block_bytes.to_vec());
        Ok(())
    }

    fn get_block(&self, height: u64) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(self
            .meta
            .read()
            .get(&u64_key(BLOCK_PREFIX, height))
            .cloned())
    }

    fn put_block_hash_index(&self, hash: Hash32, height: u64) -> Result<(), StorageError> {
        self.meta.write().insert(
            hash_key(BLOCK_HASH_PREFIX, hash),
            height.to_be_bytes().to_vec(),
        );
        Ok(())
    }

    fn get_height_by_hash(&self, hash: Hash32) -> Result<Option<u64>, StorageError> {
        Ok(self
            .meta
            .read()
            .get(&hash_key(BLOCK_HASH_PREFIX, hash))
            .map(|bytes| {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(bytes);
                u64::from_be_bytes(arr)
            }))
    }

    fn put_tx_index(&self, tx_hash: Hash32, entry_bytes: &[u8]) -> Result<(), StorageError> {
        self.meta
            .write()
            .insert(hash_key(TX_PREFIX, tx_hash), entry_bytes.to_vec());
        Ok(())
    }

    fn get_tx_index(&self, tx_hash: Hash32) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(self.meta.read().get(&hash_key(TX_PREFIX, tx_hash)).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mem_kv_store_crud_and_batch() {
        let store = MemKVStore::new();
        assert!(store.is_empty());

        store.set(b"account:alice", b"1000").unwrap();
        assert_eq!(store.get(b"account:alice").unwrap(), Some(b"1000".to_vec()));
        assert_eq!(store.len(), 1);

        let mut batch = BatchOperation::new();
        batch.insert(b"account:bob".to_vec(), b"500".to_vec());
        batch.insert(b"account:charlie".to_vec(), b"250".to_vec());
        batch.remove(b"account:alice".to_vec());
        store.write_batch(batch).unwrap();

        assert_eq!(store.get(b"account:alice").unwrap(), None);
        assert_eq!(store.get(b"account:bob").unwrap(), Some(b"500".to_vec()));
        assert_eq!(
            store.get(b"account:charlie").unwrap(),
            Some(b"250".to_vec())
        );
    }

    #[test]
    fn test_mem_kv_store_state_root_determinism() {
        let s1 = MemKVStore::new();
        let s2 = MemKVStore::new();

        s1.set(b"k1", b"v1").unwrap();
        s1.set(b"k2", b"v2").unwrap();

        s2.set(b"k2", b"v2").unwrap();
        s2.set(b"k1", b"v1").unwrap();

        assert_eq!(s1.compute_root().unwrap(), s2.compute_root().unwrap());
    }
}
