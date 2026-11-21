use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, Event, MessageInfo,
    Reply, Response, StdError, StdResult, SubMsg, SubMsgResult, Uint256, WasmMsg,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InstantiateMsg {
    pub count: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Increment {},
    WriteThenFail {},
    BurnGas {},
    EmitCustomEvent {
        key: String,
        value: String,
    },
    SendBank {
        to: String,
        amount: u128,
        denom: String,
    },
    CallOther {
        contract: String,
        msg: Binary,
        funds: Vec<Coin>,
        reply_on: Option<String>,
        reply_id: Option<u64>,
    },
    QueryBalance {
        address: String,
        denom: String,
    },
    QueryOtherCount {
        contract: String,
    },
    QueryOtherRaw {
        contract: String,
        key: Binary,
    },
    QueryContractInfo {
        contract: String,
    },
    QueryCodeInfo {
        code_id: u64,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Count {},
    LastReply {},
    QueriedBalance {},
    QueriedOtherCount {},
    QueriedRaw {},
    QueriedAdmin {},
    QueriedCodeHash {},
}

#[derive(Serialize, Deserialize)]
pub struct MigrateMsg {
    pub new_count: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CountResponse {
    pub count: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LastReplyResponse {
    pub id: u64,
    pub success: bool,
    pub data: Option<Binary>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct QueriedBalanceResponse {
    pub amount: u128,
    pub denom: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct QueriedOtherCountResponse {
    pub count: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct QueriedRawResponse {
    pub data: Option<Binary>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct QueriedAdminResponse {
    pub admin: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct QueriedCodeHashResponse {
    pub checksum: Binary,
}

fn count(deps: Deps) -> StdResult<u64> {
    let bytes = deps
        .storage
        .get(b"count")
        .ok_or_else(|| StdError::msg("counter missing"))?;
    let array: [u8; 8] = bytes
        .try_into()
        .map_err(|_| StdError::msg("counter corrupt"))?;
    Ok(u64::from_be_bytes(array))
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.storage.set(b"count", &msg.count.to_be_bytes());
    deps.storage.set(b"owner", info.sender.as_bytes());
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Increment {} => {
            let next = count(deps.as_ref())?
                .checked_add(1)
                .ok_or_else(|| StdError::msg("counter overflow"))?;
            deps.storage.set(b"count", &next.to_be_bytes());
            Ok(Response::new()
                .add_attribute("action", "increment")
                .add_attribute("count", next.to_string())
                .set_data(to_json_binary(&CountResponse { count: next })?))
        }
        ExecuteMsg::WriteThenFail {} => {
            deps.storage.set(b"count", &999u64.to_be_bytes());
            Err(StdError::msg("intentional failure after a write"))
        }
        ExecuteMsg::BurnGas {} => {
            let mut value = 0u64;
            loop {
                value = value.wrapping_add(1);
                deps.storage.set(b"count", &value.to_be_bytes());
            }
        }
        ExecuteMsg::EmitCustomEvent { key, value } => {
            let event = Event::new("custom_protocol_event").add_attribute(key, value);
            Ok(Response::new()
                .add_event(event)
                .add_attribute("action", "emit_custom_event"))
        }
        ExecuteMsg::SendBank { to, amount, denom } => {
            let send_msg = BankMsg::Send {
                to_address: to,
                amount: vec![Coin {
                    denom,
                    amount: Uint256::from(amount),
                }],
            };
            Ok(Response::new()
                .add_message(send_msg)
                .add_attribute("action", "send_bank"))
        }
        ExecuteMsg::CallOther {
            contract,
            msg,
            funds,
            reply_on,
            reply_id,
        } => {
            let wasm_msg = WasmMsg::Execute {
                contract_addr: contract,
                msg,
                funds,
            };
            let response = if let Some(reply_mode) = reply_on {
                let id = reply_id.unwrap_or(1);
                let submsg = match reply_mode.as_str() {
                    "success" => SubMsg::reply_on_success(wasm_msg, id),
                    "error" => SubMsg::reply_on_error(wasm_msg, id),
                    "always" => SubMsg::reply_always(wasm_msg, id),
                    _ => SubMsg::new(wasm_msg),
                };
                Response::new().add_submessage(submsg)
            } else {
                Response::new().add_message(wasm_msg)
            };
            Ok(response.add_attribute("action", "call_other"))
        }
        ExecuteMsg::QueryBalance { address, denom } => {
            let coin = deps.querier.query_balance(address, denom)?;
            let amount: u128 = coin.amount.to_string().parse().unwrap_or(0);
            let resp = QueriedBalanceResponse {
                amount,
                denom: coin.denom,
            };
            deps.storage
                .set(b"queried_balance", &to_json_binary(&resp)?);
            Ok(Response::new().add_attribute("action", "query_balance"))
        }
        ExecuteMsg::QueryOtherCount { contract } => {
            let query_msg = QueryMsg::Count {};
            let resp: CountResponse = deps.querier.query_wasm_smart(contract, &query_msg)?;
            deps.storage
                .set(b"queried_other_count", &resp.count.to_be_bytes());
            Ok(Response::new().add_attribute("action", "query_other_count"))
        }
        ExecuteMsg::QueryOtherRaw { contract, key } => {
            let raw_data = deps.querier.query_wasm_raw(contract, key)?;
            let resp = QueriedRawResponse {
                data: raw_data.map(Binary::new),
            };
            deps.storage.set(b"queried_raw", &to_json_binary(&resp)?);
            Ok(Response::new().add_attribute("action", "query_other_raw"))
        }
        ExecuteMsg::QueryContractInfo { contract } => {
            let info = deps.querier.query_wasm_contract_info(contract)?;
            let resp = QueriedAdminResponse {
                admin: info.admin.map(|a| a.into_string()),
            };
            deps.storage.set(b"queried_admin", &to_json_binary(&resp)?);
            Ok(Response::new().add_attribute("action", "query_contract_info"))
        }
        ExecuteMsg::QueryCodeInfo { code_id } => {
            let info = deps.querier.query_wasm_code_info(code_id)?;
            let resp = QueriedCodeHashResponse {
                checksum: Binary::from(info.checksum.as_slice()),
            };
            deps.storage
                .set(b"queried_code_hash", &to_json_binary(&resp)?);
            Ok(Response::new().add_attribute("action", "query_code_info"))
        }
    }
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    #[allow(deprecated)]
    let (success, data) = match msg.result {
        SubMsgResult::Ok(resp) => (true, resp.data),
        SubMsgResult::Err(_) => (false, None),
    };
    let last = LastReplyResponse {
        id: msg.id,
        success,
        data,
    };
    deps.storage.set(b"last_reply", &to_json_binary(&last)?);
    Ok(Response::new()
        .add_attribute("action", "reply")
        .add_attribute("reply_id", msg.id.to_string())
        .add_attribute("reply_success", success.to_string()))
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
    deps.storage.set(b"count", &msg.new_count.to_be_bytes());
    deps.storage.set(b"migrated", b"true");
    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_attribute("new_count", msg.new_count.to_string()))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Count {} => to_json_binary(&CountResponse {
            count: count(deps)?,
        }),
        QueryMsg::LastReply {} => {
            let raw = deps
                .storage
                .get(b"last_reply")
                .ok_or_else(|| StdError::msg("no reply stored"))?;
            Ok(Binary::new(raw))
        }
        QueryMsg::QueriedBalance {} => {
            let raw = deps
                .storage
                .get(b"queried_balance")
                .ok_or_else(|| StdError::msg("no balance stored"))?;
            Ok(Binary::new(raw))
        }
        QueryMsg::QueriedOtherCount {} => {
            let raw = deps
                .storage
                .get(b"queried_other_count")
                .ok_or_else(|| StdError::msg("no other count stored"))?;
            let array: [u8; 8] = raw
                .try_into()
                .map_err(|_| StdError::msg("corrupt other count"))?;
            to_json_binary(&QueriedOtherCountResponse {
                count: u64::from_be_bytes(array),
            })
        }
        QueryMsg::QueriedRaw {} => {
            let raw = deps
                .storage
                .get(b"queried_raw")
                .ok_or_else(|| StdError::msg("no raw stored"))?;
            Ok(Binary::new(raw))
        }
        QueryMsg::QueriedAdmin {} => {
            let raw = deps
                .storage
                .get(b"queried_admin")
                .ok_or_else(|| StdError::msg("no admin stored"))?;
            Ok(Binary::new(raw))
        }
        QueryMsg::QueriedCodeHash {} => {
            let raw = deps
                .storage
                .get(b"queried_code_hash")
                .ok_or_else(|| StdError::msg("no code hash stored"))?;
            Ok(Binary::new(raw))
        }
    }
}
