use crate::error::TypeError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Strongly typed ChainID identifier (e.g. `sprax-mainnet-1`, `sprax-testnet-1`, `sprax-devnet-1`).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChainId(String);

impl ChainId {
    pub const MAINNET: &'static str = "sprax-mainnet-1";
    pub const TESTNET: &'static str = "sprax-testnet-1";
    pub const DEVNET: &'static str = "sprax-devnet-1";

    pub fn new(id: impl Into<String>) -> Result<Self, TypeError> {
        let s = id.into();
        Self::validate(&s)?;
        Ok(Self(s))
    }

    pub fn validate(s: &str) -> Result<(), TypeError> {
        if s.is_empty() || s.len() > 64 {
            return Err(TypeError::InvalidChainId(
                "chain ID must be between 1 and 64 characters".into(),
            ));
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            return Err(TypeError::InvalidChainId(
                "chain ID must contain only lowercase alphanumeric characters, dashes, or underscores".into(),
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChainId({})", self.0)
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ChainId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_chain_ids() {
        assert!(ChainId::new("sprax-mainnet-1").is_ok());
        assert!(ChainId::new("sprax-testnet-1").is_ok());
        assert!(ChainId::new("sprax-devnet-1").is_ok());
    }

    #[test]
    fn test_invalid_chain_ids() {
        assert!(ChainId::new("").is_err());
        assert!(ChainId::new("SPRAX_MAINNET").is_err()); // uppercase not allowed
        assert!(ChainId::new("sprax mainnet").is_err()); // space not allowed
    }
}
