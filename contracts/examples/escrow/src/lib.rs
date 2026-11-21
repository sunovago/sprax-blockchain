use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InstantiateMsg {
    pub arbiter: String,
    pub recipient: String,
    pub timeout_height: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Deposit { amount: u128 },
    Release {},
    Refund {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetEscrow {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum EscrowStatus {
    Locked,
    Released,
    Refunded,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EscrowState {
    pub sender: String,
    pub recipient: String,
    pub arbiter: String,
    pub amount: u128,
    pub timeout_height: u64,
    pub status: EscrowStatus,
}

const KEY_ESCROW: &[u8] = b"escrow_data";

fn load_state(deps: Deps) -> StdResult<EscrowState> {
    let raw = deps
        .storage
        .get(KEY_ESCROW)
        .ok_or_else(|| StdError::msg("escrow state not found"))?;
    from_json(&raw)
}

fn save_state(deps: &mut DepsMut, state: &EscrowState) -> StdResult<()> {
    let raw = to_json_vec(state)?;
    deps.storage.set(KEY_ESCROW, &raw);
    Ok(())
}

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    if msg.arbiter.is_empty() || msg.recipient.is_empty() {
        return Err(StdError::msg("arbiter and recipient cannot be empty"));
    }

    let state = EscrowState {
        sender: info.sender.to_string(),
        recipient: msg.recipient,
        arbiter: msg.arbiter,
        amount: 0,
        timeout_height: msg.timeout_height,
        status: EscrowStatus::Locked,
    };

    save_state(&mut deps, &state)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate_escrow")
        .add_attribute("sender", state.sender)
        .add_attribute("arbiter", state.arbiter)
        .add_attribute("recipient", state.recipient))
}

#[entry_point]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let mut state = load_state(deps.as_ref())?;

    match msg {
        ExecuteMsg::Deposit { amount } => {
            if state.status != EscrowStatus::Locked {
                return Err(StdError::msg("escrow is already settled"));
            }
            if info.sender.as_str() != state.sender {
                return Err(StdError::msg(
                    "only original sender can deposit into escrow",
                ));
            }
            if amount == 0 {
                return Err(StdError::msg("deposit amount must be greater than zero"));
            }

            state.amount = state
                .amount
                .checked_add(amount)
                .ok_or_else(|| StdError::msg("escrow deposit amount overflow"))?;
            save_state(&mut deps, &state)?;

            Ok(Response::new()
                .add_attribute("action", "deposit")
                .add_attribute("amount", amount.to_string())
                .add_attribute("total_amount", state.amount.to_string()))
        }

        ExecuteMsg::Release {} => {
            if state.status != EscrowStatus::Locked {
                return Err(StdError::msg("escrow is not in locked state"));
            }
            // Only arbiter or sender can release to recipient
            let caller = info.sender.as_str();
            if caller != state.arbiter && caller != state.sender {
                return Err(StdError::msg("only arbiter or sender can release funds"));
            }

            state.status = EscrowStatus::Released;
            save_state(&mut deps, &state)?;

            Ok(Response::new()
                .add_attribute("action", "release")
                .add_attribute("released_by", caller)
                .add_attribute("recipient", &state.recipient)
                .add_attribute("amount", state.amount.to_string()))
        }

        ExecuteMsg::Refund {} => {
            if state.status != EscrowStatus::Locked {
                return Err(StdError::msg("escrow is not in locked state"));
            }
            let caller = info.sender.as_str();
            let is_arbiter = caller == state.arbiter;
            let is_sender = caller == state.sender;

            if !is_arbiter && !is_sender {
                return Err(StdError::msg("unauthorized caller for refund"));
            }

            // Sender can only refund if timeout height is reached
            if is_sender && !is_arbiter && env.block.height < state.timeout_height {
                return Err(StdError::msg(format!(
                    "cannot refund before timeout height: current={}, timeout={}",
                    env.block.height, state.timeout_height
                )));
            }

            state.status = EscrowStatus::Refunded;
            save_state(&mut deps, &state)?;

            Ok(Response::new()
                .add_attribute("action", "refund")
                .add_attribute("refunded_by", caller)
                .add_attribute("sender", &state.sender)
                .add_attribute("amount", state.amount.to_string()))
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetEscrow {} => {
            let state = load_state(deps)?;
            to_json_binary(&state)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::Addr;

    #[test]
    fn test_escrow_deposit_and_release() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let sender = Addr::unchecked("sprax1buyer");
        let recipient = Addr::unchecked("sprax1seller");
        let arbiter = Addr::unchecked("sprax1judge");

        // 1. Instantiate
        let inst_msg = InstantiateMsg {
            arbiter: arbiter.to_string(),
            recipient: recipient.to_string(),
            timeout_height: 500,
        };
        let info = message_info(&sender, &[]);
        instantiate(deps.as_mut(), env.clone(), info, inst_msg).unwrap();

        // 2. Deposit
        let deposit_msg = ExecuteMsg::Deposit { amount: 50_000 };
        let info = message_info(&sender, &[]);
        execute(deps.as_mut(), env.clone(), info, deposit_msg).unwrap();

        let state = load_state(deps.as_ref()).unwrap();
        assert_eq!(state.amount, 50_000);
        assert_eq!(state.status, EscrowStatus::Locked);

        // 3. Unauthorized caller tries to release -> fails
        let intruder = Addr::unchecked("sprax1intruder");
        let release_msg = ExecuteMsg::Release {};
        let info = message_info(&intruder, &[]);
        let err = execute(deps.as_mut(), env.clone(), info, release_msg.clone()).unwrap_err();
        assert!(err.to_string().contains("only arbiter or sender"));

        // 4. Arbiter releases
        let info = message_info(&arbiter, &[]);
        execute(deps.as_mut(), env.clone(), info, release_msg).unwrap();

        let state = load_state(deps.as_ref()).unwrap();
        assert_eq!(state.status, EscrowStatus::Released);
    }

    #[test]
    fn test_escrow_refund_timeout() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.height = 100;

        let sender = Addr::unchecked("sprax1buyer");
        let recipient = Addr::unchecked("sprax1seller");
        let arbiter = Addr::unchecked("sprax1judge");

        let inst_msg = InstantiateMsg {
            arbiter: arbiter.to_string(),
            recipient: recipient.to_string(),
            timeout_height: 500,
        };
        let info = message_info(&sender, &[]);
        instantiate(deps.as_mut(), env.clone(), info, inst_msg).unwrap();

        // Deposit
        let deposit_msg = ExecuteMsg::Deposit { amount: 10_000 };
        let info = message_info(&sender, &[]);
        execute(deps.as_mut(), env.clone(), info, deposit_msg).unwrap();

        // Sender tries refund BEFORE timeout (height 100 < 500) -> fails
        let refund_msg = ExecuteMsg::Refund {};
        let info = message_info(&sender, &[]);
        let err = execute(deps.as_mut(), env.clone(), info, refund_msg.clone()).unwrap_err();
        assert!(err
            .to_string()
            .contains("cannot refund before timeout height"));

        // Advance block height past timeout (height 550 >= 500)
        env.block.height = 550;
        let info = message_info(&sender, &[]);
        execute(deps.as_mut(), env.clone(), info, refund_msg).unwrap();

        let state = load_state(deps.as_ref()).unwrap();
        assert_eq!(state.status, EscrowStatus::Refunded);
    }
}
