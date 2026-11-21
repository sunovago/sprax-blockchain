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
    fn scan_range_bounded(
        &self,
        start: &[u8],
        end: Option<&[u8]>,
        max_records: usize,
        max_bytes: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, StorageError> {
        let guard = self.inner.read();
        let mut pairs = Vec::new();
        let mut bytes = 0usize;
        for (key, value) in guard.range(start.to_vec()..) {
            if end.is_some_and(|end| key.as_slice() >= end) {
                break;
            }
            bytes = bytes.saturating_add(key.len()).saturating_add(value.len());
            if pairs.len() >= max_records || bytes > max_bytes {
                return Err(StorageError::DatabaseError(
                    "scan resource limit exceeded".into(),
                ));
            }
            pairs.push((key.clone(), value.clone()));
        }
        Ok(pairs)
    }
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let guard = self.inner.read();
        Ok(guard.get(key).cloned())
    }

    fn scan_range(
        &self,
        start: &[u8],
        end: Option<&[u8]>,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, StorageError> {
        let guard = self.inner.read();
        let iter = guard.range(start.to_vec()..);
        let pairs = match end {
            Some(end) => iter
                .take_while(|(k, _)| k.as_slice() < end)
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            None => iter.map(|(k, v)| (k.clone(), v.clone())).collect(),
        };
        Ok(pairs)
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
const EARLIEST_HEIGHT_KEY: &[u8] = b"meta:earliest_height";
const BLOCK_PREFIX: &[u8] = b"blk:";
const BLOCK_HASH_PREFIX: &[u8] = b"blkhash:";
const TX_PREFIX: &[u8] = b"tx:";

impl ChainMetaStore for MemKVStore {
    fn commit_chain_batch(&self, batch: crate::ChainWriteBatch) -> Result<(), StorageError> {
        let mut state = self.inner.write();
        let mut meta = self.meta.write();
        for (k, v) in batch.state.puts {
            state.insert(k, v);
        }
        for k in batch.state.deletes {
            state.remove(&k);
        }
        for (h, bytes) in batch.blocks {
            meta.insert(u64_key(BLOCK_PREFIX, h), bytes);
        }
        for (hash, h) in batch.block_hashes {
            meta.insert(hash_key(BLOCK_HASH_PREFIX, hash), h.to_be_bytes().to_vec());
        }
        for (hash, bytes) in batch.transactions {
            meta.insert(hash_key(TX_PREFIX, hash), bytes);
        }
        if let Some(h) = batch.height {
            meta.insert(HEIGHT_KEY.to_vec(), h.to_be_bytes().to_vec());
        }
        Ok(())
    }
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

    fn get_earliest_height(&self) -> Result<Option<u64>, StorageError> {
        Ok(self.meta.read().get(EARLIEST_HEIGHT_KEY).map(|bytes| {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(bytes);
            u64::from_be_bytes(arr)
        }))
    }

    fn prune_history(
        &self,
        strategy: crate::pruning::PruningStrategy,
        current_height: u64,
    ) -> Result<crate::pruning::PruningStats, StorageError> {
        let cutoff = strategy.cutoff_height(current_height);
        let current_earliest = self.get_earliest_height()?.unwrap_or(0);
        if cutoff == 0 || current_earliest >= cutoff {
            return Ok(crate::pruning::PruningStats {
                blocks_pruned: 0,
                tx_indices_pruned: 0,
                earliest_retained_height: current_earliest,
            });
        }

        let mut meta = self.meta.write();
        let mut blocks_pruned = 0;
        let mut tx_indices_pruned = 0;

        for h in current_earliest..cutoff {
            let blk_k = u64_key(BLOCK_PREFIX, h);
            if let Some(bytes) = meta.remove(&blk_k) {
                blocks_pruned += 1;
                if let Ok(block) = serde_json::from_slice::<sprax_types::Block>(&bytes) {
                    if let Ok(block_hash) = sprax_crypto::Hasher::block_hash(&block.header) {
                        meta.remove(&hash_key(BLOCK_HASH_PREFIX, block_hash));
                    }
                    for tx in block.body.transactions {
                        if let Ok(tx_hash) = sprax_crypto::Hasher::tx_hash(&tx) {
                            if meta.remove(&hash_key(TX_PREFIX, tx_hash)).is_some() {
                                tx_indices_pruned += 1;
                            }
                        }
                    }
                }
            }
        }

        meta.insert(EARLIEST_HEIGHT_KEY.to_vec(), cutoff.to_be_bytes().to_vec());

        Ok(crate::pruning::PruningStats {
            blocks_pruned,
            tx_indices_pruned,
            earliest_retained_height: cutoff,
        })
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
    fn test_mem_kv_store_scan_prefix() {
        let store = MemKVStore::new();
        store.set(b"c:aaa:k1", b"v1").unwrap();
        store.set(b"c:aaa:k2", b"v2").unwrap();
        store.set(b"c:bbb:k1", b"other").unwrap();

        let scanned = store.scan_prefix(b"c:aaa:").unwrap();
        assert_eq!(
            scanned,
            vec![
                (b"c:aaa:k1".to_vec(), b"v1".to_vec()),
                (b"c:aaa:k2".to_vec(), b"v2".to_vec()),
            ]
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

    #[test]
    fn test_mem_kv_store_prune_history() {
        let store = MemKVStore::new();

        for h in 0..=8 {
            let block = sprax_types::Block {
                header: sprax_types::BlockHeader {
                    version: 1,
                    chain_id: "test".into(),
                    height: h,
                    timestamp_unix_secs: 1000 + h,
                    parent_hash: Hash32::ZERO,
                    proposer: sprax_types::Address::ZERO,
                    state_root: Hash32::ZERO,
                    txs_root: Hash32::ZERO,
                    receipts_root: Hash32::ZERO,
                    validator_set_hash: Hash32::ZERO,
                },
                body: sprax_types::BlockBody::default(),
                last_commit: vec![],
            };
            let block_bytes = serde_json::to_vec(&block).unwrap();
            let block_hash = sprax_crypto::Hasher::block_hash(&block.header).unwrap();
            store.put_block(h, &block_bytes).unwrap();
            store.put_block_hash_index(block_hash, h).unwrap();
        }
        store.put_height(8).unwrap();

        let stats = store
            .prune_history(crate::PruningStrategy::KeepRecent { keep_blocks: 2 }, 8)
            .unwrap();

        assert_eq!(stats.blocks_pruned, 6);
        assert_eq!(stats.earliest_retained_height, 6);
        assert_eq!(store.get_earliest_height().unwrap(), Some(6));

        for h in 0..6 {
            assert!(store.get_block(h).unwrap().is_none());
        }
        for h in 6..=8 {
            assert!(store.get_block(h).unwrap().is_some());
        }
    }
}
