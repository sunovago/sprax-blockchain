//! Actual metered WASM execution. All consensus data lives in the caller's store;
//! compiled instances are disposable and never determine code IDs or addresses.
use crate::backend::{ChainApi, ChainQuerier, ChainStorage};
use cosmwasm_std::{
    Addr, BankMsg, Binary, BlockInfo, Coin, ContractInfo, CosmosMsg, Empty, Env, MessageInfo,
    Reply, ReplyOn, Response, SubMsgResponse, SubMsgResult, Timestamp, WasmMsg,
};
use cosmwasm_vm::{
    call_execute, call_instantiate, call_migrate, call_query, call_reply, Backend, Instance,
    InstanceOptions, Size,
};
use serde::{Deserialize, Serialize};
use sprax_crypto::Hasher;
use sprax_storage::{KVStore, OverlayStore, ReadonlyKVStore};
use sprax_types::{account_key, supply_key, AccountState, Address, Amount, Hash32, SupplyState};
use std::sync::Arc;

pub const VM_GAS_PER_CHAIN_GAS: u64 = 1_000_000;
pub const MAX_WASM_BYTES: usize = 512 * 1024;
pub const MAX_CALL_DEPTH: u32 = 16;
pub const MAX_TOTAL_SUBMESSAGES: u32 = 100;
pub const MAX_EVENT_COUNT: usize = 128;
pub const MAX_EVENT_ATTRIBUTES: usize = 64;
pub const MAX_ATTRIBUTE_KEY_LEN: usize = 128;
pub const MAX_ATTRIBUTE_VALUE_LEN: usize = 1024;
pub const SUBMSG_BASE_GAS_COST: u64 = 10_000;

pub type DynStore = Arc<dyn KVStore>;
pub type ChainInstance = Instance<ChainApi, ChainStorage<DynStore>, ChainQuerier<DynStore>>;

