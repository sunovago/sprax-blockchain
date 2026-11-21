//! Consensus storage and address adapters for the actual CosmWasm VM.
use cosmwasm_std::{
    to_json_binary, BankQuery, Binary, Checksum, Coin, ContractResult, Empty, Order, QueryRequest,
    Record, SystemError, SystemResult, WasmQuery,
};

use cosmwasm_vm::{BackendApi, BackendError, BackendResult, GasInfo, Querier, Storage};
use sprax_storage::KVStore;
use sprax_types::{account_key, supply_key, AccountState, Address, SupplyState};
use std::collections::BTreeMap;

pub const HOST_GAS_PER_BYTE: u64 = 100_000;
pub const MAX_STORAGE_KEY_BYTES: usize = 1024;
pub const MAX_STORAGE_VALUE_BYTES: usize = 64 * 1024;
pub const MAX_SCAN_RECORDS: usize = 1024;
pub const MAX_SCAN_BYTES: usize = 1024 * 1024;
pub const MAX_ITERATORS: usize = 16;
pub const MAX_QUERY_DEPTH: u32 = 16;

#[derive(Debug, Clone)]
pub struct ChainApi;

impl BackendApi for ChainApi {
    fn addr_validate(&self, input: &str) -> BackendResult<()> {
        let valid = Address::from_bech32(input)
            .and_then(|address| address.to_bech32())
            .map_err(|e| BackendError::user_err(e.to_string()))
            .and_then(|canonical| {
                if canonical == input {
                    Ok(())
                } else {
                    Err(BackendError::user_err(
                        "address must be canonical lowercase bech32",
                    ))
                }
            });
        (valid, GasInfo::with_cost(1_000_000))
    }
    fn addr_canonicalize(&self, input: &str) -> BackendResult<Vec<u8>> {
        (
            Address::from_bech32(input)
                .map(|a| a.as_bytes().to_vec())
                .map_err(|e| BackendError::user_err(e.to_string())),
            GasInfo::with_cost(1_000_000),
        )
    }
    fn addr_humanize(&self, input: &[u8]) -> BackendResult<String> {
        (
            Address::from_slice(input)
                .and_then(|a| a.to_bech32())
                .map_err(|e| BackendError::user_err(e.to_string())),
            GasInfo::with_cost(1_000_000),
        )
    }
}

#[derive(Debug)]
pub struct ChainStorage<S> {
    store: S,
    prefix: Vec<u8>,
    readonly: bool,
    iterators: BTreeMap<u32, std::vec::IntoIter<Record>>,
    next_iterator: u32,
}

impl<S: KVStore> ChainStorage<S> {
    pub fn new(store: S, contract: Address, readonly: bool) -> Self {
        let mut prefix = b"wasm/state/".to_vec();
        prefix.extend_from_slice(contract.as_bytes());
        Self {
            store,
            prefix,
            readonly,
            iterators: BTreeMap::new(),
            next_iterator: 1,
        }
    }
    fn key(&self, key: &[u8]) -> Vec<u8> {
        let mut scoped = self.prefix.clone();
        scoped.extend_from_slice(key);
        scoped
    }
    fn cost(bytes: usize) -> GasInfo {
        GasInfo::with_cost(
            (bytes as u64)
                .saturating_mul(HOST_GAS_PER_BYTE)
                .saturating_add(1_000_000),
        )
    }
}

