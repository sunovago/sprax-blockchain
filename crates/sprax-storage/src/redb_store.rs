//! Persistent embedded key-value store backed by `redb` (pure-Rust, no native toolchain
//! required). Used in place of RocksDB, which needs `cmake` + `libclang`/LLVM that are not
//! available in this development environment (see docs/ROADMAP.md storage notes).

use crate::{
    error::StorageError,
    roots::compute_flat_root,
    traits::{BatchOperation, ChainMetaStore, KVStore, ReadonlyKVStore, StateCommitment},
};
use redb::{Database, ReadableTable, TableDefinition};
use sprax_types::Hash32;
use std::{path::Path, sync::Arc};

const STATE_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("state");
const CHAIN_META_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("chain_meta");

const HEIGHT_KEY: &[u8] = b"meta:height";
const BLOCK_PREFIX: &[u8] = b"blk:";
const BLOCK_HASH_PREFIX: &[u8] = b"blkhash:";
const TX_PREFIX: &[u8] = b"tx:";

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

fn db_err(e: impl std::fmt::Display) -> StorageError {
    StorageError::DatabaseError(e.to_string())
}

#[derive(Debug, Clone)]
pub struct RedbStore {
    db: Arc<Database>,
}

impl RedbStore {
    /// Opens (creating if missing) a `redb` database file at `path` and ensures both tables exist.
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(db_err)?;
        }
        let db = Database::create(path).map_err(db_err)?;
        // Opening a table for write creates it if it doesn't exist yet — do this once up
        // front so a fresh database doesn't error on the first read-only access.
        let write_txn = db.begin_write().map_err(db_err)?;
        {
            let _ = write_txn.open_table(STATE_TABLE).map_err(db_err)?;
            let _ = write_txn.open_table(CHAIN_META_TABLE).map_err(db_err)?;
        }
        write_txn.commit().map_err(db_err)?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Prunes historical block metadata according to the configured [`crate::PruningStrategy`].
    /// Returns the number of historical blocks pruned.
    pub fn prune_history(
        &self,
        strategy: crate::PruningStrategy,
        current_height: u64,
    ) -> Result<usize, StorageError> {
        let keep_blocks = match strategy {
            crate::PruningStrategy::Archive => return Ok(0),
            crate::PruningStrategy::KeepRecent { keep_blocks } => keep_blocks,
            crate::PruningStrategy::Aggressive { keep_blocks } => keep_blocks,
        };

        let cutoff = current_height.saturating_sub(keep_blocks);
        if cutoff == 0 {
            return Ok(0);
        }

        let mut pruned = 0;
        let write_txn = self.db.begin_write().map_err(db_err)?;
        {
            let mut table = write_txn.open_table(CHAIN_META_TABLE).map_err(db_err)?;
            for h in 1..cutoff {
                let key = u64_key(BLOCK_PREFIX, h);
                if table.remove(key.as_slice()).map_err(db_err)?.is_some() {
                    pruned += 1;
                }
            }
        }
        write_txn.commit().map_err(db_err)?;
        Ok(pruned)
    }
}

impl ReadonlyKVStore for RedbStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let read_txn = self.db.begin_read().map_err(db_err)?;
        let table = read_txn.open_table(STATE_TABLE).map_err(db_err)?;
        Ok(table
            .get(key)
            .map_err(db_err)?
            .map(|guard| guard.value().to_vec()))
    }
}

