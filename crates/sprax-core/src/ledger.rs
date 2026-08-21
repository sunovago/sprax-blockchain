use crate::{
    error::CoreError,
    executor::TxExecutor,
    genesis::GenesisConfig,
    state::{AccountState, StateAccessor},
};
use sprax_crypto::Hasher;
use sprax_storage::{ChainMetaStore, KVStore, MemKVStore, StateCommitment};
use sprax_types::{
    Address, Block, BlockBody, BlockHeader, Hash32, Transaction, TxMessage, TxReceipt,
};
use std::collections::HashMap;

/// Local Blockchain State Machine & Ledger Manager, generic over its backing store.
///
/// Defaults to [`MemKVStore`] so every existing call site that names the bare `ChainLedger`
/// type (no explicit `<S>`) keeps compiling unchanged; `sprax-node` opts into a durable
/// backend explicitly (`ChainLedger<RedbStore>`).
#[derive(Debug)]
pub struct ChainLedger<S: KVStore + StateCommitment + ChainMetaStore + Clone = MemKVStore> {
    genesis: GenesisConfig,
    store: S,
    blocks: Vec<Block>,
    block_by_hash: HashMap<Hash32, u64>,
    tx_index: HashMap<Hash32, (Transaction, TxReceipt, u64)>,
    mempool: Vec<Transaction>,
    executor: TxExecutor,
}

impl<S: KVStore + StateCommitment + ChainMetaStore + Clone> ChainLedger<S> {
    /// Initializes a fresh blockchain ledger from a genesis specification against `store`,
    /// persisting the genesis block so a subsequent [`Self::open_or_init`] against the same
    /// store recognizes state already exists.
    pub fn init_from_genesis_with_store(
        genesis: GenesisConfig,
        store: S,
    ) -> Result<Self, CoreError> {
        let genesis_header = genesis.initialize_state(&store)?;

        let genesis_block = Block {
            header: genesis_header.clone(),
            body: BlockBody::default(),
            last_commit: vec![],
        };

        let genesis_hash = Hasher::block_hash(&genesis_header)
            .map_err(|e| CoreError::StateError(e.to_string()))?;

        let mut block_by_hash = HashMap::new();
        block_by_hash.insert(genesis_hash, 0);

        Self::persist_block(&store, 0, &genesis_block, genesis_hash)?;
        store
            .put_height(0)
            .map_err(|e| CoreError::StateError(e.to_string()))?;

        Ok(Self {
            genesis,
            store,
            blocks: vec![genesis_block],
            block_by_hash,
            tx_index: HashMap::new(),
            mempool: vec![],
            executor: TxExecutor::default(),
        })
    }

    /// Opens a ledger backed by `store`: if the store already holds chain state (a prior
    /// process persisted it), rebuilds in-memory caches by reading blocks directly out of the
    /// store — **no transaction re-execution** — otherwise initializes fresh genesis state via
    /// `genesis_provider`.
    pub fn open_or_init(
        store: S,
        genesis_provider: impl FnOnce() -> Result<GenesisConfig, CoreError>,
    ) -> Result<Self, CoreError> {
        let existing_height = store
            .get_height()
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        match existing_height {
            Some(height) => Self::rebuild_from_store(genesis_provider()?, store, height),
            None => Self::init_from_genesis_with_store(genesis_provider()?, store),
        }
    }