impl<S: KVStore> Storage for ChainStorage<S> {
    fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>> {
        if key.len() > MAX_STORAGE_KEY_BYTES {
            return (
                Err(BackendError::user_err("storage key exceeds size limit")),
                Self::cost(key.len()),
            );
        }
        let result = self
            .store
            .get(&self.key(key))
            .map_err(|e| BackendError::unknown(e.to_string()));
        let bytes = result
            .as_ref()
            .ok()
            .and_then(|v| v.as_ref())
            .map_or(0, Vec::len);
        (result, Self::cost(key.len() + bytes))
    }
    fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()> {
        if key.len() > MAX_STORAGE_KEY_BYTES || value.len() > MAX_STORAGE_VALUE_BYTES {
            return (
                Err(BackendError::user_err(
                    "storage key or value exceeds size limit",
                )),
                Self::cost(key.len() + value.len()),
            );
        }
        if self.readonly {
            return (
                Err(BackendError::user_err("query storage is read-only")),
                Self::cost(key.len()),
            );
        }
        (
            self.store
                .set(&self.key(key), value)
                .map_err(|e| BackendError::unknown(e.to_string())),
            Self::cost(key.len() + value.len()),
        )
    }
    fn remove(&mut self, key: &[u8]) -> BackendResult<()> {
        if self.readonly {
            return (
                Err(BackendError::user_err("query storage is read-only")),
                Self::cost(key.len()),
            );
        }
        (
            self.store
                .delete(&self.key(key))
                .map_err(|e| BackendError::unknown(e.to_string())),
            Self::cost(key.len()),
        )
    }
    fn scan(
        &mut self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> BackendResult<u32> {
        if self.iterators.len() >= MAX_ITERATORS
            || start.is_some_and(|s| s.len() > MAX_STORAGE_KEY_BYTES)
            || end.is_some_and(|e| e.len() > MAX_STORAGE_KEY_BYTES)
        {
            return (
                Err(BackendError::user_err("iterator resource limit exceeded")),
                Self::cost(0),
            );
        }
        if start.zip(end).is_some_and(|(start, end)| start >= end) {
            let id = self.next_iterator;
            let Some(next) = id.checked_add(1) else {
                return (
                    Err(BackendError::unknown("iterator limit reached")),
                    Self::cost(0),
                );
            };
            self.next_iterator = next;
            self.iterators.insert(id, Vec::new().into_iter());
            return (Ok(id), Self::cost(0));
        }
        let lower = start.map_or_else(|| self.prefix.clone(), |start| self.key(start));
        let upper = end
            .map(|end| self.key(end))
            .or_else(|| sprax_storage::prefix_upper_bound(&self.prefix));
        let pairs = match self.store.scan_range_bounded(
            &lower,
            upper.as_deref(),
            MAX_SCAN_RECORDS,
            MAX_SCAN_BYTES,
        ) {
            Ok(pairs) => pairs,
            Err(e) => {
                return (
                    Err(BackendError::unknown(e.to_string())),
                    Self::cost(MAX_SCAN_BYTES),
                )
            }
        };
        let mut records: Vec<Record> = pairs
            .into_iter()
            .map(|(key, value)| (key[self.prefix.len()..].to_vec(), value))
            .filter(|(key, _)| {
                start.is_none_or(|s| key.as_slice() >= s) && end.is_none_or(|e| key.as_slice() < e)
            })
            .collect();
        let cost = Self::cost(records.iter().map(|(k, v)| k.len() + v.len()).sum());
        if order == Order::Descending {
            records.reverse();
        }
        let id = self.next_iterator;
        let Some(next) = id.checked_add(1) else {
            return (Err(BackendError::unknown("iterator limit reached")), cost);
        };
        self.next_iterator = next;
        self.iterators.insert(id, records.into_iter());
        (Ok(id), cost)
    }
    fn next(&mut self, iterator_id: u32) -> BackendResult<Option<Record>> {
        match self.iterators.get_mut(&iterator_id) {
            Some(iterator) => {
                let record = iterator.next();
                let bytes = record.as_ref().map_or(0, |(k, v)| k.len() + v.len());
                if record.is_none() {
                    self.iterators.remove(&iterator_id);
                }
                (Ok(record), Self::cost(bytes))
            }
            None if iterator_id > 0 && iterator_id < self.next_iterator => {
                (Ok(None), Self::cost(0))
            }
            None => (
                Err(BackendError::iterator_does_not_exist(iterator_id)),
                Self::cost(0),
            ),
        }
    }
}

/// Fully deterministic chain querier providing read-only access to state, balances, supplies, and nested contract queries.
#[derive(Debug, Clone)]
pub struct ChainQuerier<S> {
    store: S,
    chain_id: String,
    height: u64,
    timestamp: u64,
    depth: u32,
}

impl<S: KVStore + Clone + 'static> ChainQuerier<S> {
    pub fn new(store: S, chain_id: String, height: u64, timestamp: u64, depth: u32) -> Self {
        Self {
            store,
            chain_id,
            height,
            timestamp,
            depth,
        }
    }
}

