use crate::{error::CoreError, genesis::GenesisConfig, ledger::ChainLedger};
use serde::{Deserialize, Serialize};
use sprax_crypto::{Hasher, MerkleTree};
use sprax_storage::{ChainMetaStore, KVStore, StateCommitment};
use sprax_types::{Block, Hash32};

/// Classification of blockchain integrity violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationCategory {
    GenesisMismatch,
    HeaderHashMismatch,
    ParentHashMismatch,
    HeightSequenceGap,
    NonMonotonicTimestamp,
    TxMerkleRootMismatch,
    ReceiptMerkleRootMismatch,
    StateRootMismatch,
    StorageIndexInconsistency,
}

/// A specific integrity violation detected during audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityViolation {
    pub height: u64,
    pub block_hash: Option<Hash32>,
    pub category: ViolationCategory,
    pub description: String,
}

/// Comprehensive report produced by verifying blockchain and state integrity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub is_valid: bool,
    pub earliest_height: u64,
    pub tip_height: u64,
    pub total_blocks_verified: u64,
    pub total_transactions_verified: u64,
    pub violations: Vec<IntegrityViolation>,
}

impl IntegrityReport {
    #[must_use]
    pub fn success(
        earliest_height: u64,
        tip_height: u64,
        total_blocks_verified: u64,
        total_transactions_verified: u64,
    ) -> Self {
        Self {
            is_valid: true,
            earliest_height,
            tip_height,
            total_blocks_verified,
            total_transactions_verified,
            violations: vec![],
        }
    }

    #[must_use]
    pub fn failure(
        earliest_height: u64,
        tip_height: u64,
        total_blocks_verified: u64,
        total_transactions_verified: u64,
        violations: Vec<IntegrityViolation>,
    ) -> Self {
        Self {
            is_valid: false,
            earliest_height,
            tip_height,
            total_blocks_verified,
            total_transactions_verified,
            violations,
        }
    }
}

/// Cryptographic blockchain and state integrity audit engine.
#[derive(Debug, Clone, Copy)]
pub struct ChainIntegrityVerifier;

