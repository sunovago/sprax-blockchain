use serde::{Deserialize, Serialize};
use sprax_types::Hash32;

/// Discrete steps within a BFT consensus round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoundStep {
    NewHeight,
    NewRound,
    Propose,
    Prevote,
    Precommit,
    Commit,
}

/// Internal state tracker for consensus rounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundState {
    pub height: u64,
    pub round: u32,
    pub step: RoundStep,
    pub locked_round: Option<u32>,
    pub locked_block: Option<Hash32>,
    pub valid_round: Option<u32>,
    pub valid_block: Option<Hash32>,
}

impl RoundState {
    pub fn new(height: u64) -> Self {
        Self {
            height,
            round: 0,
            step: RoundStep::NewHeight,
            locked_round: None,
            locked_block: None,
            valid_round: None,
            valid_block: None,
        }
    }
}

/// Timeout parameters for BFT consensus phases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusTimeoutConfig {
    pub timeout_propose_ms: u64,
    pub timeout_prevote_ms: u64,
    pub timeout_precommit_ms: u64,
    pub timeout_commit_ms: u64,
}

impl Default for ConsensusTimeoutConfig {
    fn default() -> Self {
        Self {
            timeout_propose_ms: 1000,
            timeout_prevote_ms: 500,
            timeout_precommit_ms: 500,
            timeout_commit_ms: 500,
        }
    }
}
