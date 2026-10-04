//! Consensus storage and address adapters for the actual CosmWasm VM.
use cosmwasm_std::{Binary, ContractResult, Order, Record, SystemError, SystemResult};
use cosmwasm_vm::{BackendApi, BackendError, BackendResult, GasInfo, Querier, Storage};
use sprax_storage::KVStore;
use sprax_types::Address;
use std::collections::BTreeMap;

pub const HOST_GAS_PER_BYTE: u64 = 100_000;

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
        let pairs = match self.store.scan_prefix(&self.prefix) {
            Ok(pairs) => pairs,
            Err(e) => return (Err(BackendError::unknown(e.to_string())), Self::cost(0)),
        };
        let mut records: Vec<Record> = pairs
            .into_iter()
            .map(|(key, value)| (key[self.prefix.len()..].to_vec(), value))
            .filter(|(key, _)| {
                start.is_none_or(|s| key.as_slice() >= s)
                    && end.is_none_or(|e| key.as_slice() < e)
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
                (Ok(record), Self::cost(bytes))
            }
            None => (
                Err(BackendError::iterator_does_not_exist(iterator_id)),
                Self::cost(0),
            ),
        }
    }
}

/// Unsupported chain queries fail explicitly; they never return invented balances.
#[derive(Debug)]
pub struct ChainQuerier;
impl Querier for ChainQuerier {
    fn query_raw(
        &self,
        _request: &[u8],
        _gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>> {
        (
            Ok(SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "chain query is not supported by this execution profile".into(),
            })),
            GasInfo::with_cost(1_000_000),
        )
    }
}
