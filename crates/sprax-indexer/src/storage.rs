use crate::models::{
    IndexedAccount, IndexedBlock, IndexedTx, IndexedValidator, NetworkStats, PaginatedResponse,
    SearchResult,
};
use parking_lot::RwLock;
use sprax_types::{Address, Amount, Hash32};
use std::collections::{BTreeMap, HashMap};

/// Relational In-Memory Index Database with secondary indexing.
#[derive(Debug, Default)]
pub struct IndexStore {
    // Blocks indexed by height (ordered) and hash
    blocks_by_height: RwLock<BTreeMap<u64, IndexedBlock>>,
    block_height_by_hash: RwLock<HashMap<Hash32, u64>>,

    // Transactions indexed by hash, block height, and address
    txs_by_hash: RwLock<HashMap<Hash32, IndexedTx>>,
    tx_hashes_by_height: RwLock<HashMap<u64, Vec<Hash32>>>,
    tx_hashes_by_address: RwLock<HashMap<Address, Vec<Hash32>>>,
    tx_chronological: RwLock<Vec<Hash32>>,

    // Accounts indexed by address
    accounts_by_address: RwLock<HashMap<Address, IndexedAccount>>,

    // Validators indexed by operator address
    validators_by_address: RwLock<HashMap<Address, IndexedValidator>>,

    // Metadata
    chain_id: RwLock<String>,
}

impl IndexStore {
    pub fn new(chain_id: String) -> Self {
        Self {
            chain_id: RwLock::new(chain_id),
            ..Default::default()
        }
    }

    /// Inserts an indexed block and updates block indices.
    pub fn insert_block(&self, block: IndexedBlock) {
        let mut by_height = self.blocks_by_height.write();
        let mut by_hash = self.block_height_by_hash.write();

        by_hash.insert(block.hash, block.height);
        by_height.insert(block.height, block);
    }

    /// Inserts an indexed transaction and updates account/height links.
    pub fn insert_tx(&self, tx: IndexedTx) {
        let hash = tx.tx_hash;
        let height = tx.block_height;
        let sender = tx.sender;
        let recipient = tx.recipient;

        self.txs_by_hash.write().insert(hash, tx);
        self.tx_chronological.write().push(hash);

        self.tx_hashes_by_height
            .write()
            .entry(height)
            .or_default()
            .push(hash);

        self.tx_hashes_by_address
            .write()
            .entry(sender)
            .or_default()
            .push(hash);

        if let Some(to) = recipient {
            if to != sender {
                self.tx_hashes_by_address
                    .write()
                    .entry(to)
                    .or_default()
                    .push(hash);
            }
        }
    }

    /// Updates or inserts an account record.
    pub fn upsert_account(&self, account: IndexedAccount) {
        self.accounts_by_address
            .write()
            .insert(account.address, account);
    }

    /// Upserts a validator profile.
    pub fn upsert_validator(&self, validator: IndexedValidator) {
        self.validators_by_address
            .write()
            .insert(validator.operator_address, validator);
    }

    pub fn get_block_by_height(&self, height: u64) -> Option<IndexedBlock> {
        self.blocks_by_height.read().get(&height).cloned()
    }

    pub fn get_block_by_hash(&self, hash: &Hash32) -> Option<IndexedBlock> {
        let by_hash = self.block_height_by_hash.read();
        let height = by_hash.get(hash)?;
        self.blocks_by_height.read().get(height).cloned()
    }

    pub fn get_tx_by_hash(&self, hash: &Hash32) -> Option<IndexedTx> {
        self.txs_by_hash.read().get(hash).cloned()
    }

    pub fn get_account(&self, address: &Address) -> Option<IndexedAccount> {
        self.accounts_by_address.read().get(address).cloned()
    }

    pub fn get_validator(&self, address: &Address) -> Option<IndexedValidator> {
        self.validators_by_address.read().get(address).cloned()
    }

    /// Returns paginated list of blocks sorted descending by height.
    pub fn get_blocks_paginated(
        &self,
        limit: usize,
        offset: usize,
    ) -> PaginatedResponse<IndexedBlock> {
        let guard = self.blocks_by_height.read();
        let total = guard.len();
        let items: Vec<IndexedBlock> = guard
            .values()
            .rev()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();

        PaginatedResponse {
            has_more: offset + items.len() < total,
            items,
            total,
            limit,
            offset,
        }
    }