impl<S: KVStore + Clone + 'static> Querier for ChainQuerier<S> {
    fn query_raw(
        &self,
        request: &[u8],
        gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>> {
        let req_bin = Binary::from(request);
        let parsed: QueryRequest<Empty> = match serde_json::from_slice(request) {
            Ok(p) => p,
            Err(e) => {
                return (
                    Ok(SystemResult::Err(SystemError::InvalidRequest {
                        error: format!("parse QueryRequest failed: {e}"),
                        request: req_bin,
                    })),
                    GasInfo::with_cost(1_000_000),
                );
            }
        };

        match parsed {
            QueryRequest::Bank(bank_query) => match bank_query {
                BankQuery::Balance { address, denom } => {
                    let Ok(target) = Address::from_bech32(&address) else {
                        return (
                            Ok(SystemResult::Err(SystemError::InvalidRequest {
                                error: "invalid bech32 address".into(),
                                request: req_bin,
                            })),
                            GasInfo::with_cost(1_000_000),
                        );
                    };
                    let amount = if denom == "asprx" {
                        let key = account_key(&target);
                        match self.store.get(&key) {
                            Ok(Some(bytes)) => {
                                match serde_json::from_slice::<AccountState>(&bytes) {
                                    Ok(acc) => acc.balance.as_atto(),
                                    Err(_) => 0u128,
                                }
                            }
                            _ => 0u128,
                        }
                    } else {
                        0u128
                    };
                    let serialized = match to_json_binary(&serde_json::json!({
                        "amount": Coin::new(amount, denom)
                    })) {
                        Ok(bin) => bin,
                        Err(_) => {
                            return (
                                Ok(SystemResult::Err(SystemError::Unknown {})),
                                GasInfo::with_cost(1_000_000),
                            );
                        }
                    };
                    (
                        Ok(SystemResult::Ok(ContractResult::Ok(serialized))),
                        GasInfo::with_cost(1_000_000),
                    )
                }

                BankQuery::Supply { denom } => {
                    let amount = if denom == "asprx" {
                        let key = supply_key();
                        match self.store.get(&key) {
                            Ok(Some(bytes)) => {
                                match serde_json::from_slice::<SupplyState>(&bytes) {
                                    Ok(s) => s.circulating_supply.as_atto(),
                                    Err(_) => 0u128,
                                }
                            }
                            _ => 0u128,
                        }
                    } else {
                        0u128
                    };
                    let serialized = match to_json_binary(&serde_json::json!({
                        "amount": Coin::new(amount, denom)
                    })) {
                        Ok(bin) => bin,
                        Err(_) => {
                            return (
                                Ok(SystemResult::Err(SystemError::Unknown {})),
                                GasInfo::with_cost(1_000_000),
                            );
                        }
                    };
                    (
                        Ok(SystemResult::Ok(ContractResult::Ok(serialized))),
                        GasInfo::with_cost(1_000_000),
                    )
                }
                _ => (
                    Ok(SystemResult::Err(SystemError::UnsupportedRequest {
                        kind: "bank query variant not supported".into(),
                    })),
                    GasInfo::with_cost(1_000_000),
                ),
            },
            QueryRequest::Wasm(wasm_query) => match wasm_query {
                WasmQuery::Smart { contract_addr, msg } => {
                    let Ok(target) = Address::from_bech32(&contract_addr) else {
                        return (
                            Ok(SystemResult::Err(SystemError::InvalidRequest {
                                error: "invalid bech32 address".into(),
                                request: req_bin,
                            })),
                            GasInfo::with_cost(1_000_000),
                        );
                    };
                    if self.depth >= MAX_QUERY_DEPTH {
                        return (
                            Ok(SystemResult::Err(SystemError::UnsupportedRequest {
                                kind: "max query depth exceeded".into(),
                            })),
                            GasInfo::with_cost(1_000_000),
                        );
                    }
                    let dyn_store: crate::runtime::DynStore =
                        std::sync::Arc::new(self.store.clone());
                    let res = crate::runtime::CosmWasmRuntime::query_internal(
                        &dyn_store,
                        target,
                        &self.chain_id,
                        self.height,
                        self.timestamp,
                        &msg,
                        gas_limit / crate::runtime::VM_GAS_PER_CHAIN_GAS,
                        self.depth + 1,
                    );

                    match res {
                        Ok(result) => (
                            Ok(SystemResult::Ok(ContractResult::Ok(Binary::new(
                                result.data,
                            )))),
                            GasInfo::with_cost(
                                result
                                    .gas_used
                                    .saturating_mul(crate::runtime::VM_GAS_PER_CHAIN_GAS),
                            ),
                        ),
                        Err(e) => (
                            Ok(SystemResult::Ok(ContractResult::Err(e))),
                            GasInfo::with_cost(1_000_000),
                        ),
                    }
                }
                WasmQuery::Raw { contract_addr, key } => {
                    let Ok(target) = Address::from_bech32(&contract_addr) else {
                        return (
                            Ok(SystemResult::Err(SystemError::InvalidRequest {
                                error: "invalid bech32 address".into(),
                                request: req_bin,
                            })),
                            GasInfo::with_cost(1_000_000),
                        );
                    };
                    let mut storage_key = b"wasm/state/".to_vec();
                    storage_key.extend_from_slice(target.as_bytes());
                    storage_key.extend_from_slice(key.as_slice());
                    let val = match self.store.get(&storage_key) {
                        Ok(v) => v,
                        Err(_) => {
                            return (
                                Ok(SystemResult::Err(SystemError::Unknown {})),
                                GasInfo::with_cost(1_000_000),
                            );
                        }
                    };
                    let bin = match val {
                        Some(bytes) => Binary::new(bytes),
                        None => Binary::default(),
                    };
                    let serialized = match to_json_binary(&bin) {
                        Ok(b) => b,
                        Err(_) => {
                            return (
                                Ok(SystemResult::Err(SystemError::Unknown {})),
                                GasInfo::with_cost(1_000_000),
                            );
                        }
                    };
                    (
                        Ok(SystemResult::Ok(ContractResult::Ok(serialized))),
                        GasInfo::with_cost(1_000_000),
                    )
                }
                WasmQuery::ContractInfo { contract_addr } => {
                    let Ok(target) = Address::from_bech32(&contract_addr) else {
                        return (
                            Ok(SystemResult::Err(SystemError::InvalidRequest {
                                error: "invalid bech32 address".into(),
                                request: req_bin,
                            })),
                            GasInfo::with_cost(1_000_000),
                        );
                    };
                    let mut meta_key = b"wasm/contract/".to_vec();
                    meta_key.extend_from_slice(target.as_bytes());
                    let metadata = match self.store.get(&meta_key) {
                        Ok(Some(bytes)) => {
                            match serde_json::from_slice::<crate::runtime::RuntimeContractInfo>(
                                &bytes,
                            ) {
                                Ok(m) => m,
                                Err(_) => {
                                    return (
                                        Ok(SystemResult::Err(SystemError::Unknown {})),
                                        GasInfo::with_cost(1_000_000),
                                    );
                                }
                            }
                        }
                        Ok(None) => {
                            return (
                                Ok(SystemResult::Err(SystemError::NoSuchContract {
                                    addr: contract_addr,
                                })),
                                GasInfo::with_cost(1_000_000),
                            );
                        }
                        Err(_) => {
                            return (
                                Ok(SystemResult::Err(SystemError::Unknown {})),
                                GasInfo::with_cost(1_000_000),
                            );
                        }
                    };
                    let creator_str = metadata.creator.to_bech32().unwrap_or_default();
                    let admin_str = metadata.admin.as_ref().and_then(|a| a.to_bech32().ok());
                    let serialized = match to_json_binary(&serde_json::json!({
                        "code_id": metadata.code_id_seq,
                        "creator": creator_str,
                        "admin": admin_str,
                        "pinned": false,
                        "ibc_port": serde_json::Value::Null,
                    })) {
                        Ok(b) => b,
                        Err(_) => {
                            return (
                                Ok(SystemResult::Err(SystemError::Unknown {})),
                                GasInfo::with_cost(1_000_000),
                            );
                        }
                    };
                    (
                        Ok(SystemResult::Ok(ContractResult::Ok(serialized))),
                        GasInfo::with_cost(1_000_000),
                    )
                }
                WasmQuery::CodeInfo { code_id } => {
                    let mut code_id_key = b"wasm/code_by_id/".to_vec();
                    code_id_key.extend_from_slice(&code_id.to_be_bytes());
                    let code_hash_bytes = match self.store.get(&code_id_key) {
                        Ok(Some(bytes)) => bytes,
                        _ => {
                            return (
                                Ok(SystemResult::Err(SystemError::NoSuchCode { code_id })),
                                GasInfo::with_cost(1_000_000),
                            );
                        }
                    };
                    let mut code_key = b"wasm/code/".to_vec();
                    code_key.extend_from_slice(&code_hash_bytes);
                    let code_bytes = match self.store.get(&code_key) {
                        Ok(Some(bytes)) => bytes,
                        _ => {
                            return (
                                Ok(SystemResult::Err(SystemError::NoSuchCode { code_id })),
                                GasInfo::with_cost(1_000_000),
                            );
                        }
                    };
                    let checksum = Checksum::generate(&code_bytes);
                    let serialized = match to_json_binary(&serde_json::json!({
                        "code_id": code_id,
                        "creator": "",
                        "checksum": checksum,
                    })) {
                        Ok(b) => b,
                        Err(_) => {
                            return (
                                Ok(SystemResult::Err(SystemError::Unknown {})),
                                GasInfo::with_cost(1_000_000),
                            );
                        }
                    };
                    (
                        Ok(SystemResult::Ok(ContractResult::Ok(serialized))),
                        GasInfo::with_cost(1_000_000),
                    )
                }
                _ => (
                    Ok(SystemResult::Err(SystemError::UnsupportedRequest {
                        kind: "unsupported wasm query variant".into(),
                    })),
                    GasInfo::with_cost(1_000_000),
                ),
            },

            _ => (
                Ok(SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: "query type not supported by SPRX execution profile".into(),
                })),
                GasInfo::with_cost(1_000_000),
            ),
        }
    }
}
