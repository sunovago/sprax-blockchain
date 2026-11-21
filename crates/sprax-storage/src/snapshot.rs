use crate::{
    error::StorageError,
    traits::{ChainMetaStore, KVStore, ReadonlyKVStore},
};
use serde::{Deserialize, Serialize};
use sprax_types::Hash32;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

pub const SNAPSHOT_MANIFEST_NAME: &str = "manifest.json";
pub const RESTORED_SIGNING_DISABLED_MARKER: &str = "RESTORED_SIGNING_DISABLED";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotFileEntry {
    pub relative_path: String,
    pub size_bytes: u64,
    pub sha256_checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub format_version: u32,
    pub chain_id: String,
    pub genesis_fingerprint: Hash32,
    pub finalized_height: u64,
    pub finalized_block_hash: Hash32,
    pub state_root: Hash32,
    pub files: Vec<SnapshotFileEntry>,
    pub created_at_unix_secs: u64,
}

#[derive(Debug, Clone)]
pub struct SnapshotMetadata {
    pub chain_id: String,
    pub genesis_fingerprint: Hash32,
    pub finalized_height: u64,
    pub finalized_block_hash: Hash32,
    pub state_root: Hash32,
}

fn compute_file_sha256(path: &Path) -> Result<(u64, String), StorageError> {
    let mut file = File::open(path).map_err(|e| StorageError::IoError(e.to_string()))?;
    let mut buffer = [0u8; 64 * 1024];
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    let mut total_bytes = 0u64;
    loop {
        let n = file
            .read(&mut buffer)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        total_bytes += n as u64;
    }
    let digest = hex::encode(hasher.finalize());
    Ok((total_bytes, digest))
}

/// Creates a consistent verified snapshot of `source_data_dir` into `target_snapshot_dir`.
pub fn create_backup(
    source_data_dir: &Path,
    target_snapshot_dir: &Path,
    meta: SnapshotMetadata,
) -> Result<SnapshotManifest, StorageError> {
    if !source_data_dir.exists() {
        return Err(StorageError::DatabaseError(
            "source directory does not exist".into(),
        ));
    }
    fs::create_dir_all(target_snapshot_dir).map_err(|e| StorageError::IoError(e.to_string()))?;

    let mut file_entries = Vec::new();
    let entries =
        fs::read_dir(source_data_dir).map_err(|e| StorageError::IoError(e.to_string()))?;

    for entry in entries {
        let entry = entry.map_err(|e| StorageError::IoError(e.to_string()))?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| StorageError::IoError("invalid file name".into()))?;

            // Skip volatile sockets, lockfiles, or existing backup markers
            if file_name.ends_with(".lock")
                || file_name.ends_with(".tmp")
                || file_name == RESTORED_SIGNING_DISABLED_MARKER
            {
                continue;
            }

            let target_path = target_snapshot_dir.join(file_name);
            fs::copy(&path, &target_path).map_err(|e| StorageError::IoError(e.to_string()))?;

            let (size, checksum) = compute_file_sha256(&target_path)?;
            file_entries.push(SnapshotFileEntry {
                relative_path: file_name.to_string(),
                size_bytes: size,
                sha256_checksum: checksum,
            });
        }
    }

    let manifest = SnapshotManifest {
        format_version: 1,
        chain_id: meta.chain_id,
        genesis_fingerprint: meta.genesis_fingerprint,
        finalized_height: meta.finalized_height,
        finalized_block_hash: meta.finalized_block_hash,
        state_root: meta.state_root,
        files: file_entries,
        created_at_unix_secs: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    };

    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|e| StorageError::SerializationError(e.to_string()))?;
    fs::write(
        target_snapshot_dir.join(SNAPSHOT_MANIFEST_NAME),
        manifest_bytes,
    )
    .map_err(|e| StorageError::IoError(e.to_string()))?;

    Ok(manifest)
}

