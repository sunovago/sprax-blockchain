use serde::{Deserialize, Serialize};

/// Pruning strategy options for node storage management.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PruningStrategy {
    /// Archive node: retain all historical block states since genesis.
    Archive,
    /// Full node: retain only the most recent N block states.
    KeepRecent { keep_blocks: u64 },
    /// Validator node: aggressive pruning, retaining only minimal consensus window.
    Aggressive { keep_blocks: u64 },
}

impl Default for PruningStrategy {
    fn default() -> Self {
        Self::KeepRecent {
            keep_blocks: 100_000,
        }
    }
}

impl PruningStrategy {
    #[must_use]
    pub fn keep_blocks(&self) -> Option<u64> {
        match self {
            Self::Archive => None,
            Self::KeepRecent { keep_blocks } | Self::Aggressive { keep_blocks } => {
                Some(*keep_blocks)
            }
        }
    }

    #[must_use]
    pub fn is_archive(&self) -> bool {
        matches!(self, Self::Archive)
    }

    /// Calculates cutoff height for a given tip. Blocks with height < cutoff are eligible for pruning.
    #[must_use]
    pub fn cutoff_height(&self, current_height: u64) -> u64 {
        match self.keep_blocks() {
            Some(keep) => current_height.saturating_sub(keep),
            None => 0,
        }
    }
}

/// Results of a pruning operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PruningStats {
    pub blocks_pruned: usize,
    pub tx_indices_pruned: usize,
    pub earliest_retained_height: u64,
}