    fn rebuild_from_store(
        genesis: GenesisConfig,
        store: S,
        height: u64,
    ) -> Result<Self, CoreError> {
        let mut blocks = Vec::with_capacity(height as usize + 1);
        let mut block_by_hash = HashMap::new();
        let mut tx_index = HashMap::new();

        for h in 0..=height {
            let block_bytes = store
                .get_block(h)
                .map_err(|e| CoreError::StateError(e.to_string()))?
                .ok_or_else(|| {
                    CoreError::StateError(format!("missing persisted block at height {h}"))
                })?;
            let block: Block = serde_json::from_slice(&block_bytes)
                .map_err(|e| CoreError::StateError(format!("block decode error: {e}")))?;
            let block_hash = Hasher::block_hash(&block.header)
                .map_err(|e| CoreError::StateError(e.to_string()))?;
            block_by_hash.insert(block_hash, h);

            for tx in &block.body.transactions {
                let tx_hash =
                    Hasher::tx_hash(tx).map_err(|e| CoreError::StateError(e.to_string()))?;
                if let Some(entry_bytes) = store
                    .get_tx_index(tx_hash)
                    .map_err(|e| CoreError::StateError(e.to_string()))?
                {
                    let (stored_tx, receipt, stored_height): (Transaction, TxReceipt, u64) =
                        serde_json::from_slice(&entry_bytes).map_err(|e| {
                            CoreError::StateError(format!("tx index decode error: {e}"))
                        })?;
                    tx_index.insert(tx_hash, (stored_tx, receipt, stored_height));
                }
            }

            blocks.push(block);
        }

        Ok(Self {
            genesis,
            store,
            blocks,
            block_by_hash,
            tx_index,
            mempool: vec![],
            executor: TxExecutor::default(),
        })
    }

    fn persist_block(
        store: &S,
        height: u64,
        block: &Block,
        block_hash: Hash32,
    ) -> Result<(), CoreError> {
        let bytes = serde_json::to_vec(block)
            .map_err(|e| CoreError::StateError(format!("block encode error: {e}")))?;
        store
            .put_block(height, &bytes)
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        store
            .put_block_hash_index(block_hash, height)
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        Ok(())
    }

    fn persist_tx_index(
        store: &S,
        tx_hash: Hash32,
        tx: &Transaction,
        receipt: &TxReceipt,
        height: u64,
    ) -> Result<(), CoreError> {
        let bytes = serde_json::to_vec(&(tx, receipt, height))
            .map_err(|e| CoreError::StateError(format!("tx index encode error: {e}")))?;
        store
            .put_tx_index(tx_hash, &bytes)
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        Ok(())
    }

    #[must_use]
    pub fn genesis(&self) -> &GenesisConfig {
        &self.genesis
    }

    #[must_use]
    pub fn chain_id(&self) -> &str {
        &self.genesis.chain_id
    }

    #[must_use]
    pub fn height(&self) -> u64 {
        self.blocks.len().saturating_sub(1) as u64
    }

    #[must_use]
    pub fn latest_block(&self) -> &Block {
        self.blocks
            .last()
            .expect("ledger always contains at least genesis block")
    }

    #[must_use]
    pub fn latest_header(&self) -> &BlockHeader {
        &self.latest_block().header
    }

    pub fn state_root(&self) -> Result<Hash32, CoreError> {
        self.store
            .compute_root()
            .map_err(|e| CoreError::StateError(e.to_string()))
    }

    #[must_use]
    pub fn mempool_len(&self) -> usize {
        self.mempool.len()
    }

    pub fn get_account(&self, addr: &Address) -> Result<AccountState, CoreError> {
        StateAccessor::get_account(&self.store, addr)
    }

    /// Submits a signed transaction into the local mempool after pre-validation.
    pub fn submit_transaction(&mut self, tx: Transaction) -> Result<Hash32, CoreError> {
        let tx_hash = Hasher::tx_hash(&tx).map_err(|e| CoreError::StateError(e.to_string()))?;

        // Check if already in tx index or mempool
        if self.tx_index.contains_key(&tx_hash) {
            return Err(CoreError::ModuleError {
                module: "mempool".into(),
                reason: "transaction already included in block".into(),
            });
        }
        if self
            .mempool
            .iter()
            .any(|t| Hasher::tx_hash(t).ok() == Some(tx_hash))
        {
            return Err(CoreError::ModuleError {
                module: "mempool".into(),
                reason: "transaction already pending in mempool".into(),
            });
        }

        // Static and Nonce Pre-validation against current state
        self.executor
            .validate_transaction_static(&tx, &self.genesis.chain_id)?;
        let sender_state = StateAccessor::get_account(&self.store, &tx.body.sender)?;
        if tx.body.nonce != sender_state.nonce {
            return Err(CoreError::InvalidNonce {
                account_nonce: sender_state.nonce,
                tx_nonce: tx.body.nonce,
            });
        }

        let mut total_transfer_cost = sprax_types::Amount::ZERO;
        for msg in &tx.body.messages {
            if let TxMessage::Transfer { amount, .. } = msg {
                total_transfer_cost = total_transfer_cost
                    .checked_add(*amount)
                    .map_err(|e| CoreError::StateError(e.to_string()))?;
            }
        }
        let total_required = total_transfer_cost
            .checked_add(tx.body.fee.amount)
            .map_err(|e| CoreError::StateError(e.to_string()))?;

        if sender_state.balance < total_required {
            return Err(CoreError::InsufficientFunds {
                balance: sender_state.balance.to_string(),
                required: total_required.to_string(),
            });
        }

        self.mempool.push(tx);
        Ok(tx_hash)
    }

