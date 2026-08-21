use crate::{
    error::IndexerError,
    models::{
        IndexedAccount, IndexedBlock, IndexedTx, IndexedValidator, NetworkStats, PaginatedResponse,
        SearchResult,
    },
    storage::IndexStore,
};
use sprax_types::{Address, Hash32};
use std::sync::Arc;

pub const MAX_PAGE_LIMIT: usize = 100;
pub const DEFAULT_PAGE_LIMIT: usize = 20;

/// REST / JSON Explorer API query layer.
#[derive(Debug, Clone)]
pub struct ExplorerApi {
    store: Arc<IndexStore>,
}

impl ExplorerApi {
    pub fn new(store: Arc<IndexStore>) -> Self {
        Self { store }
    }

    /// Fetches paginated blocks list (ordered descending by height).
    pub fn get_blocks(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> PaginatedResponse<IndexedBlock> {
        let page_limit = limit.unwrap_or(DEFAULT_PAGE_LIMIT).min(MAX_PAGE_LIMIT);
        let page_offset = offset.unwrap_or(0);
        self.store.get_blocks_paginated(page_limit, page_offset)
    }

    /// Fetches a single block by integer height or 32-byte hex hash.
    pub fn get_block(&self, identifier: &str) -> Result<IndexedBlock, IndexerError> {
        let clean = identifier.trim();
        if let Ok(height) = clean.parse::<u64>() {
            return self
                .store
                .get_block_by_height(height)
                .ok_or_else(|| IndexerError::BlockNotFound(format!("height #{height}")));
        }
        if let Ok(hash) = Hash32::from_hex(clean) {
            return self
                .store
                .get_block_by_hash(&hash)
                .ok_or_else(|| IndexerError::BlockNotFound(format!("hash {hash}")));
        }
        Err(IndexerError::InvalidQuery(format!(
            "invalid block identifier '{identifier}' (expected height integer or 0x hex hash)"
        )))
    }

    /// Fetches paginated transactions list (ordered descending by chronological submission).
    pub fn get_transactions(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> PaginatedResponse<IndexedTx> {
        let page_limit = limit.unwrap_or(DEFAULT_PAGE_LIMIT).min(MAX_PAGE_LIMIT);
        let page_offset = offset.unwrap_or(0);
        self.store.get_txs_paginated(page_limit, page_offset)
    }

    /// Fetches transaction details and receipt by 32-byte hex hash.
    pub fn get_transaction(&self, tx_hash_str: &str) -> Result<IndexedTx, IndexerError> {
        let hash = Hash32::from_hex(tx_hash_str.trim())
            .map_err(|e| IndexerError::InvalidQuery(format!("invalid tx hash format: {e}")))?;
        self.store
            .get_tx_by_hash(&hash)
            .ok_or_else(|| IndexerError::TxNotFound(format!("{hash}")))
    }

    /// Fetches address profile, balance, and sequence nonce.
    pub fn get_address(&self, address_str: &str) -> Result<IndexedAccount, IndexerError> {
        let addr = Address::parse(address_str.trim())
            .map_err(|e| IndexerError::InvalidQuery(format!("invalid address format: {e}")))?;
        self.store
            .get_account(&addr)
            .ok_or_else(|| IndexerError::AddressNotFound(format!("{addr}")))
    }

    /// Fetches paginated transaction history for an address.
    pub fn get_address_transactions(
        &self,
        address_str: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<PaginatedResponse<IndexedTx>, IndexerError> {
        let addr = Address::parse(address_str.trim())
            .map_err(|e| IndexerError::InvalidQuery(format!("invalid address format: {e}")))?;
        let page_limit = limit.unwrap_or(DEFAULT_PAGE_LIMIT).min(MAX_PAGE_LIMIT);
        let page_offset = offset.unwrap_or(0);
        Ok(self
            .store
            .get_address_txs_paginated(&addr, page_limit, page_offset))
    }

    /// Fetches all active validators sorted by voting power.
    pub fn get_validators(&self) -> Vec<IndexedValidator> {
        self.store.get_validators()
    }

    /// Fetches live network metrics and TPS stats.
    pub fn get_stats(&self) -> NetworkStats {
        self.store.get_network_stats()
    }

    /// Performs universal omni-search matching height, block hash, tx hash, address, or moniker.
    pub fn search(&self, query: &str) -> Option<SearchResult> {
        self.store.search(query)
    }
}
