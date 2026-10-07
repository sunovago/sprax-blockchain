use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
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
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Count {},
}
#[derive(Serialize, Deserialize)]
pub struct CountResponse {
    pub count: u64,
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
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    if deps.storage.get(b"owner").as_deref() != Some(info.sender.as_bytes()) {
        return Err(StdError::msg("unauthorized"));
    }
    match msg {
        ExecuteMsg::Increment {} => {
            let next = count(deps.as_ref())?
                .checked_add(1)
                .ok_or_else(|| StdError::msg("counter overflow"))?;
            deps.storage.set(b"count", &next.to_be_bytes());
            Ok(Response::new().set_data(to_json_binary(&CountResponse { count: next })?))
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
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Count {} => to_json_binary(&CountResponse {
            count: count(deps)?,
        }),
    }
}