    /// Mines a new block from all pending transactions in the mempool.
    pub fn mine_block(&mut self, proposer: Address) -> Result<Block, CoreError> {
        let current_height = self.height() + 1;
        let (parent_hash, parent_timestamp, val_set_hash) = {
            let parent_header = self.latest_header();
            let hash = Hasher::block_hash(parent_header)
                .map_err(|e| CoreError::StateError(e.to_string()))?;
            (
                hash,
                parent_header.timestamp_unix_secs,
                parent_header.validator_set_hash,
            )
        };

        let pending_txs = std::mem::take(&mut self.mempool);
        let mut executed_txs = Vec::with_capacity(pending_txs.len());
        let mut receipts = Vec::with_capacity(pending_txs.len());
        let mut tx_hashes = Vec::with_capacity(pending_txs.len());

        for tx in pending_txs {
            match self.executor.execute_transaction(
                &self.store,
                &tx,
                current_height,
                &self.genesis.chain_id,
            ) {
                Ok(receipt) => {
                    let hash = receipt.tx_hash;
                    tx_hashes.push(hash);
                    receipts.push(receipt.clone());
                    self.tx_index
                        .insert(hash, (tx.clone(), receipt.clone(), current_height));
                    Self::persist_tx_index(&self.store, hash, &tx, &receipt, current_height)?;
                    executed_txs.push(tx);
                }
                Err(err) => {
                    // Drop invalid tx from block proposal
                    tracing::warn!("Dropped invalid transaction from proposal: {err}");
                }
            }
        }

        let state_root = self.state_root()?;
        let txs_root = Hasher::merkle_root(&tx_hashes);
        let receipt_hashes: Vec<Hash32> = receipts
            .iter()
            .map(|r| Hasher::receipt_hash(r).unwrap_or(Hash32::ZERO))
            .collect();
        let receipts_root = Hasher::merkle_root(&receipt_hashes);

        let header = BlockHeader {
            version: 1,
            chain_id: self.genesis.chain_id.clone(),
            height: current_height,
            timestamp_unix_secs: parent_timestamp + 2,
            parent_hash,
            proposer,
            state_root,
            txs_root,
            receipts_root,
            validator_set_hash: val_set_hash,
        };

        let block = Block {
            header: header.clone(),
            body: BlockBody {
                transactions: executed_txs,
            },
            last_commit: vec![],
        };

        let block_hash =
            Hasher::block_hash(&header).map_err(|e| CoreError::StateError(e.to_string()))?;

        self.block_by_hash.insert(block_hash, current_height);
        Self::persist_block(&self.store, current_height, &block, block_hash)?;
        self.store
            .put_height(current_height)
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        self.blocks.push(block.clone());

        Ok(block)
    }