    /// Returns paginated list of transactions sorted descending by chronological order.
    pub fn get_txs_paginated(&self, limit: usize, offset: usize) -> PaginatedResponse<IndexedTx> {
        let hashes = self.tx_chronological.read();
        let total = hashes.len();
        let txs_map = self.txs_by_hash.read();

        let items: Vec<IndexedTx> = hashes
            .iter()
            .rev()
            .skip(offset)
            .take(limit)
            .filter_map(|h| txs_map.get(h).cloned())
            .collect();

        PaginatedResponse {
            has_more: offset + items.len() < total,
            items,
            total,
            limit,
            offset,
        }
    }

    /// Returns paginated transactions for a specific address.
    pub fn get_address_txs_paginated(
        &self,
        address: &Address,
        limit: usize,
        offset: usize,
    ) -> PaginatedResponse<IndexedTx> {
        let by_addr = self.tx_hashes_by_address.read();
        let empty_vec = Vec::new();
        let hashes = by_addr.get(address).unwrap_or(&empty_vec);
        let total = hashes.len();
        let txs_map = self.txs_by_hash.read();

        let items: Vec<IndexedTx> = hashes
            .iter()
            .rev()
            .skip(offset)
            .take(limit)
            .filter_map(|h| txs_map.get(h).cloned())
            .collect();

        PaginatedResponse {
            has_more: offset + items.len() < total,
            items,
            total,
            limit,
            offset,
        }
    }

    /// Returns all validators sorted descending by voting power.
    pub fn get_validators(&self) -> Vec<IndexedValidator> {
        let mut vals: Vec<IndexedValidator> = self
            .validators_by_address
            .read()
            .values()
            .cloned()
            .collect();
        vals.sort_by_key(|b| std::cmp::Reverse(b.voting_power));
        vals
    }

    /// Derives overall network health and aggregate statistics.
    pub fn get_network_stats(&self) -> NetworkStats {
        let blocks = self.blocks_by_height.read();
        let total_txs = self.tx_chronological.read().len();
        let total_accounts = self.accounts_by_address.read().len();
        let validators = self.validators_by_address.read();

        let latest_height = blocks.keys().next_back().copied().unwrap_or(0);
        let latest_block = blocks.get(&latest_height);
        let latest_block_hash = latest_block.map(|b| b.hash).unwrap_or(Hash32::ZERO);
        let latest_state_root = latest_block.map(|b| b.state_root).unwrap_or(Hash32::ZERO);

        let active_val_count = validators.values().filter(|v| v.status == "Active").count();
        let total_bonded_atto: u128 = validators.values().map(|v| v.tokens.as_atto()).sum();

        NetworkStats {
            chain_id: self.chain_id.read().clone(),
            latest_height,
            latest_block_hash,
            total_transactions: total_txs,
            total_accounts,
            active_validators_count: active_val_count,
            total_bonded_tokens: Amount::from_atto(total_bonded_atto),
            avg_block_time_seconds: 1.50,
            current_tps: if total_txs > 0 {
                (total_txs as f64) / ((latest_height + 1) as f64 * 1.5)
            } else {
                0.0
            },
            latest_state_root,
        }
    }

    /// Universal Search resolution (height, block hash, tx hash, address, moniker).
    pub fn search(&self, query: &str) -> Option<SearchResult> {
        let q = query.trim();
        if q.is_empty() {
            return None;
        }

        // 1. Try integer block height
        if let Ok(height) = q.parse::<u64>() {
            if let Some(b) = self.get_block_by_height(height) {
                return Some(SearchResult::Block(b));
            }
        }

        // 2. Try 32-byte Hex Hash (Block Hash or Tx Hash)
        if let Ok(hash) = Hash32::from_hex(q) {
            if let Some(b) = self.get_block_by_hash(&hash) {
                return Some(SearchResult::Block(b));
            }
            if let Some(t) = self.get_tx_by_hash(&hash) {
                return Some(SearchResult::Transaction(t));
            }
        }

        // 3. Try Address (Bech32 or 0x Hex)
        if let Ok(addr) = Address::parse(q) {
            if let Some(acc) = self.get_account(&addr) {
                return Some(SearchResult::Address(acc));
            }
        }

        // 4. Try Validator Moniker search
        let val_guard = self.validators_by_address.read();
        for v in val_guard.values() {
            if v.moniker.eq_ignore_ascii_case(q) {
                return Some(SearchResult::Validator(v.clone()));
            }
        }

        None
    }
}
