use crate::{
    error::ContractError,
    gas::GasMeter,
    storage::ContractStorage,
    types::{ContractEvent, ContractResponse, Env, MessageInfo},
};
use serde::{Deserialize, Serialize};
use sprax_types::Address;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Voting,
    Passed,
    Rejected,
    Executed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub title: String,
    pub description: String,
    pub start_height: u64,
    pub end_height: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub abstain_votes: u64,
    pub status: ProposalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovConfig {
    pub voting_period_blocks: u64,
    pub quorum_votes_required: u64,
    pub proposal_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovExecuteMsg {
    SubmitProposal {
        title: String,
        description: String,
    },
    CastVote {
        proposal_id: u64,
        option: VoteOption,
        voting_weight: u64,
    },
    TallyAndExecute {
        proposal_id: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovQueryMsg {
    GetProposal { proposal_id: u64 },
    GetConfig {},
}

/// On-Chain Governance & Proposal Voting contract implementation.
#[derive(Debug)]
pub struct GovernanceContract;

impl GovernanceContract {
    const KEY_CONFIG: &'static [u8] = b"gov_config";

    fn proposal_key(id: u64) -> Vec<u8> {
        let mut k = b"prop_".to_vec();
        k.extend_from_slice(&id.to_be_bytes());
        k
    }

    fn voter_key(proposal_id: u64, voter: &Address) -> Vec<u8> {
        let mut k = b"voted_".to_vec();
        k.extend_from_slice(&proposal_id.to_be_bytes());
        k.extend_from_slice(b"_");
        k.extend_from_slice(voter.as_bytes());
        k
    }

    pub fn instantiate(
        storage: &mut ContractStorage,
        env: &Env,
        info: &MessageInfo,
        gas: &mut GasMeter,
        voting_period_blocks: u64,
        quorum_votes_required: u64,
    ) -> Result<ContractResponse, ContractError> {
        let config = GovConfig {
            voting_period_blocks,
            quorum_votes_required,
            proposal_count: 0,
        };

        storage.set(
            &env.contract_address,
            Self::KEY_CONFIG.to_vec(),
            serde_json::to_vec(&config).map_err(|e| ContractError::StorageError(e.to_string()))?,
            gas,
        )?;

        let event = ContractEvent::new("gov-instantiate")
            .add_attribute("creator", info.sender.to_string())
            .add_attribute("voting_period", voting_period_blocks.to_string())
            .add_attribute("quorum", quorum_votes_required.to_string());

        Ok(ContractResponse::new().add_event(event))
    }

    pub fn execute(
        storage: &mut ContractStorage,
        env: &Env,
        info: &MessageInfo,
        gas: &mut GasMeter,
        msg: GovExecuteMsg,
    ) -> Result<ContractResponse, ContractError> {
        match msg {
            GovExecuteMsg::SubmitProposal { title, description } => {
                let conf_bytes = storage
                    .get(&env.contract_address, Self::KEY_CONFIG, gas)?
                    .ok_or_else(|| {
                        ContractError::ContractNotFound("gov config not found".into())
                    })?;
                let mut config: GovConfig = serde_json::from_slice(&conf_bytes).unwrap();

                config.proposal_count += 1;
                let prop_id = config.proposal_count;

                let proposal = Proposal {
                    id: prop_id,
                    proposer: info.sender,
                    title: title.clone(),
                    description,
                    start_height: env.block_height,
                    end_height: env.block_height + config.voting_period_blocks,
                    yes_votes: 0,
                    no_votes: 0,
                    abstain_votes: 0,
                    status: ProposalStatus::Voting,
                };

                // Save updated config and new proposal
                storage.set(
                    &env.contract_address,
                    Self::KEY_CONFIG.to_vec(),
                    serde_json::to_vec(&config).unwrap(),
                    gas,
                )?;
                storage.set(
                    &env.contract_address,
                    Self::proposal_key(prop_id),
                    serde_json::to_vec(&proposal).unwrap(),
                    gas,
                )?;

                let event = ContractEvent::new("gov-proposal-submitted")
                    .add_attribute("proposal_id", prop_id.to_string())
                    .add_attribute("proposer", info.sender.to_string())
                    .add_attribute("title", title);

                Ok(ContractResponse::new().add_event(event))
            }
            GovExecuteMsg::CastVote {
                proposal_id,
                option,
                voting_weight,
            } => {
                if voting_weight == 0 {
                    return Err(ContractError::InvalidInput(
                        "voting weight must be > 0".into(),
                    ));
                }

                let p_key = Self::proposal_key(proposal_id);
                let prop_bytes = storage
                    .get(&env.contract_address, &p_key, gas)?
                    .ok_or_else(|| {
                        ContractError::ContractNotFound(format!(
                            "proposal #{proposal_id} not found"
                        ))
                    })?;
                let mut proposal: Proposal = serde_json::from_slice(&prop_bytes).unwrap();

                if proposal.status != ProposalStatus::Voting {
                    return Err(ContractError::ExecutionFailed(
                        "proposal voting is closed".into(),
                    ));
                }
                if env.block_height > proposal.end_height {
                    return Err(ContractError::ExecutionFailed(
                        "voting period has expired".into(),
                    ));
                }

                // Assert hasn't already voted
                let v_key = Self::voter_key(proposal_id, &info.sender);
                if storage.get(&env.contract_address, &v_key, gas)?.is_some() {
                    return Err(ContractError::Unauthorized(
                        "voter has already cast a ballot".into(),
                    ));
                }

                match option {
                    VoteOption::Yes => proposal.yes_votes += voting_weight,
                    VoteOption::No => proposal.no_votes += voting_weight,
                    VoteOption::Abstain => proposal.abstain_votes += voting_weight,
                }

                storage.set(
                    &env.contract_address,
                    p_key,
                    serde_json::to_vec(&proposal).unwrap(),
                    gas,
                )?;
                storage.set(&env.contract_address, v_key, vec![1], gas)?;

                let event = ContractEvent::new("gov-vote-cast")
                    .add_attribute("proposal_id", proposal_id.to_string())
                    .add_attribute("voter", info.sender.to_string())
                    .add_attribute("option", format!("{option:?}"))
                    .add_attribute("weight", voting_weight.to_string());

                Ok(ContractResponse::new().add_event(event))
            }
            GovExecuteMsg::TallyAndExecute { proposal_id } => {
                let conf_bytes = storage
                    .get(&env.contract_address, Self::KEY_CONFIG, gas)?
                    .unwrap();
                let config: GovConfig = serde_json::from_slice(&conf_bytes).unwrap();

                let p_key = Self::proposal_key(proposal_id);
                let prop_bytes = storage
                    .get(&env.contract_address, &p_key, gas)?
                    .ok_or_else(|| {
                        ContractError::ContractNotFound(format!(
                            "proposal #{proposal_id} not found"
                        ))
                    })?;
                let mut proposal: Proposal = serde_json::from_slice(&prop_bytes).unwrap();

                if env.block_height < proposal.end_height {
                    return Err(ContractError::ExecutionFailed(
                        "voting period still active".into(),
                    ));
                }
                if proposal.status != ProposalStatus::Voting {
                    return Err(ContractError::ExecutionFailed(
                        "proposal already finalized".into(),
                    ));
                }

                let total_votes = proposal.yes_votes + proposal.no_votes + proposal.abstain_votes;
                if total_votes < config.quorum_votes_required {
                    proposal.status = ProposalStatus::Rejected;
                } else if proposal.yes_votes > proposal.no_votes {
                    proposal.status = ProposalStatus::Passed;
                } else {
                    proposal.status = ProposalStatus::Rejected;
                }

                storage.set(
                    &env.contract_address,
                    p_key,
                    serde_json::to_vec(&proposal).unwrap(),
                    gas,
                )?;

                let event = ContractEvent::new("gov-proposal-tallied")
                    .add_attribute("proposal_id", proposal_id.to_string())
                    .add_attribute("status", format!("{:?}", proposal.status))
                    .add_attribute("yes_votes", proposal.yes_votes.to_string())
                    .add_attribute("no_votes", proposal.no_votes.to_string());

                Ok(ContractResponse::new().add_event(event))
            }
        }
    }

    pub fn query(
        storage: &ContractStorage,
        env: &Env,
        msg: GovQueryMsg,
    ) -> Result<Vec<u8>, ContractError> {
        match msg {
            GovQueryMsg::GetProposal { proposal_id } => {
                let p_key = Self::proposal_key(proposal_id);
                let prop_bytes =
                    storage
                        .get_raw(&env.contract_address, &p_key)
                        .ok_or_else(|| {
                            ContractError::ContractNotFound(format!(
                                "proposal #{proposal_id} not found"
                            ))
                        })?;
                Ok(prop_bytes)
            }
            GovQueryMsg::GetConfig {} => {
                let conf_bytes = storage
                    .get_raw(&env.contract_address, Self::KEY_CONFIG)
                    .ok_or_else(|| {
                        ContractError::ContractNotFound("gov config not found".into())
                    })?;
                Ok(conf_bytes)
            }
        }
    }
}