    /// Validates and applies an externally received block to the local ledger (for gossip / synchronization).
    pub fn apply_block(&mut self, block: Block) -> Result<Vec<TxReceipt>, CoreError> {
        let block_height = block.header.height;
        let local_height = self.height();

        // 1. Idempotently ignore already processed or duplicate historical blocks
        if block_height <= local_height {
            return Ok(vec![]);
        }

        // 2. Enforce strict sequential block order
        if block_height != local_height + 1 {
            return Err(CoreError::ModuleError {
                module: "consensus".into(),
                reason: format!(
                    "out-of-order block: expected height {}, received {}",
                    local_height + 1,
                    block_height
                ),
            });
        }

        // 3. Verify Chain ID
        if block.header.chain_id != self.genesis.chain_id {
            return Err(CoreError::ModuleError {
                module: "consensus".into(),
                reason: format!(
                    "chain ID mismatch in block: expected '{}', found '{}'",
                    self.genesis.chain_id, block.header.chain_id
                ),
            });
        }

        // 4. Verify Parent Hash
        let expected_parent_hash = Hasher::block_hash(self.latest_header())
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        if block.header.parent_hash != expected_parent_hash {
            return Err(CoreError::ModuleError {
                module: "consensus".into(),
                reason: format!(
                    "parent hash mismatch: expected {expected_parent_hash}, found {}",
                    block.header.parent_hash
                ),
            });
        }

        // 5. Execute all transactions in the block
        let mut tx_hashes = Vec::with_capacity(block.body.transactions.len());
        let mut receipts = Vec::with_capacity(block.body.transactions.len());

        for tx in &block.body.transactions {
            let receipt = self.executor.execute_transaction(
                &self.store,
                tx,
                block_height,
                &self.genesis.chain_id,
            )?;
            let hash = receipt.tx_hash;
            tx_hashes.push(hash);
            receipts.push(receipt.clone());
            self.tx_index
                .insert(hash, (tx.clone(), receipt.clone(), block_height));
            Self::persist_tx_index(&self.store, hash, tx, &receipt, block_height)?;
        }

        // 6. Verify Computed State Root matches Header State Root
        let computed_state_root = self.state_root()?;
        if computed_state_root != block.header.state_root {
            return Err(CoreError::ModuleError {
                module: "consensus".into(),
                reason: format!(
                    "state root mismatch: computed {computed_state_root}, header {}",
                    block.header.state_root
                ),
            });
        }

        // 7. Verify Computed Txs Root matches Header Txs Root
        let computed_txs_root = Hasher::merkle_root(&tx_hashes);
        if computed_txs_root != block.header.txs_root {
            return Err(CoreError::ModuleError {
                module: "consensus".into(),
                reason: format!(
                    "txs root mismatch: computed {computed_txs_root}, header {}",
                    block.header.txs_root
                ),
            });
        }

        // 7b. Verify Computed Receipts Root matches Header Receipts Root
        let receipt_hashes: Vec<Hash32> = receipts
            .iter()
            .map(|r| Hasher::receipt_hash(r).unwrap_or(Hash32::ZERO))
            .collect();
        let computed_receipts_root = Hasher::merkle_root(&receipt_hashes);
        if computed_receipts_root != block.header.receipts_root {
            return Err(CoreError::ModuleError {
                module: "consensus".into(),
                reason: format!(
                    "receipts root mismatch: computed {computed_receipts_root}, header {}",
                    block.header.receipts_root
                ),
            });
        }

        // 8. Commit block to local ledger
        let block_hash =
            Hasher::block_hash(&block.header).map_err(|e| CoreError::StateError(e.to_string()))?;
        self.block_by_hash.insert(block_hash, block_height);
        Self::persist_block(&self.store, block_height, &block, block_hash)?;
        self.store
            .put_height(block_height)
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        self.blocks.push(block);

        // Remove executed txs from mempool if present
        self.mempool.retain(|tx| {
            if let Ok(h) = Hasher::tx_hash(tx) {
                !self.tx_index.contains_key(&h)
            } else {
                false
            }
        });

        Ok(receipts)
    }

    /// Applies a sequential batch of historical blocks for catch-up synchronization.
    pub fn apply_blocks_batch(&mut self, blocks: Vec<Block>) -> Result<usize, CoreError> {
        let mut count = 0;
        for block in blocks {
            self.apply_block(block)?;
            count += 1;
        }
        Ok(count)
    }

    pub fn get_block_by_height(&self, height: u64) -> Option<&Block> {
        self.blocks.get(height as usize)
    }

    pub fn get_block_by_hash(&self, hash: &Hash32) -> Option<&Block> {
        let height = self.block_by_hash.get(hash)?;
        self.get_block_by_height(*height)
    }

