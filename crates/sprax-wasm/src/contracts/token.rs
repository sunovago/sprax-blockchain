use crate::{
    error::ContractError,
    gas::GasMeter,
    storage::ContractStorage,
    types::{ContractEvent, ContractResponse, Env, MessageInfo},
};
use serde::{Deserialize, Serialize};
use sprax_types::{Address, Amount};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialBalance {
    pub address: Address,
    pub amount: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Amount,
    pub minter: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenExecuteMsg {
    Transfer {
        recipient: Address,
        amount: Amount,
    },
    TransferFrom {
        owner: Address,
        recipient: Address,
        amount: Amount,
    },
    Approve {
        spender: Address,
        amount: Amount,
    },
    Mint {
        recipient: Address,
        amount: Amount,
    },
    Burn {
        amount: Amount,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenQueryMsg {
    Balance { address: Address },
    Allowance { owner: Address, spender: Address },
    TokenInfo {},
}

/// CW20 / Fungible Token standard contract implementation.
#[derive(Debug)]
pub struct TokenContract;

impl TokenContract {
    const KEY_INFO: &'static [u8] = b"token_info";

    fn balance_key(addr: &Address) -> Vec<u8> {
        let mut k = b"bal_".to_vec();
        k.extend_from_slice(addr.as_bytes());
        k
    }

    fn allowance_key(owner: &Address, spender: &Address) -> Vec<u8> {
        let mut k = b"allow_".to_vec();
        k.extend_from_slice(owner.as_bytes());
        k.extend_from_slice(b"_");
        k.extend_from_slice(spender.as_bytes());
        k
    }

    #[allow(clippy::too_many_arguments)]
    pub fn instantiate(
        storage: &mut ContractStorage,
        env: &Env,
        info: &MessageInfo,
        gas: &mut GasMeter,
        name: String,
        symbol: String,
        decimals: u8,
        initial_balances: Vec<InitialBalance>,
        minter: Option<Address>,
    ) -> Result<ContractResponse, ContractError> {
        let mut total_supply = Amount::ZERO;

        for ib in initial_balances {
            total_supply = total_supply
                .checked_add(ib.amount)
                .map_err(|e| ContractError::ArithmeticError(e.to_string()))?;

            let b_key = Self::balance_key(&ib.address);
            storage.set(
                &env.contract_address,
                b_key,
                serde_json::to_vec(&ib.amount)
                    .map_err(|e| ContractError::StorageError(e.to_string()))?,
                gas,
            )?;
        }

        let token_info = TokenInfo {
            name: name.clone(),
            symbol: symbol.clone(),
            decimals,
            total_supply,
            minter,
        };

        storage.set(
            &env.contract_address,
            Self::KEY_INFO.to_vec(),
            serde_json::to_vec(&token_info)
                .map_err(|e| ContractError::StorageError(e.to_string()))?,
            gas,
        )?;

        let event = ContractEvent::new("cw20-instantiate")
            .add_attribute("creator", info.sender.to_string())
            .add_attribute("name", name)
            .add_attribute("symbol", symbol)
            .add_attribute("total_supply", total_supply.to_string());

        Ok(ContractResponse::new().add_event(event))
    }

    pub fn execute(
        storage: &mut ContractStorage,
        env: &Env,
        info: &MessageInfo,
        gas: &mut GasMeter,
        msg: TokenExecuteMsg,
    ) -> Result<ContractResponse, ContractError> {
        match msg {
            TokenExecuteMsg::Transfer { recipient, amount } => {
                Self::perform_transfer(storage, env, &info.sender, &recipient, amount, gas)?;
                let event = ContractEvent::new("cw20-transfer")
                    .add_attribute("from", info.sender.to_string())
                    .add_attribute("to", recipient.to_string())
                    .add_attribute("amount", amount.to_string());
                Ok(ContractResponse::new().add_event(event))
            }
            TokenExecuteMsg::TransferFrom {
                owner,
                recipient,
                amount,
            } => {
                // Check allowance
                let allow_key = Self::allowance_key(&owner, &info.sender);
                let allowance: Amount = storage
                    .get(&env.contract_address, &allow_key, gas)?
                    .and_then(|v| serde_json::from_slice(&v).ok())
                    .unwrap_or(Amount::ZERO);

                if allowance < amount {
                    return Err(ContractError::InsufficientFunds {
                        balance: allowance.to_string(),
                        required: amount.to_string(),
                    });
                }

                let new_allowance = allowance
                    .checked_sub(amount)
                    .map_err(|e| ContractError::ArithmeticError(e.to_string()))?;

                storage.set(
                    &env.contract_address,
                    allow_key,
                    serde_json::to_vec(&new_allowance).unwrap(),
                    gas,
                )?;

                Self::perform_transfer(storage, env, &owner, &recipient, amount, gas)?;

                let event = ContractEvent::new("cw20-transfer-from")
                    .add_attribute("owner", owner.to_string())
                    .add_attribute("spender", info.sender.to_string())
                    .add_attribute("recipient", recipient.to_string())
                    .add_attribute("amount", amount.to_string());

                Ok(ContractResponse::new().add_event(event))
            }
            TokenExecuteMsg::Approve { spender, amount } => {
                let allow_key = Self::allowance_key(&info.sender, &spender);
                storage.set(
                    &env.contract_address,
                    allow_key,
                    serde_json::to_vec(&amount).unwrap(),
                    gas,
                )?;

                let event = ContractEvent::new("cw20-approve")
                    .add_attribute("owner", info.sender.to_string())
                    .add_attribute("spender", spender.to_string())
                    .add_attribute("amount", amount.to_string());

                Ok(ContractResponse::new().add_event(event))
            }
            TokenExecuteMsg::Mint { recipient, amount } => {
                let info_bytes = storage
                    .get(&env.contract_address, Self::KEY_INFO, gas)?
                    .ok_or_else(|| ContractError::ContractNotFound("missing token info".into()))?;
                let mut token_info: TokenInfo = serde_json::from_slice(&info_bytes).unwrap();

                if token_info.minter != Some(info.sender) {
                    return Err(ContractError::Unauthorized(
                        "only minter can mint new tokens".into(),
                    ));
                }

                token_info.total_supply = token_info
                    .total_supply
                    .checked_add(amount)
                    .map_err(|e| ContractError::ArithmeticError(e.to_string()))?;

                storage.set(
                    &env.contract_address,
                    Self::KEY_INFO.to_vec(),
                    serde_json::to_vec(&token_info).unwrap(),
                    gas,
                )?;

                let rec_key = Self::balance_key(&recipient);
                let current_bal: Amount = storage
                    .get(&env.contract_address, &rec_key, gas)?
                    .and_then(|v| serde_json::from_slice(&v).ok())
                    .unwrap_or(Amount::ZERO);

                let new_bal = current_bal
                    .checked_add(amount)
                    .map_err(|e| ContractError::ArithmeticError(e.to_string()))?;

                storage.set(
                    &env.contract_address,
                    rec_key,
                    serde_json::to_vec(&new_bal).unwrap(),
                    gas,
                )?;

                let event = ContractEvent::new("cw20-mint")
                    .add_attribute("recipient", recipient.to_string())
                    .add_attribute("amount", amount.to_string());

                Ok(ContractResponse::new().add_event(event))
            }
            TokenExecuteMsg::Burn { amount } => {
                let sender_key = Self::balance_key(&info.sender);
                let current_bal: Amount = storage
                    .get(&env.contract_address, &sender_key, gas)?
                    .and_then(|v| serde_json::from_slice(&v).ok())
                    .unwrap_or(Amount::ZERO);

                if current_bal < amount {
                    return Err(ContractError::InsufficientFunds {
                        balance: current_bal.to_string(),
                        required: amount.to_string(),
                    });
                }

                let new_bal = current_bal
                    .checked_sub(amount)
                    .map_err(|e| ContractError::ArithmeticError(e.to_string()))?;

                storage.set(
                    &env.contract_address,
                    sender_key,
                    serde_json::to_vec(&new_bal).unwrap(),
                    gas,
                )?;

                let info_bytes = storage
                    .get(&env.contract_address, Self::KEY_INFO, gas)?
                    .unwrap();
                let mut token_info: TokenInfo = serde_json::from_slice(&info_bytes).unwrap();
                token_info.total_supply = token_info
                    .total_supply
                    .checked_sub(amount)
                    .map_err(|e| ContractError::ArithmeticError(e.to_string()))?;

                storage.set(
                    &env.contract_address,
                    Self::KEY_INFO.to_vec(),
                    serde_json::to_vec(&token_info).unwrap(),
                    gas,
                )?;

                let event = ContractEvent::new("cw20-burn")
                    .add_attribute("from", info.sender.to_string())
                    .add_attribute("amount", amount.to_string());

                Ok(ContractResponse::new().add_event(event))
            }
        }
    }

    fn perform_transfer(
        storage: &mut ContractStorage,
        env: &Env,
        from: &Address,
        to: &Address,
        amount: Amount,
        gas: &mut GasMeter,
    ) -> Result<(), ContractError> {
        if amount.is_zero() {
            return Ok(());
        }

        let from_key = Self::balance_key(from);
        let from_bal: Amount = storage
            .get(&env.contract_address, &from_key, gas)?
            .and_then(|v| serde_json::from_slice(&v).ok())
            .unwrap_or(Amount::ZERO);

        if from_bal < amount {
            return Err(ContractError::InsufficientFunds {
                balance: from_bal.to_string(),
                required: amount.to_string(),
            });
        }

        let to_key = Self::balance_key(to);
        let to_bal: Amount = storage
            .get(&env.contract_address, &to_key, gas)?
            .and_then(|v| serde_json::from_slice(&v).ok())
            .unwrap_or(Amount::ZERO);

        let new_from_bal = from_bal
            .checked_sub(amount)
            .map_err(|e| ContractError::ArithmeticError(e.to_string()))?;
        let new_to_bal = to_bal
            .checked_add(amount)
            .map_err(|e| ContractError::ArithmeticError(e.to_string()))?;

        storage.set(
            &env.contract_address,
            from_key,
            serde_json::to_vec(&new_from_bal).unwrap(),
            gas,
        )?;
        storage.set(
            &env.contract_address,
            to_key,
            serde_json::to_vec(&new_to_bal).unwrap(),
            gas,
        )?;

        Ok(())
    }

    pub fn query(
        storage: &ContractStorage,
        env: &Env,
        msg: TokenQueryMsg,
    ) -> Result<Vec<u8>, ContractError> {
        match msg {
            TokenQueryMsg::Balance { address } => {
                let key = Self::balance_key(&address);
                let bal: Amount = storage
                    .get_raw(&env.contract_address, &key)
                    .and_then(|v| serde_json::from_slice(&v).ok())
                    .unwrap_or(Amount::ZERO);
                Ok(serde_json::to_vec(&bal).unwrap())
            }
            TokenQueryMsg::Allowance { owner, spender } => {
                let key = Self::allowance_key(&owner, &spender);
                let allowance: Amount = storage
                    .get_raw(&env.contract_address, &key)
                    .and_then(|v| serde_json::from_slice(&v).ok())
                    .unwrap_or(Amount::ZERO);
                Ok(serde_json::to_vec(&allowance).unwrap())
            }
            TokenQueryMsg::TokenInfo {} => {
                let info = storage
                    .get_raw(&env.contract_address, Self::KEY_INFO)
                    .ok_or_else(|| ContractError::ContractNotFound("missing token info".into()))?;
                Ok(info)
            }
        }
    }
}
