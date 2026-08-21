use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConsensusError {
    #[error("Invalid vote: {0}")]
    InvalidVote(String),

    #[error("Invalid validator set: {0}")]
    InvalidValidatorSet(String),

    #[error("Quorum not reached: required {required}, found {found}")]
    QuorumNotReached { required: u64, found: u64 },

    #[error("Double sign detected for validator: {0}")]
    DoubleSignDetected(String),

    #[error("Invalid proposal: {0}")]
    InvalidProposal(String),

    #[error("Invalid proposer: {0}")]
    InvalidProposer(String),

    #[error("Round timeout: height {height}, round {round}")]
    RoundTimeout { height: u64, round: u32 },
}
