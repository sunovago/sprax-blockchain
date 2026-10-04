use crate::error::StorageError;
use sprax_types::Hash32;

pub type KvPairs = Vec<(Vec<u8>, Vec<u8>)>;

/// Read-only Key-Value store interface.
pub trait ReadonlyKVStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    fn has(&self, key: &[u8]) -> Result<bool, StorageError> {
        Ok(self.get(key)?.is_some())
    }

    /// Returns all key-value pairs with `start <= key < end` (or `key >= start` when `end` is
    /// `None`), sorted by key ascending. The one range primitive both backends implement
    /// natively; [`scan_prefix`](Self::scan_prefix) is defined in terms of it below.
    fn scan_range(&self, start: &[u8], end: Option<&[u8]>) -> Result<KvPairs, StorageError>;

    /// Returns all key-value pairs whose key starts with `prefix`, sorted by key ascending.
    /// Backs range/iterator host functions (e.g. CosmWasm's `db_scan`/`db_next`) that need to
    /// walk a contract's own key space without seeing unrelated keys in the same flat store.
    fn scan_prefix(&self, prefix: &[u8]) -> Result<KvPairs, StorageError> {
        self.scan_range(prefix, prefix_upper_bound(prefix).as_deref())
    }
}

/// Exclusive upper bound one past `prefix` in lexicographic byte order (`None` for an all-0xff
/// prefix, which has no such bound and scans to the end of the keyspace).
#[must_use]
pub fn prefix_upper_bound(prefix: &[u8]) -> Option<Vec<u8>> {
    let mut bound = prefix.to_vec();
    while let Some(last) = bound.last_mut() {
        if *last < 0xff {
            *last += 1;
            return Some(bound);
        }
        bound.pop();
    }
    None
}

/// Batch mutation writer for atomic multi-key updates.
pub trait BatchWriter {
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>);
    fn delete(&mut self, key: Vec<u8>);
}

/// Mutable Key-Value store interface supporting atomic batch commits.
pub trait KVStore: ReadonlyKVStore {
    fn set(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError>;
    fn delete(&self, key: &[u8]) -> Result<(), StorageError>;
    fn write_batch(&self, batch: BatchOperation) -> Result<(), StorageError>;
}

/// In-memory batch operation container.
#[derive(Debug, Clone, Default)]
pub struct BatchOperation {
    pub puts: Vec<(Vec<u8>, Vec<u8>)>,
    pub deletes: Vec<Vec<u8>>,
}

impl BatchOperation {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) {
        self.puts.push((key.into(), value.into()));
    }

    pub fn remove(&mut self, key: impl Into<Vec<u8>>) {
        self.deletes.push(key.into());
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.puts.is_empty() && self.deletes.is_empty()
    }
}

impl BatchWriter for BatchOperation {
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.insert(key, value);
    }

    fn delete(&mut self, key: Vec<u8>) {
        self.remove(key);
    }
}

/// Cryptographic state commitment calculator trait (Merkle Root).
pub trait StateCommitment {
    fn compute_root(&self) -> Result<Hash32, StorageError>;
}

/// Chain metadata storage: finalized blocks, height, and tx index — kept separate from
/// [`KVStore`] so that [`StateCommitment::compute_root`] only ever hashes account/contract
/// state, never block/tx bookkeeping data.
pub trait ChainMetaStore {
    /// Atomically commits state and finalized-chain metadata together.
    fn commit_chain_batch(&self, batch: ChainWriteBatch) -> Result<(), StorageError>;
    fn put_height(&self, height: u64) -> Result<(), StorageError>;
    fn get_height(&self) -> Result<Option<u64>, StorageError>;
    fn put_block(&self, height: u64, block_bytes: &[u8]) -> Result<(), StorageError>;
    fn get_block(&self, height: u64) -> Result<Option<Vec<u8>>, StorageError>;
    fn put_block_hash_index(&self, hash: Hash32, height: u64) -> Result<(), StorageError>;
    fn get_height_by_hash(&self, hash: Hash32) -> Result<Option<u64>, StorageError>;
    fn put_tx_index(&self, tx_hash: Hash32, entry_bytes: &[u8]) -> Result<(), StorageError>;
    fn get_tx_index(&self, tx_hash: Hash32) -> Result<Option<Vec<u8>>, StorageError>;
}

#[derive(Debug, Clone, Default)]
pub struct ChainWriteBatch {
    pub state: BatchOperation,
    pub height: Option<u64>,
    pub blocks: Vec<(u64, Vec<u8>)>,
    pub block_hashes: Vec<(Hash32, u64)>,
    pub transactions: Vec<(Hash32, Vec<u8>)>,
}