impl KVStore for RedbStore {
    fn set(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let write_txn = self.db.begin_write().map_err(db_err)?;
        {
            let mut table = write_txn.open_table(STATE_TABLE).map_err(db_err)?;
            table.insert(key, value).map_err(db_err)?;
        }
        write_txn.commit().map_err(db_err)?;
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<(), StorageError> {
        let write_txn = self.db.begin_write().map_err(db_err)?;
        {
            let mut table = write_txn.open_table(STATE_TABLE).map_err(db_err)?;
            table.remove(key).map_err(db_err)?;
        }
        write_txn.commit().map_err(db_err)?;
        Ok(())
    }

    fn write_batch(&self, batch: BatchOperation) -> Result<(), StorageError> {
        let write_txn = self.db.begin_write().map_err(db_err)?;
        {
            let mut table = write_txn.open_table(STATE_TABLE).map_err(db_err)?;
            for (k, v) in &batch.puts {
                table.insert(k.as_slice(), v.as_slice()).map_err(db_err)?;
            }
            for k in &batch.deletes {
                table.remove(k.as_slice()).map_err(db_err)?;
            }
        }
        write_txn.commit().map_err(db_err)?;
        Ok(())
    }
}

impl StateCommitment for RedbStore {
    fn compute_root(&self) -> Result<Hash32, StorageError> {
        let read_txn = self.db.begin_read().map_err(db_err)?;
        let table = read_txn.open_table(STATE_TABLE).map_err(db_err)?;
        let mut owned: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        for item in table.iter().map_err(db_err)? {
            let (k, v) = item.map_err(db_err)?;
            owned.push((k.value().to_vec(), v.value().to_vec()));
        }
        let pairs = owned.iter().map(|(k, v)| (k.as_slice(), v.as_slice()));
        Ok(compute_flat_root(pairs))
    }
}

impl ChainMetaStore for RedbStore {
    fn put_height(&self, height: u64) -> Result<(), StorageError> {
        let write_txn = self.db.begin_write().map_err(db_err)?;
        {
            let mut table = write_txn.open_table(CHAIN_META_TABLE).map_err(db_err)?;
            table
                .insert(HEIGHT_KEY, height.to_be_bytes().as_slice())
                .map_err(db_err)?;
        }
        write_txn.commit().map_err(db_err)?;
        Ok(())
    }

    fn get_height(&self) -> Result<Option<u64>, StorageError> {
        let read_txn = self.db.begin_read().map_err(db_err)?;
        let table = read_txn.open_table(CHAIN_META_TABLE).map_err(db_err)?;
        Ok(table.get(HEIGHT_KEY).map_err(db_err)?.map(|guard| {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(guard.value());
            u64::from_be_bytes(arr)
        }))
    }

    fn put_block(&self, height: u64, block_bytes: &[u8]) -> Result<(), StorageError> {
        let key = u64_key(BLOCK_PREFIX, height);
        let write_txn = self.db.begin_write().map_err(db_err)?;
        {
            let mut table = write_txn.open_table(CHAIN_META_TABLE).map_err(db_err)?;
            table.insert(key.as_slice(), block_bytes).map_err(db_err)?;
        }
        write_txn.commit().map_err(db_err)?;
        Ok(())
    }

    fn get_block(&self, height: u64) -> Result<Option<Vec<u8>>, StorageError> {
        let key = u64_key(BLOCK_PREFIX, height);
        let read_txn = self.db.begin_read().map_err(db_err)?;
        let table = read_txn.open_table(CHAIN_META_TABLE).map_err(db_err)?;
        Ok(table
            .get(key.as_slice())
            .map_err(db_err)?
            .map(|guard| guard.value().to_vec()))
    }

    fn put_block_hash_index(&self, hash: Hash32, height: u64) -> Result<(), StorageError> {
        let key = hash_key(BLOCK_HASH_PREFIX, hash);
        let write_txn = self.db.begin_write().map_err(db_err)?;
        {
            let mut table = write_txn.open_table(CHAIN_META_TABLE).map_err(db_err)?;
            table
                .insert(key.as_slice(), height.to_be_bytes().as_slice())
                .map_err(db_err)?;
        }
        write_txn.commit().map_err(db_err)?;
        Ok(())
    }

    fn get_height_by_hash(&self, hash: Hash32) -> Result<Option<u64>, StorageError> {
        let key = hash_key(BLOCK_HASH_PREFIX, hash);
        let read_txn = self.db.begin_read().map_err(db_err)?;
        let table = read_txn.open_table(CHAIN_META_TABLE).map_err(db_err)?;
        Ok(table.get(key.as_slice()).map_err(db_err)?.map(|guard| {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(guard.value());
            u64::from_be_bytes(arr)
        }))
    }

