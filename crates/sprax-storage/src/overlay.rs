//! Isolated writes for transaction execution and block validation. Dropping an
//! overlay discards every pending change; committing uses one backing-store batch.
use crate::{
    compute_flat_root, BatchOperation, ChainMetaStore, ChainWriteBatch, KVStore, ReadonlyKVStore,
    StateCommitment, StorageError,
};
use parking_lot::RwLock;
use sprax_types::Hash32;
use std::{collections::BTreeMap, sync::Arc};

type PendingWrites = BTreeMap<Vec<u8>, Option<Vec<u8>>>;

#[derive(Debug, Clone)]
pub struct OverlayStore<S> {
    base: S,
    writes: Arc<RwLock<PendingWrites>>,
    meta: Arc<RwLock<ChainWriteBatch>>,
}

impl<S: KVStore> OverlayStore<S> {
    pub fn new(base: S) -> Self {
        Self {
            base,
            writes: Arc::default(),
            meta: Arc::default(),
        }
    }

    fn state_batch(&self) -> BatchOperation {
        let mut batch = BatchOperation::new();
        for (key, value) in self.writes.read().iter() {
            match value {
                Some(value) => batch.insert(key.clone(), value.clone()),
                None => batch.remove(key.clone()),
            }
        }
        batch
    }

    pub fn commit_state(&self) -> Result<(), StorageError> {
        self.base.write_batch(self.state_batch())
    }
}

impl<S: KVStore + ChainMetaStore> OverlayStore<S> {
    pub fn commit_chain(&self) -> Result<(), StorageError> {
        let mut batch = self.meta.read().clone();
        batch.state = self.state_batch();
        self.base.commit_chain_batch(batch)
    }
}

impl<S: KVStore> ReadonlyKVStore for OverlayStore<S> {
    fn scan_range_bounded(
        &self,
        start: &[u8],
        end: Option<&[u8]>,
        max_records: usize,
        max_bytes: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, StorageError> {
        let base = self
            .base
            .scan_range_bounded(start, end, max_records, max_bytes)?;
        let mut bytes: usize = base
            .iter()
            .map(|(key, value)| key.len() + value.len())
            .sum();
        let mut merged: BTreeMap<_, _> = base.into_iter().collect();
        for (key, value) in self.writes.read().range(start.to_vec()..) {
            if end.is_some_and(|end| key.as_slice() >= end) {
                break;
            }
            if let Some(previous) = merged.remove(key) {
                bytes -= key.len() + previous.len();
            }
            if let Some(value) = value {
                bytes = bytes.saturating_add(key.len()).saturating_add(value.len());
                if merged.len() >= max_records || bytes > max_bytes {
                    return Err(StorageError::DatabaseError(
                        "scan resource limit exceeded".into(),
                    ));
                }
                merged.insert(key.clone(), value.clone());
            }
        }
        Ok(merged.into_iter().collect())
    }
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        if let Some(value) = self.writes.read().get(key) {
            return Ok(value.clone());
        }
        self.base.get(key)
    }

    fn scan_range(
        &self,
        start: &[u8],
        end: Option<&[u8]>,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, StorageError> {
        let mut merged: BTreeMap<_, _> = self.base.scan_range(start, end)?.into_iter().collect();
        for (key, value) in self.writes.read().iter() {
            if key.as_slice() < start || end.is_some_and(|end| key.as_slice() >= end) {
                continue;
            }
            match value {
                Some(value) => {
                    merged.insert(key.clone(), value.clone());
                }
                None => {
                    merged.remove(key);
                }
            }
        }
        Ok(merged.into_iter().collect())
    }
}

impl<S: KVStore> KVStore for OverlayStore<S> {
    fn set(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.writes
            .write()
            .insert(key.to_vec(), Some(value.to_vec()));
        Ok(())
    }
    fn delete(&self, key: &[u8]) -> Result<(), StorageError> {
        self.writes.write().insert(key.to_vec(), None);
        Ok(())
    }
    fn write_batch(&self, batch: BatchOperation) -> Result<(), StorageError> {
        let mut writes = self.writes.write();
        for (key, value) in batch.puts {
            writes.insert(key, Some(value));
        }
        for key in batch.deletes {
            writes.insert(key, None);
        }
        Ok(())
    }
}

impl<S: KVStore> StateCommitment for OverlayStore<S> {
    fn compute_root(&self) -> Result<Hash32, StorageError> {
        let pairs = self.scan_range(b"", None)?;
        Ok(compute_flat_root(
            pairs.iter().map(|(k, v)| (k.as_slice(), v.as_slice())),
        ))
    }
}

impl<S: KVStore + ChainMetaStore> ChainMetaStore for OverlayStore<S> {
    fn commit_chain_batch(&self, batch: ChainWriteBatch) -> Result<(), StorageError> {
        self.write_batch(batch.state)?;
        let mut meta = self.meta.write();
        if batch.height.is_some() {
            meta.height = batch.height;
        }
        meta.blocks.extend(batch.blocks);
        meta.block_hashes.extend(batch.block_hashes);
        meta.transactions.extend(batch.transactions);
        Ok(())
    }
    fn put_height(&self, height: u64) -> Result<(), StorageError> {
        self.meta.write().height = Some(height);
        Ok(())
    }
    fn get_height(&self) -> Result<Option<u64>, StorageError> {
        match self.meta.read().height {
            Some(h) => Ok(Some(h)),
            None => self.base.get_height(),
        }
    }
    fn put_block(&self, height: u64, bytes: &[u8]) -> Result<(), StorageError> {
        self.meta.write().blocks.push((height, bytes.to_vec()));
        Ok(())
    }
    fn get_block(&self, height: u64) -> Result<Option<Vec<u8>>, StorageError> {
        if let Some((_, bytes)) = self
            .meta
            .read()
            .blocks
            .iter()
            .rev()
            .find(|(h, _)| *h == height)
        {
            return Ok(Some(bytes.clone()));
        }
        self.base.get_block(height)
    }
    fn put_block_hash_index(&self, hash: Hash32, height: u64) -> Result<(), StorageError> {
        self.meta.write().block_hashes.push((hash, height));
        Ok(())
    }
    fn get_height_by_hash(&self, hash: Hash32) -> Result<Option<u64>, StorageError> {
        if let Some((_, height)) = self
            .meta
            .read()
            .block_hashes
            .iter()
            .rev()
            .find(|(h, _)| *h == hash)
        {
            return Ok(Some(*height));
        }
        self.base.get_height_by_hash(hash)
    }
    fn put_tx_index(&self, hash: Hash32, bytes: &[u8]) -> Result<(), StorageError> {
        self.meta.write().transactions.push((hash, bytes.to_vec()));
        Ok(())
    }
    fn get_tx_index(&self, hash: Hash32) -> Result<Option<Vec<u8>>, StorageError> {
        if let Some((_, bytes)) = self
            .meta
            .read()
            .transactions
            .iter()
            .rev()
            .find(|(h, _)| *h == hash)
        {
            return Ok(Some(bytes.clone()));
        }
        self.base.get_tx_index(hash)
    }
}