    pub fn get_transaction(&self, hash: &Hash32) -> Option<(&Transaction, &TxReceipt, u64)> {
        self.tx_index
            .get(hash)
            .map(|(tx, receipt, h)| (tx, receipt, *h))
    }

    /// Overwrites the finalized commit signatures on a already-committed block (called once a
    /// BFT round reaches +2/3 precommit quorum) and re-persists that block.
    pub fn set_last_commit(
        &mut self,
        height: u64,
        commit_sigs: Vec<sprax_types::CommitSignature>,
    ) -> Result<(), CoreError> {
        let block = self
            .blocks
            .get_mut(height as usize)
            .ok_or_else(|| CoreError::StateError(format!("no block at height {height}")))?;
        block.last_commit = commit_sigs;
        let block_snapshot = block.clone();
        let block_hash = Hasher::block_hash(&block_snapshot.header)
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        Self::persist_block(&self.store, height, &block_snapshot, block_hash)?;
        Ok(())
    }
}

impl ChainLedger<MemKVStore> {
    /// Initializes a fresh in-memory blockchain ledger from a genesis specification.
    pub fn init_from_genesis(genesis: GenesisConfig) -> Result<Self, CoreError> {
        Self::init_from_genesis_with_store(genesis, MemKVStore::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sprax_crypto::Ed25519Keypair;
    use sprax_types::{Amount, ChainId, KeyType, TxBody, TxFee, TxMessage};

    #[test]
    fn test_full_chain_ledger_flow() {
        let genesis = GenesisConfig::default_development();
        let ledger = ChainLedger::init_from_genesis(genesis).unwrap();
        assert_eq!(ledger.height(), 0);

        let alice_addr = Address::new([1u8; 20]);
        let alice_state = ledger.get_account(&alice_addr).unwrap();
        assert_eq!(
            alice_state.balance,
            Amount::from_sprx_whole(1_000_000).unwrap()
        );

        // Alice transfers to Bob
        let bob_kp = Ed25519Keypair::generate();
        let bob_addr = bob_kp.address();

        // Let's create an Alice keypair whose address is alice_addr
        let seed = [1u8; 32];
        let alice_kp = Ed25519Keypair::from_seed(&seed);
        let alice_real_addr = alice_kp.address();

        // Update ledger so alice_real_addr has balance
        let mut real_genesis = GenesisConfig::default_development();
        real_genesis.accounts[0].address = alice_real_addr;
        let mut ledger = ChainLedger::init_from_genesis(real_genesis).unwrap();

        let tx_body = TxBody {
            chain_id: ChainId::new("sprax-devnet-1").unwrap(),
            sender: alice_real_addr,
            nonce: 0,
            messages: vec![TxMessage::Transfer {
                to: bob_addr,
                amount: Amount::from_sprx_whole(250).unwrap(),
            }],
            fee: TxFee::default(),
            memo: "dev transfer".into(),
            timeout_height: 50,
        };

        let sign_bytes = serde_json::to_vec(&tx_body).unwrap();
        let sig = alice_kp.sign(&sign_bytes);
        let tx = Transaction::new(
            tx_body,
            KeyType::Ed25519,
            alice_kp.public_key_bytes().to_vec(),
            sig,
        )
        .unwrap();

        let tx_hash = ledger.submit_transaction(tx).unwrap();
        assert_eq!(ledger.mempool_len(), 1);

        let mined_block = ledger.mine_block(Address::new([9u8; 20])).unwrap();
        assert_eq!(mined_block.header.height, 1);
        assert_eq!(ledger.height(), 1);
        assert_eq!(ledger.mempool_len(), 0);

        let (queried_tx, receipt, h) = ledger.get_transaction(&tx_hash).unwrap();
        assert_eq!(h, 1);
        assert!(receipt.success);
        assert_eq!(queried_tx.body.sender, alice_real_addr);

        let bob_bal = ledger.get_account(&bob_addr).unwrap();
        assert_eq!(bob_bal.balance, Amount::from_sprx_whole(250).unwrap());
    }
}
