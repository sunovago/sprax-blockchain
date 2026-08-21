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
