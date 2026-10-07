//! Actual metered WASM execution. All consensus data lives in the caller's store;
//! compiled instances are disposable and never determine code IDs or addresses.
use crate::backend::{ChainApi, ChainQuerier, ChainStorage};
use cosmwasm_std::{
    Addr, BlockInfo, Coin, ContractInfo, Empty, Env, MessageInfo, Response, Timestamp,
};
use cosmwasm_vm::{Backend, Instance, InstanceOptions, Size};
use serde::{Deserialize, Serialize};
use sprax_crypto::Hasher;
use sprax_storage::KVStore;
use sprax_types::{Address, Amount, Hash32};

pub const VM_GAS_PER_CHAIN_GAS: u64 = 1_000_000;
pub const MAX_WASM_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeContractInfo {
    pub code_id: Hash32,
    pub creator: Address,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct ContractContext {
    pub chain_id: String,
    pub height: u64,
    pub timestamp: u64,
    pub sender: Address,
    pub nonce: u64,
    pub message_index: u32,
}

#[derive(Debug)]
pub struct RuntimeResult {
    pub data: Vec<u8>,
    pub gas_used: u64,
}

#[derive(Debug, Clone, Default)]
pub struct CosmWasmRuntime;

type ChainInstance<S> = Instance<ChainApi, ChainStorage<S>, ChainQuerier>;

impl CosmWasmRuntime {
    fn key(prefix: &[u8], bytes: &[u8]) -> Vec<u8> {
        let mut key = prefix.to_vec();
        key.extend_from_slice(bytes);
        key
    }
    fn instance<S: KVStore + Clone + 'static>(
        store: &S,
        code: &[u8],
        address: Address,
        gas: u64,
        readonly: bool,
    ) -> Result<ChainInstance<S>, String> {
        let gas_limit = gas
            .checked_mul(VM_GAS_PER_CHAIN_GAS)
            .ok_or("VM gas limit overflow")?;
        Instance::from_code(
            code,
            Backend {
                api: ChainApi,
                storage: ChainStorage::new(store.clone(), address, readonly),
                querier: ChainQuerier,
            },
            InstanceOptions { gas_limit },
            Some(Size::mebi(64)),
        )
        .map_err(|e| e.to_string())
    }
    pub fn store_code<S: KVStore + Clone + 'static>(
        &self,
        store: &S,
        code: &[u8],
        gas: u64,
    ) -> Result<(Hash32, u64), String> {
        if code.len() > MAX_WASM_BYTES || !code.starts_with(b"\0asm\x01\0\0\0") {
            return Err("invalid or oversized WASM module".into());
        }
        let mut exports = std::collections::HashSet::new();
        for payload in wasmparser::Parser::new(0).parse_all(code) {
            match payload.map_err(|e| e.to_string())? {
                wasmparser::Payload::ExportSection(section) => {
                    for export in section {
                        let export = export.map_err(|e| e.to_string())?;
                        exports.insert(export.name.to_owned());
                    }
                }
                wasmparser::Payload::StartSection { .. } => {
                    return Err("WASM start functions are forbidden".into())
                }
                wasmparser::Payload::MemorySection(section) => {
                    for memory in section {
                        let memory = memory.map_err(|e| e.to_string())?;
                        if memory.memory64 || memory.shared || memory.initial > 1024 {
                            return Err(
                                "WASM memory exceeds the supported execution profile".into()
                            );
                        }
                    }
                }
                _ => {}
            }
        }
        for required in [
            "allocate",
            "deallocate",
            "interface_version_8",
            "memory",
            "instantiate",
            "execute",
            "query",
        ] {
            if !exports.contains(required) {
                return Err(format!("missing CosmWasm export: {required}"));
            }
        }
        let charge = (code.len() as u64)
            .checked_mul(10)
            .ok_or("upload gas overflow")?;
        if charge > gas {
            return Err("out of gas storing WASM code".into());
        }
        let _validated = Self::instance(store, code, Address::ZERO, gas - charge, true)?;
        let code_id = Hasher::sha256(code);
        store
            .set(&Self::key(b"wasm/code/", code_id.as_bytes()), code)
            .map_err(|e| e.to_string())?;
        Ok((code_id, charge))
    }
    pub fn contract_address(context: &ContractContext, code_id: Hash32) -> Address {
        let mut preimage = b"sprax/contract/v1/".to_vec();
        preimage.extend_from_slice(&(context.chain_id.len() as u64).to_be_bytes());
        preimage.extend_from_slice(context.chain_id.as_bytes());
        preimage.extend_from_slice(context.sender.as_bytes());
        preimage.extend_from_slice(&context.nonce.to_be_bytes());
        preimage.extend_from_slice(&context.message_index.to_be_bytes());
        preimage.extend_from_slice(code_id.as_bytes());
        let hash = Hasher::sha256(&preimage);
        let mut address = [0; 20];
        address.copy_from_slice(&hash.as_bytes()[..20]);
        Address::new(address)
    }
    fn code<S: KVStore>(store: &S, id: Hash32) -> Result<Vec<u8>, String> {
        let code = store
            .get(&Self::key(b"wasm/code/", id.as_bytes()))
            .map_err(|e| e.to_string())?
            .ok_or("contract code not found")?;
        if Hasher::sha256(&code) != id {
            return Err("stored code checksum mismatch".into());
        }
        Ok(code)
    }
    pub fn contract_info<S: KVStore>(
        store: &S,
        address: Address,
    ) -> Result<RuntimeContractInfo, String> {
        let bytes = store
            .get(&Self::key(b"wasm/contract/", address.as_bytes()))
            .map_err(|e| e.to_string())?
            .ok_or("contract not found")?;
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())
    }
    fn env(context: &ContractContext, address: Address) -> Result<Env, String> {
        Ok(Env {
            block: BlockInfo {
                height: context.height,
                time: Timestamp::from_nanos(
                    context
                        .timestamp
                        .checked_mul(1_000_000_000)
                        .ok_or("block timestamp exceeds WASM timestamp range")?,
                ),
                chain_id: context.chain_id.clone(),
            },
            transaction: None,
            contract: ContractInfo {
                address: Addr::unchecked(address.to_bech32().map_err(|e| e.to_string())?),
            },
        })
    }
    fn info(sender: Address, funds: Amount) -> Result<MessageInfo, String> {
        Ok(MessageInfo {
            sender: Addr::unchecked(sender.to_bech32().map_err(|e| e.to_string())?),
            funds: if funds.is_zero() {
                vec![]
            } else {
                vec![Coin::new(funds.as_atto(), "asprx")]
            },
        })
    }
    fn result<S: KVStore + Clone + 'static>(
        instance: &mut ChainInstance<S>,
        response: Response<Empty>,
    ) -> Result<RuntimeResult, String> {
        if !response.messages.is_empty() {
            return Err("contract submessages are not supported yet".into());
        }
        let report = instance.create_gas_report();
        let consumed = report.limit - report.remaining;
        Ok(RuntimeResult {
            data: response.data.map_or_else(Vec::new, |data| data.to_vec()),
            gas_used: consumed.div_ceil(VM_GAS_PER_CHAIN_GAS),
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub fn instantiate<S: KVStore + Clone + 'static>(
        &self,
        store: &S,
        code_id: Hash32,
        context: &ContractContext,
        msg: &[u8],
        label: &str,
        funds: Amount,
        gas: u64,
    ) -> Result<(Address, RuntimeResult), String> {
        if label.is_empty() || label.len() > 128 {
            return Err("contract label must contain 1 to 128 bytes".into());
        }
        let address = Self::contract_address(context, code_id);
        let metadata_key = Self::key(b"wasm/contract/", address.as_bytes());
        if store.has(&metadata_key).map_err(|e| e.to_string())? {
            return Err("contract address already exists".into());
        }
        let code = Self::code(store, code_id)?;
        let mut instance = Self::instance(store, &code, address, gas, false)?;
        let response: Response<Empty> = cosmwasm_vm::call_instantiate(
            &mut instance,
            &Self::env(context, address)?,
            &Self::info(context.sender, funds)?,
            msg,
        )
        .map_err(|e| e.to_string())?
        .into_result()
        .map_err(|e| e.to_string())?;
        let result = Self::result(&mut instance, response)?;
        let bytes = serde_json::to_vec(&RuntimeContractInfo {
            code_id,
            creator: context.sender,
            label: label.into(),
        })
        .map_err(|e| e.to_string())?;
        store
            .set(&metadata_key, &bytes)
            .map_err(|e| e.to_string())?;
        Ok((address, result))
    }
    pub fn execute<S: KVStore + Clone + 'static>(
        &self,
        store: &S,
        address: Address,
        context: &ContractContext,
        msg: &[u8],
        funds: Amount,
        gas: u64,
    ) -> Result<RuntimeResult, String> {
        let metadata = Self::contract_info(store, address)?;
        let code = Self::code(store, metadata.code_id)?;
        let mut instance = Self::instance(store, &code, address, gas, false)?;
        let response: Response<Empty> = cosmwasm_vm::call_execute(
            &mut instance,
            &Self::env(context, address)?,
            &Self::info(context.sender, funds)?,
            msg,
        )
        .map_err(|e| e.to_string())?
        .into_result()
        .map_err(|e| e.to_string())?;
        Self::result(&mut instance, response)
    }
    pub fn query<S: KVStore + Clone + 'static>(
        &self,
        store: &S,
        address: Address,
        context: &ContractContext,
        msg: &[u8],
        gas: u64,
    ) -> Result<RuntimeResult, String> {
        let metadata = Self::contract_info(store, address)?;
        let code = Self::code(store, metadata.code_id)?;
        let mut instance = Self::instance(store, &code, address, gas, true)?;
        let result = cosmwasm_vm::call_query(&mut instance, &Self::env(context, address)?, msg)
            .map_err(|e| e.to_string())?
            .into_result()
            .map_err(|e| e.to_string())?;
        let report = instance.create_gas_report();
        Ok(RuntimeResult {
            data: result.to_vec(),
            gas_used: (report.limit - report.remaining).div_ceil(VM_GAS_PER_CHAIN_GAS),
        })
    }
}