pub fn verify_snapshot(snapshot_path: &Path) -> Result<SnapshotManifest, StorageError> {
    let (snapshot_dir, manifest_path) = if snapshot_path.is_file() {
        (
            snapshot_path.parent().unwrap_or(snapshot_path),
            snapshot_path.to_path_buf(),
        )
    } else {
        (snapshot_path, snapshot_path.join(SNAPSHOT_MANIFEST_NAME))
    };

    if !manifest_path.exists() {
        return Err(StorageError::DatabaseError(
            "snapshot manifest.json missing".into(),
        ));
    }
    let manifest_bytes =
        fs::read(&manifest_path).map_err(|e| StorageError::IoError(e.to_string()))?;
    let manifest: SnapshotManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| StorageError::SerializationError(format!("manifest decode error: {e}")))?;

    if manifest.format_version != 1 {
        return Err(StorageError::DatabaseError(format!(
            "unsupported snapshot format version: {}",
            manifest.format_version
        )));
    }
    if manifest.files.is_empty() {
        return Err(StorageError::DatabaseError(
            "snapshot manifest contains no files".into(),
        ));
    }

    for file_entry in &manifest.files {
        let file_path = snapshot_dir.join(&file_entry.relative_path);
        if !file_path.exists() {
            return Err(StorageError::DatabaseError(format!(
                "snapshot file missing: {}",
                file_entry.relative_path
            )));
        }
        let (size, checksum) = compute_file_sha256(&file_path)?;
        if size != file_entry.size_bytes {
            return Err(StorageError::DatabaseError(format!(
                "file size mismatch for {}: expected {}, found {}",
                file_entry.relative_path, file_entry.size_bytes, size
            )));
        }
        if checksum != file_entry.sha256_checksum {
            return Err(StorageError::DatabaseError(format!(
                "checksum mismatch for {}: expected {}, found {}",
                file_entry.relative_path, file_entry.sha256_checksum, checksum
            )));
        }
    }

    Ok(manifest)
}

/// Safely restores a verified snapshot into `destination_dir`.
/// Leaves `RESTORED_SIGNING_DISABLED` marker in destination to prevent signer rollback hazards.
pub fn restore_backup(
    snapshot_path: &Path,
    destination_dir: &Path,
    expected_chain_id: Option<&str>,
    expected_genesis: Option<Hash32>,
) -> Result<SnapshotManifest, StorageError> {
    let snapshot_dir = if snapshot_path.is_file() {
        snapshot_path.parent().unwrap_or(snapshot_path)
    } else {
        snapshot_path
    };

    // 1. Verify snapshot integrity before taking any action
    let manifest = verify_snapshot(snapshot_path)?;

    // 2. Validate chain ID and genesis fingerprint if provided
    if let Some(chain_id) = expected_chain_id {
        if manifest.chain_id != chain_id {
            return Err(StorageError::DatabaseError(format!(
                "snapshot chain ID mismatch: expected '{chain_id}', found '{}'",
                manifest.chain_id
            )));
        }
    }
    if let Some(genesis) = expected_genesis {
        if manifest.genesis_fingerprint != genesis {
            return Err(StorageError::DatabaseError(
                "snapshot genesis fingerprint mismatch".into(),
            ));
        }
    }

    // 3. Ensure destination does not already contain conflicting data
    if destination_dir.exists() {
        let read =
            fs::read_dir(destination_dir).map_err(|e| StorageError::IoError(e.to_string()))?;
        if read.count() > 0 {
            return Err(StorageError::DatabaseError(
                "destination directory already exists and is not empty; explicit overwrite required".into(),
            ));
        }
    }
    fs::create_dir_all(destination_dir).map_err(|e| StorageError::IoError(e.to_string()))?;

    // 4. Copy each verified file into destination
    for file_entry in &manifest.files {
        let src = snapshot_dir.join(&file_entry.relative_path);
        let dst = destination_dir.join(&file_entry.relative_path);
        fs::copy(&src, &dst).map_err(|e| StorageError::IoError(e.to_string()))?;
    }

    // 5. CRITICAL SIGNER SAFETY GUARD:
    // Restored nodes MUST NOT automatically resume signing with an old journal.
    let marker_path = destination_dir.join(RESTORED_SIGNING_DISABLED_MARKER);
    let mut marker_file =
        File::create(&marker_path).map_err(|e| StorageError::IoError(e.to_string()))?;
    marker_file
        .write_all(b"Restored ledger. Validator signing is disabled until explicitly unblocked by operator.\n")
        .map_err(|e| StorageError::IoError(e.to_string()))?;
    marker_file
        .sync_all()
        .map_err(|e| StorageError::IoError(e.to_string()))?;

    Ok(manifest)
}

/// Rebuilds derived block hash and transaction indices from canonical blocks without rewriting state.
pub fn rebuild_chain_indexes<S: KVStore + ChainMetaStore + ReadonlyKVStore>(
    store: &S,
) -> Result<usize, StorageError> {
    let Some(height) = store.get_height()? else {
        return Ok(0);
    };
    let earliest = store.get_earliest_height()?.unwrap_or(0);
    let mut indexed_count = 0;

    for h in earliest..=height {
        let block_bytes = store.get_block(h)?.ok_or_else(|| {
            StorageError::DatabaseError(format!("missing canonical block at height {h}"))
        })?;
        let block: sprax_types::Block = serde_json::from_slice(&block_bytes)
            .map_err(|e| StorageError::SerializationError(format!("block decode error: {e}")))?;

        let block_hash = sprax_crypto::Hasher::block_hash(&block.header)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        store.put_block_hash_index(block_hash, h)?;
        indexed_count += 1;
    }

    Ok(indexed_count)
}
