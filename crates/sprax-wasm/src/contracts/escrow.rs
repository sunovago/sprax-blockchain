use crate::{
    error::ContractError,
    gas::GasMeter,
    storage::ContractStorage,
    types::{ContractEvent, ContractResponse, ContractSubMessage, Env, MessageInfo},
};
use serde::{Deserialize, Serialize};
use sprax_types::{Address, Amount};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscrowStatus {
    Locked,
    Released,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowState {
    pub sender: Address,
    pub recipient: Address,
    pub arbiter: Address,
    pub amount: Amount,
    pub timeout_height: u64,
    pub status: EscrowStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscrowExecuteMsg {
    Deposit {},
    Release {},
    Refund {},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscrowQueryMsg {
    GetEscrow {},
}

/// Conditional Payment & Escrow contract implementation.
#[derive(Debug)]
pub struct EscrowContract;

impl EscrowContract {
    const KEY_ESCROW: &'static [u8] = b"escrow_state";

    pub fn instantiate(
        storage: &mut ContractStorage,
        env: &Env,
        info: &MessageInfo,
        gas: &mut GasMeter,
        recipient: Address,
        arbiter: Address,
        timeout_height: u64,
    ) -> Result<ContractResponse, ContractError> {
        let state = EscrowState {
            sender: info.sender,
            recipient,
            arbiter,
            amount: info.funds,
            timeout_height,
            status: EscrowStatus::Locked,
        };

        storage.set(
            &env.contract_address,
            Self::KEY_ESCROW.to_vec(),
            serde_json::to_vec(&state).map_err(|e| ContractError::StorageError(e.to_string()))?,
            gas,
        )?;

        let event = ContractEvent::new("escrow-instantiate")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("recipient", recipient.to_string())
            .add_attribute("arbiter", arbiter.to_string())
            .add_attribute("amount", info.funds.to_string())
            .add_attribute("timeout_height", timeout_height.to_string());

        Ok(ContractResponse::new().add_event(event))
    }

    pub fn execute(
        storage: &mut ContractStorage,
        env: &Env,
        info: &MessageInfo,
        gas: &mut GasMeter,
        msg: EscrowExecuteMsg,
    ) -> Result<ContractResponse, ContractError> {
        let state_bytes = storage
            .get(&env.contract_address, Self::KEY_ESCROW, gas)?
            .ok_or_else(|| ContractError::ContractNotFound("escrow not found".into()))?;
        let mut state: EscrowState = serde_json::from_slice(&state_bytes).unwrap();

        if state.status != EscrowStatus::Locked {
            return Err(ContractError::ExecutionFailed(
                "escrow is no longer in locked state".into(),
            ));
        }

        match msg {
            EscrowExecuteMsg::Deposit {} => {
                state.amount = state
                    .amount
                    .checked_add(info.funds)
                    .map_err(|e| ContractError::ArithmeticError(e.to_string()))?;

                storage.set(
                    &env.contract_address,
                    Self::KEY_ESCROW.to_vec(),
                    serde_json::to_vec(&state).unwrap(),
                    gas,
                )?;

                let event = ContractEvent::new("escrow-deposit")
                    .add_attribute("depositor", info.sender.to_string())
                    .add_attribute("added_amount", info.funds.to_string())
                    .add_attribute("total_amount", state.amount.to_string());

                Ok(ContractResponse::new().add_event(event))
            }
            EscrowExecuteMsg::Release {} => {
                // Arbiter or sender can authorize release
                if info.sender != state.arbiter && info.sender != state.sender {
                    return Err(ContractError::Unauthorized(
                        "only arbiter or sender can release escrow funds".into(),
                    ));
                }

                let payout = state.amount;
                state.status = EscrowStatus::Released;
                state.amount = Amount::ZERO;

                storage.set(
                    &env.contract_address,
                    Self::KEY_ESCROW.to_vec(),
                    serde_json::to_vec(&state).unwrap(),
                    gas,
                )?;

                let event = ContractEvent::new("escrow-release")
                    .add_attribute("recipient", state.recipient.to_string())
                    .add_attribute("payout_amount", payout.to_string());

                let sub_msg = ContractSubMessage::BankSend {
                    to: state.recipient,
                    amount: payout,
                };

                Ok(ContractResponse::new()
                    .add_event(event)
                    .add_message(sub_msg))
            }
            EscrowExecuteMsg::Refund {} => {
                // Arbiter can refund anytime; sender can refund if timeout expired
                let is_arbiter = info.sender == state.arbiter;
                let is_sender_timeout =
                    info.sender == state.sender && env.block_height >= state.timeout_height;

                if !is_arbiter && !is_sender_timeout {
                    return Err(ContractError::Unauthorized(
                        "refund requires arbiter authorization or timeout expiration".into(),
                    ));
                }

                let refund_amount = state.amount;
                state.status = EscrowStatus::Refunded;
                state.amount = Amount::ZERO;

                storage.set(
                    &env.contract_address,
                    Self::KEY_ESCROW.to_vec(),
                    serde_json::to_vec(&state).unwrap(),
                    gas,
                )?;

                let event = ContractEvent::new("escrow-refund")
                    .add_attribute("sender", state.sender.to_string())
                    .add_attribute("refund_amount", refund_amount.to_string());

                let sub_msg = ContractSubMessage::BankSend {
                    to: state.sender,
                    amount: refund_amount,
                };

                Ok(ContractResponse::new()
                    .add_event(event)
                    .add_message(sub_msg))
            }
        }
    }

    pub fn query(
        storage: &ContractStorage,
        env: &Env,
        msg: EscrowQueryMsg,
    ) -> Result<Vec<u8>, ContractError> {
        match msg {
            EscrowQueryMsg::GetEscrow {} => {
                let state_bytes = storage
                    .get_raw(&env.contract_address, Self::KEY_ESCROW)
                    .ok_or_else(|| ContractError::ContractNotFound("escrow not found".into()))?;
                Ok(state_bytes)
            }
        }
    }
}