impl ChainIntegrityVerifier {
    /// Audits an active `ChainLedger` in-memory and against its storage backend.
    pub fn verify_ledger<S: KVStore + StateCommitment + ChainMetaStore + Clone + 'static>(
        ledger: &ChainLedger<S>,
    ) -> Result<IntegrityReport, CoreError> {
        let earliest_height = ledger.earliest_height();
        let tip_height = ledger.height();
        let mut violations = Vec::new();
        let mut total_blocks = 0u64;
        let mut total_txs = 0u64;

        let mut prev_block: Option<&Block> = None;

        for h in earliest_height..=tip_height {
            let block = match ledger.get_block_by_height(h) {
                Some(b) => b,
                None => {
                    violations.push(IntegrityViolation {
                        height: h,
                        block_hash: None,
                        category: ViolationCategory::HeightSequenceGap,
                        description: format!("missing block in ledger at height {h}"),
                    });
                    continue;
                }
            };

            total_blocks += 1;
            total_txs += block.body.transactions.len() as u64;

            // 1. Verify computed header hash
            let computed_block_hash = match Hasher::block_hash(&block.header) {
                Ok(hash) => hash,
                Err(e) => {
                    violations.push(IntegrityViolation {
                        height: h,
                        block_hash: None,
                        category: ViolationCategory::HeaderHashMismatch,
                        description: format!("failed to hash block header: {e}"),
                    });
                    prev_block = Some(block);
                    continue;
                }
            };

            // 2. Verify genesis block constraints if height 0
            if h == 0 {
                if block.header.height != 0 {
                    violations.push(IntegrityViolation {
                        height: 0,
                        block_hash: Some(computed_block_hash),
                        category: ViolationCategory::GenesisMismatch,
                        description: format!(
                            "genesis block header height must be 0, found {}",
                            block.header.height
                        ),
                    });
                }
                if block.header.parent_hash != Hash32::ZERO {
                    violations.push(IntegrityViolation {
                        height: 0,
                        block_hash: Some(computed_block_hash),
                        category: ViolationCategory::GenesisMismatch,
                        description: format!(
                            "genesis parent hash must be ZERO, found {}",
                            block.header.parent_hash
                        ),
                    });
                }
            } else if let Some(parent) = prev_block {
                // 3. Verify sequential height
                if block.header.height != parent.header.height + 1 {
                    violations.push(IntegrityViolation {
                        height: h,
                        block_hash: Some(computed_block_hash),
                        category: ViolationCategory::HeightSequenceGap,
                        description: format!(
                            "height sequence gap: expected {}, found {}",
                            parent.header.height + 1,
                            block.header.height
                        ),
                    });
                }

                // 4. Verify parent hash linkage
                let expected_parent_hash =
                    Hasher::block_hash(&parent.header).unwrap_or(Hash32::ZERO);
                if block.header.parent_hash != expected_parent_hash {
                    violations.push(IntegrityViolation {
                        height: h,
                        block_hash: Some(computed_block_hash),
                        category: ViolationCategory::ParentHashMismatch,
                        description: format!(
                            "parent hash mismatch at height {h}: expected {expected_parent_hash}, found {}",
                            block.header.parent_hash
                        ),
                    });
                }

                // 5. Verify timestamp monotonicity
                if block.header.timestamp_unix_secs <= parent.header.timestamp_unix_secs {
                    violations.push(IntegrityViolation {
                        height: h,
                        block_hash: Some(computed_block_hash),
                        category: ViolationCategory::NonMonotonicTimestamp,
                        description: format!(
                            "non-monotonic timestamp at height {h}: parent {}, block {}",
                            parent.header.timestamp_unix_secs, block.header.timestamp_unix_secs
                        ),
                    });
                }
            }

            // 6. Verify Transaction Merkle Root
            let mut tx_hashes = Vec::with_capacity(block.body.transactions.len());
            for tx in &block.body.transactions {
                match Hasher::tx_hash(tx) {
                    Ok(tx_h) => tx_hashes.push(tx_h),
                    Err(e) => {
                        violations.push(IntegrityViolation {
                            height: h,
                            block_hash: Some(computed_block_hash),
                            category: ViolationCategory::TxMerkleRootMismatch,
                            description: format!("transaction hashing error: {e}"),
                        });
                    }
                }
            }
            let computed_txs_root = MerkleTree::compute_root(&tx_hashes);
            if computed_txs_root != block.header.txs_root {
                violations.push(IntegrityViolation {
                    height: h,
                    block_hash: Some(computed_block_hash),
                    category: ViolationCategory::TxMerkleRootMismatch,
                    description: format!(
                        "txs root mismatch at height {h}: computed {computed_txs_root}, header {}",
                        block.header.txs_root
                    ),
                });
            }

            // 7. Verify Receipts and Receipts Merkle Root
            let mut receipt_hashes = Vec::with_capacity(block.body.transactions.len());
            for tx_h in &tx_hashes {
                if let Some((_, receipt, stored_h)) = ledger.get_transaction(tx_h) {
                    if stored_h != h {
                        violations.push(IntegrityViolation {
                            height: h,
                            block_hash: Some(computed_block_hash),
                            category: ViolationCategory::StorageIndexInconsistency,
                            description: format!(
                                "tx {tx_h} indexed at height {stored_h}, but included in block {h}"
                            ),
                        });
                    }
                    if let Ok(rh) = Hasher::receipt_hash(receipt) {
                        receipt_hashes.push(rh);
                    }
                }
            }
            if !receipt_hashes.is_empty() {
                let computed_receipts_root = MerkleTree::compute_root(&receipt_hashes);
                if computed_receipts_root != block.header.receipts_root {
                    violations.push(IntegrityViolation {
                        height: h,
                        block_hash: Some(computed_block_hash),
                        category: ViolationCategory::ReceiptMerkleRootMismatch,
                        description: format!(
                            "receipts root mismatch at height {h}: computed {computed_receipts_root}, header {}",
                            block.header.receipts_root
                        ),
                    });
                }
            }

            // 8. Verify Block Hash Lookup Index
            if let Some(indexed_block) = ledger.get_block_by_hash(&computed_block_hash) {
                if indexed_block.header.height != h {
                    violations.push(IntegrityViolation {
                        height: h,
                        block_hash: Some(computed_block_hash),
                        category: ViolationCategory::StorageIndexInconsistency,
                        description: format!(
                            "block hash index points to height {}, expected {h}",
                            indexed_block.header.height
                        ),
                    });
                }
            } else {
                violations.push(IntegrityViolation {
                    height: h,
                    block_hash: Some(computed_block_hash),
                    category: ViolationCategory::StorageIndexInconsistency,
                    description: format!("block hash {computed_block_hash} not found in index"),
                });
            }

            prev_block = Some(block);
        }

        if violations.is_empty() {
            Ok(IntegrityReport::success(
                earliest_height,
                tip_height,
                total_blocks,
                total_txs,
            ))
        } else {
            Ok(IntegrityReport::failure(
                earliest_height,
                tip_height,
                total_blocks,
                total_txs,
                violations,
            ))
        }
    }

    /// Performs offline audit of persistent storage directly from disk without initializing full ledger state.
    pub fn verify_storage<S: KVStore + StateCommitment + ChainMetaStore>(
        store: &S,
        genesis: &GenesisConfig,
    ) -> Result<IntegrityReport, CoreError> {
        let height = match store
            .get_height()
            .map_err(|e| CoreError::StateError(e.to_string()))?
        {
            Some(h) => h,
            None => {
                return Ok(IntegrityReport::success(0, 0, 0, 0));
            }
        };

        let earliest_height = store
            .get_earliest_height()
            .map_err(|e| CoreError::StateError(e.to_string()))?
            .unwrap_or(0);

        let mut violations = Vec::new();
        let mut total_blocks = 0u64;
        let mut total_txs = 0u64;
        let mut prev_block: Option<Block> = None;

        for h in earliest_height..=height {
            let block_bytes = match store
                .get_block(h)
                .map_err(|e| CoreError::StateError(e.to_string()))?
            {
                Some(bytes) => bytes,
                None => {
                    violations.push(IntegrityViolation {
                        height: h,
                        block_hash: None,
                        category: ViolationCategory::HeightSequenceGap,
                        description: format!("missing block in storage at height {h}"),
                    });
                    continue;
                }
            };

            let block: Block = match serde_json::from_slice(&block_bytes) {
                Ok(b) => b,
                Err(e) => {
                    violations.push(IntegrityViolation {
                        height: h,
                        block_hash: None,
                        category: ViolationCategory::HeaderHashMismatch,
                        description: format!("failed to decode block at height {h}: {e}"),
                    });
                    continue;
                }
            };

            total_blocks += 1;
            total_txs += block.body.transactions.len() as u64;

            let computed_hash = Hasher::block_hash(&block.header).unwrap_or(Hash32::ZERO);

            if block.header.chain_id != genesis.chain_id {
                violations.push(IntegrityViolation {
                    height: h,
                    block_hash: Some(computed_hash),
                    category: ViolationCategory::GenesisMismatch,
                    description: format!(
                        "chain ID mismatch: expected '{}', found '{}'",
                        genesis.chain_id, block.header.chain_id
                    ),
                });
            }

            if h == 0 {
                if block.header.parent_hash != Hash32::ZERO {
                    violations.push(IntegrityViolation {
                        height: 0,
                        block_hash: Some(computed_hash),
                        category: ViolationCategory::GenesisMismatch,
                        description: "genesis parent hash must be ZERO".into(),
                    });
                }
            } else if let Some(parent) = &prev_block {
                let parent_hash = Hasher::block_hash(&parent.header).unwrap_or(Hash32::ZERO);
                if block.header.parent_hash != parent_hash {
                    violations.push(IntegrityViolation {
                        height: h,
                        block_hash: Some(computed_hash),
                        category: ViolationCategory::ParentHashMismatch,
                        description: format!(
                            "parent hash mismatch at height {h}: expected {parent_hash}, found {}",
                            block.header.parent_hash
                        ),
                    });
                }
                if block.header.timestamp_unix_secs <= parent.header.timestamp_unix_secs {
                    violations.push(IntegrityViolation {
                        height: h,
                        block_hash: Some(computed_hash),
                        category: ViolationCategory::NonMonotonicTimestamp,
                        description: format!(
                            "non-monotonic timestamp: parent {}, current {}",
                            parent.header.timestamp_unix_secs, block.header.timestamp_unix_secs
                        ),
                    });
                }
            }

            prev_block = Some(block);
        }

        if violations.is_empty() {
            Ok(IntegrityReport::success(
                earliest_height,
                height,
                total_blocks,
                total_txs,
            ))
        } else {
            Ok(IntegrityReport::failure(
                earliest_height,
                height,
                total_blocks,
                total_txs,
                violations,
            ))
        }
    }
}
