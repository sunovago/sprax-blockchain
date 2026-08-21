use serde::{Deserialize, Serialize};
use std::fmt;

/// Deployment and execution profile environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Environment {
    #[default]
    Development,
    Testnet,
    Mainnet,
}

impl Environment {
    #[must_use]
    pub const fn default_chain_id(&self) -> &'static str {
        match self {
            Self::Development => "sprax-devnet-1",
            Self::Testnet => "sprax-testnet-1",
            Self::Mainnet => "sprax-mainnet-1",
        }
    }

    #[must_use]
    pub const fn default_rpc_port(&self) -> u16 {
        match self {
            Self::Development => 26657,
            Self::Testnet => 26657,
            Self::Mainnet => 26657,
        }
    }

    #[must_use]
    pub const fn default_p2p_port(&self) -> u16 {
        match self {
            Self::Development => 26656,
            Self::Testnet => 26656,
            Self::Mainnet => 26656,
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Development => write!(f, "development"),
            Self::Testnet => write!(f, "testnet"),
            Self::Mainnet => write!(f, "mainnet"),
        }
    }
}
