use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InitialBalance {
    pub address: String,
    pub amount: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<InitialBalance>,
    pub minter: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Transfer {
        recipient: String,
        amount: u128,
    },
    TransferFrom {
        owner: String,
        recipient: String,
        amount: u128,
    },
    Approve {
        spender: String,
        amount: u128,
    },
    Mint {
        recipient: String,
        amount: u128,
    },
    Burn {
        amount: u128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Balance { address: String },
    Allowance { owner: String, spender: String },
    TokenInfo {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BalanceResponse {
    pub balance: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AllowanceResponse {
    pub allowance: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TokenInfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u128,
    pub minter: Option<String>,
}

const KEY_TOKEN_INFO: &[u8] = b"token_info";

fn balance_key(address: &str) -> Vec<u8> {
    let mut key = b"bal:".to_vec();
    key.extend_from_slice(address.as_bytes());
    key
}

fn allowance_key(owner: &str, spender: &str) -> Vec<u8> {
    let mut key = b"allow:".to_vec();
    key.extend_from_slice(owner.as_bytes());
    key.push(b':');
    key.extend_from_slice(spender.as_bytes());
    key
}

fn read_balance(deps: Deps, address: &str) -> StdResult<u128> {
    let key = balance_key(address);
    match deps.storage.get(&key) {
        Some(bytes) => {
            let arr: [u8; 16] = bytes
                .try_into()
                .map_err(|_| StdError::msg("corrupted balance data"))?;
            Ok(u128::from_be_bytes(arr))
        }
        None => Ok(0),
    }
}

fn write_balance(deps: &mut DepsMut, address: &str, amount: u128) {
    let key = balance_key(address);
    if amount == 0 {
        deps.storage.remove(&key);
    } else {
        deps.storage.set(&key, &amount.to_be_bytes());
    }
}

fn read_allowance(deps: Deps, owner: &str, spender: &str) -> StdResult<u128> {
    let key = allowance_key(owner, spender);
    match deps.storage.get(&key) {
        Some(bytes) => {
            let arr: [u8; 16] = bytes
                .try_into()
                .map_err(|_| StdError::msg("corrupted allowance data"))?;
            Ok(u128::from_be_bytes(arr))
        }
        None => Ok(0),
    }
}

fn write_allowance(deps: &mut DepsMut, owner: &str, spender: &str, amount: u128) {
    let key = allowance_key(owner, spender);
    if amount == 0 {
        deps.storage.remove(&key);
    } else {
        deps.storage.set(&key, &amount.to_be_bytes());
    }
}

fn read_token_info(deps: Deps) -> StdResult<TokenInfoResponse> {
    let raw = deps
        .storage
        .get(KEY_TOKEN_INFO)
        .ok_or_else(|| StdError::msg("token info not found"))?;
    from_json(&raw)
}

fn write_token_info(deps: &mut DepsMut, info: &TokenInfoResponse) -> StdResult<()> {
    let raw = to_json_vec(info)?;
    deps.storage.set(KEY_TOKEN_INFO, &raw);
    Ok(())
}

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let mut total_supply: u128 = 0;
    for initial in msg.initial_balances {
        total_supply = total_supply
            .checked_add(initial.amount)
            .ok_or_else(|| StdError::msg("total supply overflow in initial balances"))?;
        let current = read_balance(deps.as_ref(), &initial.address)?;
        let new_bal = current
            .checked_add(initial.amount)
            .ok_or_else(|| StdError::msg("balance overflow"))?;
        write_balance(&mut deps, &initial.address, new_bal);
    }

    let token_info = TokenInfoResponse {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply,
        minter: msg.minter,
    };
    write_token_info(&mut deps, &token_info)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn execute(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            let sender = info.sender.as_str();
            let sender_bal = read_balance(deps.as_ref(), sender)?;
            let new_sender_bal = sender_bal
                .checked_sub(amount)
                .ok_or_else(|| StdError::msg("insufficient funds for transfer"))?;
            let recip_bal = read_balance(deps.as_ref(), &recipient)?;
            let new_recip_bal = recip_bal
                .checked_add(amount)
                .ok_or_else(|| StdError::msg("recipient balance overflow"))?;

            write_balance(&mut deps, sender, new_sender_bal);
            write_balance(&mut deps, &recipient, new_recip_bal);

            Ok(Response::new()
                .add_attribute("action", "transfer")
                .add_attribute("from", sender)
                .add_attribute("to", recipient)
                .add_attribute("amount", amount.to_string()))
        }

        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => {
            let spender = info.sender.as_str();
            let allowance = read_allowance(deps.as_ref(), &owner, spender)?;
            let new_allowance = allowance
                .checked_sub(amount)
                .ok_or_else(|| StdError::msg("insufficient allowance"))?;

            let owner_bal = read_balance(deps.as_ref(), &owner)?;
            let new_owner_bal = owner_bal
                .checked_sub(amount)
                .ok_or_else(|| StdError::msg("insufficient funds for transfer_from"))?;
            let recip_bal = read_balance(deps.as_ref(), &recipient)?;
            let new_recip_bal = recip_bal
                .checked_add(amount)
                .ok_or_else(|| StdError::msg("recipient balance overflow"))?;

            write_allowance(&mut deps, &owner, spender, new_allowance);
            write_balance(&mut deps, &owner, new_owner_bal);
            write_balance(&mut deps, &recipient, new_recip_bal);

            Ok(Response::new()
                .add_attribute("action", "transfer_from")
                .add_attribute("spender", spender)
                .add_attribute("owner", owner)
                .add_attribute("to", recipient)
                .add_attribute("amount", amount.to_string()))
        }

        ExecuteMsg::Approve { spender, amount } => {
            let owner = info.sender.as_str();
            write_allowance(&mut deps, owner, &spender, amount);

            Ok(Response::new()
                .add_attribute("action", "approve")
                .add_attribute("owner", owner)
                .add_attribute("spender", spender)
                .add_attribute("amount", amount.to_string()))
        }

        ExecuteMsg::Mint { recipient, amount } => {
            let mut info_state = read_token_info(deps.as_ref())?;
            match &info_state.minter {
                Some(minter) if minter == info.sender.as_str() => {}
                _ => return Err(StdError::msg("only designated minter can mint")),
            }

            let new_total = info_state
                .total_supply
                .checked_add(amount)
                .ok_or_else(|| StdError::msg("total supply overflow on mint"))?;
            info_state.total_supply = new_total;
            write_token_info(&mut deps, &info_state)?;

            let recip_bal = read_balance(deps.as_ref(), &recipient)?;
            let new_recip_bal = recip_bal
                .checked_add(amount)
                .ok_or_else(|| StdError::msg("recipient balance overflow on mint"))?;
            write_balance(&mut deps, &recipient, new_recip_bal);

            Ok(Response::new()
                .add_attribute("action", "mint")
                .add_attribute("recipient", recipient)
                .add_attribute("amount", amount.to_string()))
        }

        ExecuteMsg::Burn { amount } => {
            let sender = info.sender.as_str();
            let sender_bal = read_balance(deps.as_ref(), sender)?;
            let new_sender_bal = sender_bal
                .checked_sub(amount)
                .ok_or_else(|| StdError::msg("insufficient balance to burn"))?;

            let mut info_state = read_token_info(deps.as_ref())?;
            let new_total = info_state
                .total_supply
                .checked_sub(amount)
                .ok_or_else(|| StdError::msg("total supply underflow on burn"))?;
            info_state.total_supply = new_total;
            write_token_info(&mut deps, &info_state)?;

            write_balance(&mut deps, sender, new_sender_bal);

            Ok(Response::new()
                .add_attribute("action", "burn")
                .add_attribute("from", sender)
                .add_attribute("amount", amount.to_string()))
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => {
            let balance = read_balance(deps, &address)?;
            to_json_binary(&BalanceResponse { balance })
        }
        QueryMsg::Allowance { owner, spender } => {
            let allowance = read_allowance(deps, &owner, &spender)?;
            to_json_binary(&AllowanceResponse { allowance })
        }
        QueryMsg::TokenInfo {} => {
            let info = read_token_info(deps)?;
            to_json_binary(&info)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::Addr;

    #[test]
    fn test_token_lifecycle() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let alice = Addr::unchecked("sprax1alice");
        let bob = Addr::unchecked("sprax1bob");
        let minter = Addr::unchecked("sprax1minter");

        // 1. Instantiate
        let instantiate_msg = InstantiateMsg {
            name: "Sprax Token".to_string(),
            symbol: "SPRX20".to_string(),
            decimals: 18,
            initial_balances: vec![InitialBalance {
                address: alice.to_string(),
                amount: 1_000_000,
            }],
            minter: Some(minter.to_string()),
        };
        let info = message_info(&alice, &[]);
        let res = instantiate(deps.as_mut(), env.clone(), info, instantiate_msg).unwrap();
        assert_eq!(res.attributes[0].value, "instantiate");

        // Check initial balance
        let query_bal = QueryMsg::Balance {
            address: alice.to_string(),
        };
        let bal_bin = query(deps.as_ref(), env.clone(), query_bal).unwrap();
        let bal_res: BalanceResponse = from_json(bal_bin).unwrap();
        assert_eq!(bal_res.balance, 1_000_000);

        // 2. Transfer from Alice to Bob
        let transfer_msg = ExecuteMsg::Transfer {
            recipient: bob.to_string(),
            amount: 250_000,
        };
        let info = message_info(&alice, &[]);
        execute(deps.as_mut(), env.clone(), info, transfer_msg).unwrap();

        // Check Bob balance
        let bob_bal = read_balance(deps.as_ref(), bob.as_str()).unwrap();
        assert_eq!(bob_bal, 250_000);

        // Check Alice balance
        let alice_bal = read_balance(deps.as_ref(), alice.as_str()).unwrap();
        assert_eq!(alice_bal, 750_000);

        // 3. Approve and TransferFrom
        let approve_msg = ExecuteMsg::Approve {
            spender: bob.to_string(),
            amount: 100_000,
        };
        let info = message_info(&alice, &[]);
        execute(deps.as_mut(), env.clone(), info, approve_msg).unwrap();

        let transfer_from_msg = ExecuteMsg::TransferFrom {
            owner: alice.to_string(),
            recipient: bob.to_string(),
            amount: 60_000,
        };
        let info = message_info(&bob, &[]);
        execute(deps.as_mut(), env.clone(), info, transfer_from_msg).unwrap();

        assert_eq!(read_balance(deps.as_ref(), bob.as_str()).unwrap(), 310_000);
        assert_eq!(read_balance(deps.as_ref(), alice.as_str()).unwrap(), 690_000);
        assert_eq!(read_allowance(deps.as_ref(), alice.as_str(), bob.as_str()).unwrap(), 40_000);

        // 4. Mint by minter
        let mint_msg = ExecuteMsg::Mint {
            recipient: bob.to_string(),
            amount: 50_000,
        };
        let info = message_info(&minter, &[]);
        execute(deps.as_mut(), env.clone(), info, mint_msg).unwrap();
        assert_eq!(read_balance(deps.as_ref(), bob.as_str()).unwrap(), 360_000);

        // Non-minter cannot mint
        let info_fail = message_info(&alice, &[]);
        let mint_err = execute(deps.as_mut(), env.clone(), info_fail, mint_msg).unwrap_err();
        assert!(mint_err.to_string().contains("only designated minter"));

        // 5. Burn
        let burn_msg = ExecuteMsg::Burn { amount: 10_000 };
        let info = message_info(&bob, &[]);
        execute(deps.as_mut(), env.clone(), info, burn_msg).unwrap();
        assert_eq!(read_balance(deps.as_ref(), bob.as_str()).unwrap(), 350_000);

        // Verify total supply
        let token_info = read_token_info(deps.as_ref()).unwrap();
        // Initial 1_000_000 + 50_000 minted - 10_000 burned = 1_040_000
        assert_eq!(token_info.total_supply, 1_040_000);
    }
}
