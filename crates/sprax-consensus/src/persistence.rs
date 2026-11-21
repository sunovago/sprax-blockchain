use crate::{
    engine::BftConsensusEngine,
    error::ConsensusError,
    round::{RoundState, RoundStep},
    validator::ValidatorSet,
    vote::Vote,
};
use serde::{Deserialize, Serialize};
use sprax_types::Hash32;
use std::collections::HashMap;
use std::path::Path;

/// Serializable consensus state snapshot for crash-recovery and restart safety.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusSnapshot {
    pub height: u64,
    pub round: u32,
    pub step: RoundStep,
    pub locked_round: Option<u32>,
    pub locked_block: Option<Hash32>,
    pub valid_round: Option<u32>,
    pub valid_block: Option<Hash32>,
    pub val_set: ValidatorSet,
    pub prevotes: Vec<Vote>,
    pub precommits: Vec<Vote>,
    pub proposed_blocks: Vec<(u64, u32, Hash32)>,
}

impl ConsensusSnapshot {
    /// Takes a full point-in-time snapshot from a live consensus engine.
    #[must_use]
    pub fn from_engine(engine: &BftConsensusEngine) -> Self {
        let prevotes = engine.all_prevotes();
        let precommits = engine.all_precommits();
        let proposed_blocks = engine.all_proposed_blocks();

        Self {
            height: engine.current_height(),
            round: engine.current_round(),
            step: engine.current_step(),
            locked_round: engine.locked_round(),
            locked_block: engine.locked_block(),
            valid_round: engine.valid_round(),
            valid_block: engine.valid_block(),
            val_set: engine.validator_set().clone(),
            prevotes,
            precommits,
            proposed_blocks,
        }
    }

    /// Restores a consensus engine from this snapshot.
    #[must_use]
    pub fn restore_engine(&self) -> BftConsensusEngine {
        let state = RoundState {
            height: self.height,
            round: self.round,
            step: self.step,
            locked_round: self.locked_round,
            locked_block: self.locked_block,
            valid_round: self.valid_round,
            valid_block: self.valid_block,
        };

        let mut prevotes_map = HashMap::new();
        for vote in &self.prevotes {
            prevotes_map.insert(
                (vote.height, vote.round, vote.validator_address),
                vote.clone(),
            );
        }

        let mut precommits_map = HashMap::new();
        for vote in &self.precommits {
            precommits_map.insert(
                (vote.height, vote.round, vote.validator_address),
                vote.clone(),
            );
        }

        let mut proposed_map = HashMap::new();
        for (h, r, hash) in &self.proposed_blocks {
            proposed_map.insert((*h, *r), *hash);
        }

        BftConsensusEngine::from_raw_parts(
            state,
            self.val_set.clone(),
            prevotes_map,
            precommits_map,
            proposed_map,
        )
    }

    /// Saves snapshot to a JSON file.
    pub fn save_to_file(&self, path: &Path) -> Result<(), ConsensusError> {
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| ConsensusError::InvalidValidatorSet(format!("json encode error: {e}")))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ConsensusError::InvalidValidatorSet(format!("fs error: {e}")))?;
        }
        std::fs::write(path, json_str)
            .map_err(|e| ConsensusError::InvalidValidatorSet(format!("fs write error: {e}")))?;
        Ok(())
    }

    /// Loads snapshot from a JSON file.
    pub fn load_from_file(path: &Path) -> Result<Self, ConsensusError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConsensusError::InvalidValidatorSet(format!("fs read error: {e}")))?;
        let snapshot: Self = serde_json::from_str(&content)
            .map_err(|e| ConsensusError::InvalidValidatorSet(format!("json parse error: {e}")))?;
        Ok(snapshot)
    }
}