    fn put_tx_index(&self, tx_hash: Hash32, entry_bytes: &[u8]) -> Result<(), StorageError> {
        let key = hash_key(TX_PREFIX, tx_hash);
        let write_txn = self.db.begin_write().map_err(db_err)?;
        {
            let mut table = write_txn.open_table(CHAIN_META_TABLE).map_err(db_err)?;
            table.insert(key.as_slice(), entry_bytes).map_err(db_err)?;
        }
        write_txn.commit().map_err(db_err)?;
        Ok(())
    }

    fn get_tx_index(&self, tx_hash: Hash32) -> Result<Option<Vec<u8>>, StorageError> {
        let key = hash_key(TX_PREFIX, tx_hash);
        let read_txn = self.db.begin_read().map_err(db_err)?;
        let table = read_txn.open_table(CHAIN_META_TABLE).map_err(db_err)?;
        Ok(table
            .get(key.as_slice())
            .map_err(db_err)?
            .map(|guard| guard.value().to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemKVStore;

    fn temp_store() -> (RedbStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = RedbStore::open(&dir.path().join("state.redb")).unwrap();
        (store, dir)
    }

    #[test]
    fn test_redb_store_crud_and_batch() {
        let (store, _dir) = temp_store();

        store.set(b"account:alice", b"1000").unwrap();
        assert_eq!(store.get(b"account:alice").unwrap(), Some(b"1000".to_vec()));

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
    fn test_redb_store_state_root_determinism() {
        let (s1, _d1) = temp_store();
        let (s2, _d2) = temp_store();

        s1.set(b"k1", b"v1").unwrap();
        s1.set(b"k2", b"v2").unwrap();

        s2.set(b"k2", b"v2").unwrap();
        s2.set(b"k1", b"v1").unwrap();

        assert_eq!(s1.compute_root().unwrap(), s2.compute_root().unwrap());
    }

    #[test]
    fn test_redb_and_mem_produce_identical_state_root() {
        let (redb, _dir) = temp_store();
        let mem = MemKVStore::new();

        for (k, v) in [
            (b"account:alice".as_slice(), b"1000".as_slice()),
            (b"account:bob", b"500"),
            (b"account:charlie", b"250"),
        ] {
            redb.set(k, v).unwrap();
            mem.set(k, v).unwrap();
        }

        assert_eq!(redb.compute_root().unwrap(), mem.compute_root().unwrap());
    }

    #[test]
    fn test_redb_store_chain_meta() {
        let (store, _dir) = temp_store();

        assert_eq!(store.get_height().unwrap(), None);
        store.put_height(5).unwrap();
        assert_eq!(store.get_height().unwrap(), Some(5));

        let block_bytes = b"fake-block-bytes";
        store.put_block(5, block_bytes).unwrap();
        assert_eq!(store.get_block(5).unwrap(), Some(block_bytes.to_vec()));

        let hash = Hash32::ZERO;
        store.put_block_hash_index(hash, 5).unwrap();
        assert_eq!(store.get_height_by_hash(hash).unwrap(), Some(5));

        let tx_hash = Hash32::ZERO;
        store.put_tx_index(tx_hash, b"entry").unwrap();
        assert_eq!(
            store.get_tx_index(tx_hash).unwrap(),
            Some(b"entry".to_vec())
        );
    }

    #[test]
    fn test_redb_store_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.redb");
        {
            let store = RedbStore::open(&path).unwrap();
            store.set(b"k", b"v").unwrap();
            store.put_height(3).unwrap();
        }
        {
            let store = RedbStore::open(&path).unwrap();
            assert_eq!(store.get(b"k").unwrap(), Some(b"v".to_vec()));
            assert_eq!(store.get_height().unwrap(), Some(3));
        }
    }
}