pub fn coin_amount_to_u128(amount: &cosmwasm_std::Uint256) -> Result<u128, String> {
    amount
        .to_string()
        .parse::<u128>()
        .map_err(|e| format!("amount exceeds u128: {e}"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeContractInfo {
    pub code_id: Hash32,
    #[serde(default)]
    pub code_id_seq: u64,
    pub creator: Address,
    #[serde(default)]
    pub admin: Option<Address>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeResult {
    pub data: Vec<u8>,
    pub gas_used: u64,
    pub events: Vec<sprax_types::ContractEvent>,
}

#[derive(Debug, Clone, Default)]
pub struct CosmWasmRuntime;

impl CosmWasmRuntime {
    fn key(prefix: &[u8], bytes: &[u8]) -> Vec<u8> {
        let mut key = prefix.to_vec();
        key.extend_from_slice(bytes);
        key
    }

    #[allow(clippy::too_many_arguments)]
    fn instance(
        store: &DynStore,
        code: &[u8],
        address: Address,
        chain_id: &str,
        height: u64,
        timestamp: u64,
        gas: u64,
        readonly: bool,
        depth: u32,
    ) -> Result<ChainInstance, String> {
        let gas_limit = gas
            .checked_mul(VM_GAS_PER_CHAIN_GAS)
            .ok_or("VM gas limit overflow")?;
        Instance::from_code(
            code,
            Backend {
                api: ChainApi,
                storage: ChainStorage::new(store.clone(), address, readonly),
                querier: ChainQuerier::new(
                    store.clone(),
                    chain_id.to_string(),
                    height,
                    timestamp,
                    depth,
                ),
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
        let dyn_store: DynStore = Arc::new(store.clone());
        self.store_code_dyn(&dyn_store, code, gas)
    }

    fn store_code_dyn(
        &self,
        store: &DynStore,
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
        let _validated = Self::instance(
            store,
            code,
            Address::ZERO,
            "sprax-1",
            1,
            0,
            gas - charge,
            true,
            0,
        )?;
        let code_id = Hasher::sha256(code);
        let hash_key = Self::key(b"wasm/code_by_hash/", code_id.as_bytes());
        let _code_seq = match store.get(&hash_key).map_err(|e| e.to_string())? {
            Some(bytes) => {
                let mut arr = [0u8; 8];
                if bytes.len() == 8 {
                    arr.copy_from_slice(&bytes);
                    u64::from_be_bytes(arr)
                } else {
                    1
                }
            }
            None => {
                let count_key = b"wasm/code_count";
                let current_count = match store.get(count_key).map_err(|e| e.to_string())? {
                    Some(bytes) => {
                        let mut arr = [0u8; 8];
                        if bytes.len() == 8 {
                            arr.copy_from_slice(&bytes);
                            u64::from_be_bytes(arr)
                        } else {
                            0
                        }
                    }
                    None => 0,
                };
                let next_count = current_count
                    .checked_add(1)
                    .ok_or("code id sequence overflow")?;
                store
                    .set(count_key, &next_count.to_be_bytes())
                    .map_err(|e| e.to_string())?;
                store
                    .set(
                        &Self::key(b"wasm/code_by_id/", &next_count.to_be_bytes()),
                        code_id.as_bytes(),
                    )
                    .map_err(|e| e.to_string())?;
                store
                    .set(&hash_key, &next_count.to_be_bytes())
                    .map_err(|e| e.to_string())?;
                next_count
            }
        };

        store
            .set(&Self::key(b"wasm/code/", code_id.as_bytes()), code)
            .map_err(|e| e.to_string())?;
        Ok((code_id, charge))
    }

    pub fn get_code_seq_by_hash(
        store: &dyn ReadonlyKVStore,
        hash: &Hash32,
    ) -> Result<Option<u64>, String> {
        let hash_key = Self::key(b"wasm/code_by_hash/", hash.as_bytes());
        match store.get(&hash_key).map_err(|e| e.to_string())? {
            Some(bytes) if bytes.len() == 8 => {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes);
                Ok(Some(u64::from_be_bytes(arr)))
            }
            _ => Ok(None),
        }
    }

    pub fn get_hash_by_code_seq(
        store: &dyn ReadonlyKVStore,
        seq: u64,
    ) -> Result<Option<Hash32>, String> {
        let id_key = Self::key(b"wasm/code_by_id/", &seq.to_be_bytes());
        match store.get(&id_key).map_err(|e| e.to_string())? {
            Some(bytes) if bytes.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(Some(Hash32::new(arr)))
            }
            _ => Ok(None),
        }
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

    fn code(store: &dyn ReadonlyKVStore, id: Hash32) -> Result<Vec<u8>, String> {
        let code = store
            .get(&Self::key(b"wasm/code/", id.as_bytes()))
            .map_err(|e| e.to_string())?
            .ok_or("contract code not found")?;
        if Hasher::sha256(&code) != id {
            return Err("stored code checksum mismatch".into());
        }
        Ok(code)
    }

    pub fn contract_info(
        store: &dyn ReadonlyKVStore,
        address: Address,
    ) -> Result<RuntimeContractInfo, String> {
        let bytes = store
            .get(&Self::key(b"wasm/contract/", address.as_bytes()))
            .map_err(|e| e.to_string())?
            .ok_or("contract not found")?;
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())
    }

    fn env(chain_id: &str, height: u64, timestamp: u64, address: Address) -> Result<Env, String> {
        Ok(Env {
            block: BlockInfo {
                height,
                time: Timestamp::from_nanos(
                    timestamp
                        .checked_mul(1_000_000_000)
                        .ok_or("block timestamp exceeds WASM timestamp range")?,
                ),
                chain_id: chain_id.to_string(),
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

    fn validate_and_convert_events(
        contract_addr: Address,
        response: &Response<Empty>,
    ) -> Result<Vec<sprax_types::ContractEvent>, String> {
        if response.events.len() > MAX_EVENT_COUNT {
            return Err("contract emitted too many events".into());
        }
        let mut events = Vec::new();
        if !response.attributes.is_empty() {
            if response.attributes.len() > MAX_EVENT_ATTRIBUTES {
                return Err("contract emitted too many default attributes".into());
            }
            let mut attrs = Vec::with_capacity(response.attributes.len());
            for attr in &response.attributes {
                if attr.key.len() > MAX_ATTRIBUTE_KEY_LEN
                    || attr.value.len() > MAX_ATTRIBUTE_VALUE_LEN
                {
                    return Err("attribute exceeds size limits".into());
                }
                attrs.push((attr.key.clone(), attr.value.clone()));
            }
            events.push(sprax_types::ContractEvent {
                contract: contract_addr,
                event_type: "wasm".into(),
                attributes: attrs,
            });
        }
        for ev in &response.events {
            if ev.ty.len() > MAX_ATTRIBUTE_KEY_LEN {
                return Err("event type exceeds size limit".into());
            }
            if ev.attributes.len() > MAX_EVENT_ATTRIBUTES {
                return Err("custom event has too many attributes".into());
            }
            let mut attrs = Vec::with_capacity(ev.attributes.len());
            for attr in &ev.attributes {
                if attr.key.len() > MAX_ATTRIBUTE_KEY_LEN
                    || attr.value.len() > MAX_ATTRIBUTE_VALUE_LEN
                {
                    return Err("attribute exceeds size limits".into());
                }
                attrs.push((attr.key.clone(), attr.value.clone()));
            }
            events.push(sprax_types::ContractEvent {
                contract: contract_addr,
                event_type: ev.ty.clone(),
                attributes: attrs,
            });
        }
        Ok(events)
    }

    #[allow(clippy::too_many_arguments, clippy::arc_with_non_send_sync)]
    fn process_response(
        &self,
        instance: &mut ChainInstance,
        store: &DynStore,
        contract_addr: Address,
        context: &ContractContext,
        response: Response<Empty>,
        mut remaining_gas: u64,
        depth: u32,
        total_messages: &mut u32,
    ) -> Result<(Vec<sprax_types::ContractEvent>, Vec<u8>, u64), String> {
        let initial_gas = remaining_gas;
        let mut collected_events = Self::validate_and_convert_events(contract_addr, &response)?;
        let mut return_data = response.data.map_or_else(Vec::new, |d| d.to_vec());

        for submsg in response.messages {
            *total_messages = total_messages
                .checked_add(1)
                .ok_or("submessage count overflow")?;
            if *total_messages > MAX_TOTAL_SUBMESSAGES {
                return Err("maximum submessage limit exceeded".into());
            }
            if depth >= MAX_CALL_DEPTH {
                return Err("maximum call depth exceeded".into());
            }

            remaining_gas = remaining_gas
                .checked_sub(SUBMSG_BASE_GAS_COST)
                .ok_or("out of gas dispatching submessage")?;
            let child_gas_limit = submsg
                .gas_limit
                .map(|limit| limit.min(remaining_gas))
                .unwrap_or(remaining_gas);

            let child_overlay = OverlayStore::new(store.clone());
            let child_dyn: DynStore = Arc::new(child_overlay.clone());
            let child_outcome = self.dispatch_submessage(
                &child_dyn,
                contract_addr,
                context,
                submsg.msg,
                child_gas_limit,
                depth + 1,
                total_messages,
            );

            let child_gas_consumed = match &child_outcome {
                Ok((_, _, gas_used)) => *gas_used,
                Err((_, gas_used)) => *gas_used,
            };
            remaining_gas = remaining_gas
                .checked_sub(child_gas_consumed)
                .ok_or("out of gas in child execution")?;

            match child_outcome {
                Ok((child_events, child_data, _)) => {
                    child_overlay.commit_state().map_err(|e| e.to_string())?;

                    if submsg.reply_on == ReplyOn::Success || submsg.reply_on == ReplyOn::Always {
                        #[allow(deprecated)]
                        let reply = Reply {
                            id: submsg.id,
                            payload: submsg.payload,
                            gas_used: child_gas_consumed,
                            result: SubMsgResult::Ok(SubMsgResponse {
                                events: vec![],
                                data: child_data.map(Binary::new),
                                msg_responses: vec![],
                            }),
                        };
                        let env = Self::env(
                            &context.chain_id,
                            context.height,
                            context.timestamp,
                            contract_addr,
                        )?;
                        let reply_res: Response<Empty> = call_reply(instance, &env, &reply)
                            .map_err(|e| format!("reply call failed: {e}"))?
                            .into_result()
                            .map_err(|e| format!("reply execution reverted: {e}"))?;

                        let report = instance.create_gas_report();
                        let reply_gas =
                            (report.limit - report.remaining).div_ceil(VM_GAS_PER_CHAIN_GAS);
                        remaining_gas = remaining_gas
                            .checked_sub(reply_gas)
                            .ok_or("out of gas executing reply")?;

                        let (r_events, r_data, sub_gas) = self.process_response(
                            instance,
                            store,
                            contract_addr,
                            context,
                            reply_res,
                            remaining_gas,
                            depth + 1,
                            total_messages,
                        )?;
                        remaining_gas = remaining_gas
                            .checked_sub(sub_gas)
                            .ok_or("out of gas in nested reply submessages")?;

                        collected_events.extend(child_events);
                        collected_events.extend(r_events);
                        if !r_data.is_empty() {
                            return_data = r_data;
                        }
                    } else {
                        collected_events.extend(child_events);
                    }
                }
                Err((err_msg, _)) => {
                    if submsg.reply_on == ReplyOn::Error || submsg.reply_on == ReplyOn::Always {
                        let reply = Reply {
                            id: submsg.id,
                            payload: submsg.payload,
                            gas_used: child_gas_consumed,
                            result: SubMsgResult::Err(err_msg),
                        };
                        let env = Self::env(
                            &context.chain_id,
                            context.height,
                            context.timestamp,
                            contract_addr,
                        )?;
                        let reply_res: Response<Empty> = call_reply(instance, &env, &reply)
                            .map_err(|e| format!("reply error call failed: {e}"))?
                            .into_result()
                            .map_err(|e| format!("reply error handler reverted: {e}"))?;

                        let report = instance.create_gas_report();
                        let reply_gas =
                            (report.limit - report.remaining).div_ceil(VM_GAS_PER_CHAIN_GAS);
                        remaining_gas = remaining_gas
                            .checked_sub(reply_gas)
                            .ok_or("out of gas executing reply on error")?;

                        let (r_events, r_data, sub_gas) = self.process_response(
                            instance,
                            store,
                            contract_addr,
                            context,
                            reply_res,
                            remaining_gas,
                            depth + 1,
                            total_messages,
                        )?;
                        remaining_gas = remaining_gas
                            .checked_sub(sub_gas)
                            .ok_or("out of gas in nested reply on error submessages")?;

                        collected_events.extend(r_events);
                        if !r_data.is_empty() {
                            return_data = r_data;
                        }
                    } else {
                        return Err(format!("submessage execution reverted: {err_msg}"));
                    }
                }
            }
        }

        let total_consumed = initial_gas - remaining_gas;
        Ok((collected_events, return_data, total_consumed))
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn dispatch_submessage(
        &self,
        store: &DynStore,
        sender_contract: Address,
        context: &ContractContext,
        msg: CosmosMsg<Empty>,
        gas_limit: u64,
        depth: u32,
        total_messages: &mut u32,
    ) -> Result<(Vec<sprax_types::ContractEvent>, Option<Vec<u8>>, u64), (String, u64)> {
        match msg {
            CosmosMsg::Bank(bank_msg) => match bank_msg {
                BankMsg::Send { to_address, amount } => {
                    let recipient = Address::from_bech32(&to_address)
                        .map_err(|e| (format!("invalid recipient address: {e}"), 1_000))?;
                    if amount.is_empty() {
                        return Err(("bank send requires at least one coin".into(), 1_000));
                    }
                    let mut total_atto: u128 = 0;
                    for coin in amount {
                        if coin.denom != "asprx" {
                            return Err((
                                format!("unsupported denomination: {}", coin.denom),
                                1_000,
                            ));
                        }
                        if coin.amount.is_zero() {
                            return Err(("cannot send zero coin".into(), 1_000));
                        }
                        let coin_atto =
                            coin_amount_to_u128(&coin.amount).map_err(|e| (e, 1_000))?;
                        total_atto = total_atto
                            .checked_add(coin_atto)
                            .ok_or_else(|| ("amount overflow".into(), 1_000))?;
                    }
                    let send_amount = Amount::from_atto(total_atto);

                    let sender_key = account_key(&sender_contract);
                    let mut sender_acc: AccountState =
                        match store.get(&sender_key).map_err(|e| (e.to_string(), 1_000))? {
                            Some(bytes) => serde_json::from_slice(&bytes)
                                .map_err(|e| (e.to_string(), 1_000))?,
                            None => AccountState::default(),
                        };
                    if sender_acc.balance < send_amount {
                        return Err(("insufficient funds in contract balance".into(), 1_000));
                    }
                    sender_acc.balance = sender_acc
                        .balance
                        .checked_sub(send_amount)
                        .map_err(|e| (e.to_string(), 1_000))?;
                    store
                        .set(
                            &sender_key,
                            &serde_json::to_vec(&sender_acc).map_err(|e| (e.to_string(), 1_000))?,
                        )
                        .map_err(|e| (e.to_string(), 1_000))?;

                    let recipient_key = account_key(&recipient);
                    let mut recipient_acc: AccountState = match store
                        .get(&recipient_key)
                        .map_err(|e| (e.to_string(), 1_000))?
                    {
                        Some(bytes) => {
                            serde_json::from_slice(&bytes).map_err(|e| (e.to_string(), 1_000))?
                        }
                        None => AccountState::default(),
                    };
                    recipient_acc.balance = recipient_acc
                        .balance
                        .checked_add(send_amount)
                        .map_err(|e| (e.to_string(), 1_000))?;
                    store
                        .set(
                            &recipient_key,
                            &serde_json::to_vec(&recipient_acc)
                                .map_err(|e| (e.to_string(), 1_000))?,
                        )
                        .map_err(|e| (e.to_string(), 1_000))?;

                    Ok((vec![], None, 1_000))
                }
                BankMsg::Burn { amount } => {
                    let mut total_atto: u128 = 0;
                    for coin in amount {
                        if coin.denom != "asprx" {
                            return Err((
                                format!("unsupported denomination: {}", coin.denom),
                                1_000,
                            ));
                        }
                        let coin_atto =
                            coin_amount_to_u128(&coin.amount).map_err(|e| (e, 1_000))?;
                        total_atto = total_atto
                            .checked_add(coin_atto)
                            .ok_or_else(|| ("amount overflow".into(), 1_000))?;
                    }
                    let burn_amount = Amount::from_atto(total_atto);
                    let sender_key = account_key(&sender_contract);
                    let mut sender_acc: AccountState =
                        match store.get(&sender_key).map_err(|e| (e.to_string(), 1_000))? {
                            Some(bytes) => serde_json::from_slice(&bytes)
                                .map_err(|e| (e.to_string(), 1_000))?,
                            None => AccountState::default(),
                        };
                    if sender_acc.balance < burn_amount {
                        return Err(("insufficient funds to burn".into(), 1_000));
                    }
                    sender_acc.balance = sender_acc
                        .balance
                        .checked_sub(burn_amount)
                        .map_err(|e| (e.to_string(), 1_000))?;
                    store
                        .set(
                            &sender_key,
                            &serde_json::to_vec(&sender_acc).map_err(|e| (e.to_string(), 1_000))?,
                        )
                        .map_err(|e| (e.to_string(), 1_000))?;

                    let supply_k = supply_key();
                    let mut supply: SupplyState =
                        match store.get(&supply_k).map_err(|e| (e.to_string(), 1_000))? {
                            Some(bytes) => serde_json::from_slice(&bytes)
                                .map_err(|e| (e.to_string(), 1_000))?,
                            None => SupplyState::default(),
                        };
                    supply.circulating_supply = supply
                        .circulating_supply
                        .checked_sub(burn_amount)
                        .map_err(|e| (e.to_string(), 1_000))?;
                    supply.total_burned = supply
                        .total_burned
                        .checked_add(burn_amount)
                        .map_err(|e| (e.to_string(), 1_000))?;
                    store
                        .set(
                            &supply_k,
                            &serde_json::to_vec(&supply).map_err(|e| (e.to_string(), 1_000))?,
                        )
                        .map_err(|e| (e.to_string(), 1_000))?;

                    Ok((vec![], None, 1_000))
                }
                _ => Err(("unsupported BankMsg variant".into(), 1_000)),
            },
            CosmosMsg::Wasm(wasm_msg) => match wasm_msg {
                WasmMsg::Execute {
                    contract_addr,
                    msg,
                    funds,
                } => {
                    let target = Address::from_bech32(&contract_addr)
                        .map_err(|e| (format!("invalid contract address: {e}"), 1_000))?;
                    let mut send_atto: u128 = 0;
                    for coin in funds {
                        if coin.denom != "asprx" {
                            return Err((
                                format!("unsupported denomination: {}", coin.denom),
                                1_000,
                            ));
                        }
                        let coin_atto =
                            coin_amount_to_u128(&coin.amount).map_err(|e| (e, 1_000))?;
                        send_atto = send_atto
                            .checked_add(coin_atto)
                            .ok_or_else(|| ("funds overflow".into(), 1_000))?;
                    }
                    let attached_funds = Amount::from_atto(send_atto);
                    if !attached_funds.is_zero() {
                        let sender_key = account_key(&sender_contract);
                        let mut sender_acc: AccountState =
                            match store.get(&sender_key).map_err(|e| (e.to_string(), 1_000))? {
                                Some(bytes) => serde_json::from_slice(&bytes)
                                    .map_err(|e| (e.to_string(), 1_000))?,
                                None => AccountState::default(),
                            };
                        if sender_acc.balance < attached_funds {
                            return Err(("insufficient funds attached to execute".into(), 1_000));
                        }
                        sender_acc.balance = sender_acc
                            .balance
                            .checked_sub(attached_funds)
                            .map_err(|e| (e.to_string(), 1_000))?;
                        store
                            .set(
                                &sender_key,
                                &serde_json::to_vec(&sender_acc)
                                    .map_err(|e| (e.to_string(), 1_000))?,
                            )
                            .map_err(|e| (e.to_string(), 1_000))?;

                        let target_key = account_key(&target);
                        let mut target_acc: AccountState =
                            match store.get(&target_key).map_err(|e| (e.to_string(), 1_000))? {
                                Some(bytes) => serde_json::from_slice(&bytes)
                                    .map_err(|e| (e.to_string(), 1_000))?,
                                None => AccountState::default(),
                            };
                        target_acc.balance = target_acc
                            .balance
                            .checked_add(attached_funds)
                            .map_err(|e| (e.to_string(), 1_000))?;
                        store
                            .set(
                                &target_key,
                                &serde_json::to_vec(&target_acc)
                                    .map_err(|e| (e.to_string(), 1_000))?,
                            )
                            .map_err(|e| (e.to_string(), 1_000))?;
                    }

                    let mut child_context = context.clone();
                    child_context.sender = sender_contract;
                    child_context.message_index = child_context.message_index.saturating_add(1);

                    let res = self.execute_internal(
                        store,
                        target,
                        &child_context,
                        msg.as_slice(),
                        attached_funds,
                        gas_limit,
                        depth,
                        total_messages,
                    );
                    match res {
                        Ok(r) => Ok((r.events, Some(r.data), r.gas_used)),
                        Err((e, gas)) => Err((e, gas)),
                    }
                }
                WasmMsg::Instantiate {
                    code_id,
                    msg,
                    funds,
                    label,
                    admin,
                } => {
                    let code_hash = Self::get_hash_by_code_seq(store.as_ref(), code_id)
                        .map_err(|e| (e, 1_000))?
                        .ok_or_else(|| (format!("code ID {code_id} not found"), 1_000))?;
                    let admin_addr = admin
                        .map(|a| Address::from_bech32(&a))
                        .transpose()
                        .map_err(|e| (format!("invalid admin address: {e}"), 1_000))?;

                    let mut send_atto: u128 = 0;
                    for coin in funds {
                        if coin.denom != "asprx" {
                            return Err((
                                format!("unsupported denomination: {}", coin.denom),
                                1_000,
                            ));
                        }
                        let coin_atto =
                            coin_amount_to_u128(&coin.amount).map_err(|e| (e, 1_000))?;
                        send_atto = send_atto
                            .checked_add(coin_atto)
                            .ok_or_else(|| ("funds overflow".into(), 1_000))?;
                    }
                    let attached_funds = Amount::from_atto(send_atto);

                    let mut child_context = context.clone();
                    child_context.sender = sender_contract;
                    child_context.message_index = child_context.message_index.saturating_add(1);

                    let (new_addr, res) = self.instantiate_internal(
                        store,
                        code_hash,
                        &child_context,
                        msg.as_slice(),
                        &label,
                        admin_addr,
                        attached_funds,
                        gas_limit,
                        depth,
                        total_messages,
                    )?;

                    if !attached_funds.is_zero() {
                        let sender_key = account_key(&sender_contract);
                        let mut sender_acc: AccountState =
                            match store.get(&sender_key).map_err(|e| (e.to_string(), 1_000))? {
                                Some(bytes) => serde_json::from_slice(&bytes)
                                    .map_err(|e| (e.to_string(), 1_000))?,
                                None => AccountState::default(),
                            };
                        if sender_acc.balance < attached_funds {
                            return Err((
                                "insufficient funds attached to instantiate".into(),
                                1_000,
                            ));
                        }
                        sender_acc.balance = sender_acc
                            .balance
                            .checked_sub(attached_funds)
                            .map_err(|e| (e.to_string(), 1_000))?;
                        store
                            .set(
                                &sender_key,
                                &serde_json::to_vec(&sender_acc)
                                    .map_err(|e| (e.to_string(), 1_000))?,
                            )
                            .map_err(|e| (e.to_string(), 1_000))?;

                        let new_addr_key = account_key(&new_addr);
                        let mut new_acc: AccountState = match store
                            .get(&new_addr_key)
                            .map_err(|e| (e.to_string(), 1_000))?
                        {
                            Some(bytes) => serde_json::from_slice(&bytes)
                                .map_err(|e| (e.to_string(), 1_000))?,
                            None => AccountState::default(),
                        };
                        new_acc.balance = new_acc
                            .balance
                            .checked_add(attached_funds)
                            .map_err(|e| (e.to_string(), 1_000))?;
                        store
                            .set(
                                &new_addr_key,
                                &serde_json::to_vec(&new_acc)
                                    .map_err(|e| (e.to_string(), 1_000))?,
                            )
                            .map_err(|e| (e.to_string(), 1_000))?;
                    }

                    Ok((res.events, Some(res.data), res.gas_used))
                }
                WasmMsg::Migrate {
                    contract_addr,
                    new_code_id,
                    msg,
                } => {
                    let target = Address::from_bech32(&contract_addr)
                        .map_err(|e| (format!("invalid contract address: {e}"), 1_000))?;
                    let new_code_hash = Self::get_hash_by_code_seq(store.as_ref(), new_code_id)
                        .map_err(|e| (e, 1_000))?
                        .ok_or_else(|| (format!("code ID {new_code_id} not found"), 1_000))?;

                    let mut child_context = context.clone();
                    child_context.sender = sender_contract;
                    child_context.message_index = child_context.message_index.saturating_add(1);

                    let res = self.migrate_internal(
                        store,
                        target,
                        new_code_hash,
                        &child_context,
                        msg.as_slice(),
                        gas_limit,
                        depth,
                        total_messages,
                    );
                    match res {
                        Ok(r) => Ok((r.events, Some(r.data), r.gas_used)),
                        Err((e, gas)) => Err((e, gas)),
                    }
                }
                WasmMsg::UpdateAdmin {
                    contract_addr,
                    admin,
                } => {
                    let target = Address::from_bech32(&contract_addr)
                        .map_err(|e| (format!("invalid contract address: {e}"), 1_000))?;
                    let new_admin = Address::from_bech32(&admin)
                        .map_err(|e| (format!("invalid admin address: {e}"), 1_000))?;
                    let mut metadata = Self::contract_info(store.as_ref(), target)
                        .map_err(|e| (format!("contract info lookup failed: {e}"), 1_000))?;
                    if metadata.admin != Some(sender_contract) {
                        return Err(("unauthorized: caller is not contract admin".into(), 1_000));
                    }
                    metadata.admin = Some(new_admin);
                    let meta_key = Self::key(b"wasm/contract/", target.as_bytes());
                    store
                        .set(
                            &meta_key,
                            &serde_json::to_vec(&metadata).map_err(|e| (e.to_string(), 1_000))?,
                        )
                        .map_err(|e| (e.to_string(), 1_000))?;
                    Ok((vec![], None, 1_000))
                }
                WasmMsg::ClearAdmin { contract_addr } => {
                    let target = Address::from_bech32(&contract_addr)
                        .map_err(|e| (format!("invalid contract address: {e}"), 1_000))?;
                    let mut metadata = Self::contract_info(store.as_ref(), target)
                        .map_err(|e| (format!("contract info lookup failed: {e}"), 1_000))?;
                    if metadata.admin != Some(sender_contract) {
                        return Err(("unauthorized: caller is not contract admin".into(), 1_000));
                    }
                    metadata.admin = None;
                    let meta_key = Self::key(b"wasm/contract/", target.as_bytes());
                    store
                        .set(
                            &meta_key,
                            &serde_json::to_vec(&metadata).map_err(|e| (e.to_string(), 1_000))?,
                        )
                        .map_err(|e| (e.to_string(), 1_000))?;
                    Ok((vec![], None, 1_000))
                }
                _ => Err(("unsupported WasmMsg variant".into(), 1_000)),
            },
            _ => Err(("unsupported CosmosMsg variant".into(), 1_000)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn instantiate<S: KVStore + Clone + 'static>(
        &self,
        store: &S,
        code_id: Hash32,
        context: &ContractContext,
        msg: &[u8],
        label: &str,
        admin: Option<Address>,
        funds: Amount,
        gas: u64,
    ) -> Result<(Address, RuntimeResult), String> {
        let dyn_store: DynStore = Arc::new(store.clone());
        let mut total_messages = 0;
        self.instantiate_internal(
            &dyn_store,
            code_id,
            context,
            msg,
            label,
            admin,
            funds,
            gas,
            0,
            &mut total_messages,
        )
        .map_err(|(e, _)| e)
    }

    #[allow(clippy::too_many_arguments)]
    fn instantiate_internal(
        &self,
        store: &DynStore,
        code_id: Hash32,
        context: &ContractContext,
        msg: &[u8],
        label: &str,
        admin: Option<Address>,
        funds: Amount,
        gas: u64,
        depth: u32,
        total_messages: &mut u32,
    ) -> Result<(Address, RuntimeResult), (String, u64)> {
        if label.is_empty() || label.len() > 128 {
            return Err(("contract label must contain 1 to 128 bytes".into(), 1_000));
        }
        let address = Self::contract_address(context, code_id);
        let metadata_key = Self::key(b"wasm/contract/", address.as_bytes());
        if store
            .has(&metadata_key)
            .map_err(|e| (e.to_string(), 1_000))?
        {
            return Err(("contract address already exists".into(), 1_000));
        }
        let code = Self::code(store.as_ref(), code_id).map_err(|e| (e, 1_000))?;
        let code_id_seq = Self::get_code_seq_by_hash(store.as_ref(), &code_id)
            .map_err(|e| (e, 1_000))?
            .unwrap_or(1);

        let metadata = RuntimeContractInfo {
            code_id,
            code_id_seq,
            creator: context.sender,
            admin,
            label: label.into(),
        };
        let bytes = serde_json::to_vec(&metadata).map_err(|e| (e.to_string(), 1_000))?;
        store
            .set(&metadata_key, &bytes)
            .map_err(|e| (e.to_string(), 1_000))?;

        let mut instance = Self::instance(
            store,
            &code,
            address,
            &context.chain_id,
            context.height,
            context.timestamp,
            gas,
            false,
            depth,
        )
        .map_err(|e| (e, 1_000))?;

        let env = Self::env(
            &context.chain_id,
            context.height,
            context.timestamp,
            address,
        )
        .map_err(|e| (e, 1_000))?;
        let info = Self::info(context.sender, funds).map_err(|e| (e, 1_000))?;

        let response: Response<Empty> = call_instantiate(&mut instance, &env, &info, msg)
            .map_err(|e| (format!("instantiate VM error: {e}"), 1_000))?
            .into_result()
            .map_err(|e| (format!("instantiate reverted: {e}"), 1_000))?;

        let report = instance.create_gas_report();
        let gas_used = (report.limit - report.remaining).div_ceil(VM_GAS_PER_CHAIN_GAS);
        let remaining_gas = gas
            .checked_sub(gas_used)
            .ok_or_else(|| ("out of gas in contract instantiation".into(), gas_used))?;

        let (collected_events, return_data, submsg_gas) = self
            .process_response(
                &mut instance,
                store,
                address,
                context,
                response,
                remaining_gas,
                depth,
                total_messages,
            )
            .map_err(|e| (e, gas_used))?;

        Ok((
            address,
            RuntimeResult {
                data: return_data,
                gas_used: gas_used + submsg_gas,
                events: collected_events,
            },
        ))
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
        let dyn_store: DynStore = Arc::new(store.clone());
        let mut total_messages = 0;
        self.execute_internal(
            &dyn_store,
            address,
            context,
            msg,
            funds,
            gas,
            0,
            &mut total_messages,
        )
        .map_err(|(e, _)| e)
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_internal(
        &self,
        store: &DynStore,
        address: Address,
        context: &ContractContext,
        msg: &[u8],
        funds: Amount,
        gas: u64,
        depth: u32,
        total_messages: &mut u32,
    ) -> Result<RuntimeResult, (String, u64)> {
        let metadata = Self::contract_info(store.as_ref(), address).map_err(|e| (e, 1_000))?;
        let code = Self::code(store.as_ref(), metadata.code_id).map_err(|e| (e, 1_000))?;
        let mut instance = Self::instance(
            store,
            &code,
            address,
            &context.chain_id,
            context.height,
            context.timestamp,
            gas,
            false,
            depth,
        )
        .map_err(|e| (e, 1_000))?;

        let env = Self::env(
            &context.chain_id,
            context.height,
            context.timestamp,
            address,
        )
        .map_err(|e| (e, 1_000))?;
        let info = Self::info(context.sender, funds).map_err(|e| (e, 1_000))?;

        let response: Response<Empty> = call_execute(&mut instance, &env, &info, msg)
            .map_err(|e| (format!("execute VM error: {e}"), 1_000))?
            .into_result()
            .map_err(|e| (format!("execute reverted: {e}"), 1_000))?;

        let report = instance.create_gas_report();
        let gas_used = (report.limit - report.remaining).div_ceil(VM_GAS_PER_CHAIN_GAS);
        let remaining_gas = gas
            .checked_sub(gas_used)
            .ok_or_else(|| ("out of gas in contract execution".into(), gas_used))?;

        let (collected_events, return_data, submsg_gas) = self
            .process_response(
                &mut instance,
                store,
                address,
                context,
                response,
                remaining_gas,
                depth,
                total_messages,
            )
            .map_err(|e| (e, gas_used))?;

        Ok(RuntimeResult {
            data: return_data,
            gas_used: gas_used + submsg_gas,
            events: collected_events,
        })
    }

    pub fn migrate<S: KVStore + Clone + 'static>(
        &self,
        store: &S,
        address: Address,
        new_code_id: Hash32,
        context: &ContractContext,
        msg: &[u8],
        gas: u64,
    ) -> Result<RuntimeResult, String> {
        let dyn_store: DynStore = Arc::new(store.clone());
        let mut total_messages = 0;
        self.migrate_internal(
            &dyn_store,
            address,
            new_code_id,
            context,
            msg,
            gas,
            0,
            &mut total_messages,
        )
        .map_err(|(e, _)| e)
    }

    #[allow(clippy::too_many_arguments)]
    fn migrate_internal(
        &self,
        store: &DynStore,
        address: Address,
        new_code_id: Hash32,
        context: &ContractContext,
        msg: &[u8],
        gas: u64,
        depth: u32,
        total_messages: &mut u32,
    ) -> Result<RuntimeResult, (String, u64)> {
        let mut metadata = Self::contract_info(store.as_ref(), address).map_err(|e| (e, 1_000))?;
        match metadata.admin {
            Some(admin) => {
                if admin != context.sender {
                    return Err(("unauthorized: caller is not contract admin".into(), 1_000));
                }
            }
            None => {
                return Err((
                    "cannot migrate contract: contract has no admin".into(),
                    1_000,
                ));
            }
        }

        let new_code = Self::code(store.as_ref(), new_code_id).map_err(|e| (e, 1_000))?;
        let mut instance = Self::instance(
            store,
            &new_code,
            address,
            &context.chain_id,
            context.height,
            context.timestamp,
            gas,
            false,
            depth,
        )
        .map_err(|e| (e, 1_000))?;

        let env = Self::env(
            &context.chain_id,
            context.height,
            context.timestamp,
            address,
        )
        .map_err(|e| (e, 1_000))?;

        let response: Response<Empty> = call_migrate(&mut instance, &env, msg)
            .map_err(|e| (format!("migrate VM error: {e}"), 1_000))?
            .into_result()
            .map_err(|e| (format!("migrate reverted: {e}"), 1_000))?;

        let report = instance.create_gas_report();
        let gas_used = (report.limit - report.remaining).div_ceil(VM_GAS_PER_CHAIN_GAS);
        let remaining_gas = gas
            .checked_sub(gas_used)
            .ok_or_else(|| ("out of gas in contract migration".into(), gas_used))?;

        let (collected_events, return_data, submsg_gas) = self
            .process_response(
                &mut instance,
                store,
                address,
                context,
                response,
                remaining_gas,
                depth,
                total_messages,
            )
            .map_err(|e| (e, gas_used))?;

        metadata.code_id = new_code_id;
        metadata.code_id_seq = Self::get_code_seq_by_hash(store.as_ref(), &new_code_id)
            .map_err(|e| (e, 1_000))?
            .unwrap_or(1);
        let meta_key = Self::key(b"wasm/contract/", address.as_bytes());
        store
            .set(
                &meta_key,
                &serde_json::to_vec(&metadata).map_err(|e| (e.to_string(), 1_000))?,
            )
            .map_err(|e| (e.to_string(), 1_000))?;

        Ok(RuntimeResult {
            data: return_data,
            gas_used: gas_used + submsg_gas,
            events: collected_events,
        })
    }

    pub fn update_admin<S: KVStore + Clone + 'static>(
        &self,
        store: &S,
        address: Address,
        context: &ContractContext,
        new_admin: Option<Address>,
    ) -> Result<(), String> {
        let mut metadata = Self::contract_info(store, address)?;
        match metadata.admin {
            Some(admin) => {
                if admin != context.sender {
                    return Err("unauthorized: caller is not contract admin".into());
                }
            }
            None => {
                return Err("cannot update admin: contract has no admin".into());
            }
        }
        metadata.admin = new_admin;
        let meta_key = Self::key(b"wasm/contract/", address.as_bytes());
        store
            .set(
                &meta_key,
                &serde_json::to_vec(&metadata).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn query<S: KVStore + Clone + 'static>(
        &self,
        store: &S,
        address: Address,
        context: &ContractContext,
        msg: &[u8],
        gas: u64,
    ) -> Result<RuntimeResult, String> {
        let dyn_store: DynStore = Arc::new(store.clone());
        Self::query_internal(
            &dyn_store,
            address,
            &context.chain_id,
            context.height,
            context.timestamp,
            msg,
            gas,
            0,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn query_internal(
        store: &DynStore,
        address: Address,
        chain_id: &str,
        height: u64,
        timestamp: u64,
        msg: &[u8],
        gas: u64,
        depth: u32,
    ) -> Result<RuntimeResult, String> {
        let metadata = Self::contract_info(store.as_ref(), address)?;
        let code = Self::code(store.as_ref(), metadata.code_id)?;
        let mut instance = Self::instance(
            store, &code, address, chain_id, height, timestamp, gas, true, depth,
        )?;
        let env = Self::env(chain_id, height, timestamp, address)?;
        let result = call_query(&mut instance, &env, msg)
            .map_err(|e| e.to_string())?
            .into_result()
            .map_err(|e| e.to_string())?;
        let report = instance.create_gas_report();
        Ok(RuntimeResult {
            data: result.to_vec(),
            gas_used: (report.limit - report.remaining).div_ceil(VM_GAS_PER_CHAIN_GAS),
            events: vec![],
        })
    }
}
