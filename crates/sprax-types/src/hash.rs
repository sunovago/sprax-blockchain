use crate::error::TypeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Standard 32-byte cryptographic hash.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Hash32(pub [u8; 32]);

impl Hash32 {
    pub const ZERO: Self = Self([0u8; 32]);

    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, TypeError> {
        if slice.len() != 32 {
            return Err(TypeError::InvalidHashLength {
                expected: 32,
                found: slice.len(),
            });
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(slice);
        Ok(Self(arr))
    }

    pub fn from_hex(hex_str: &str) -> Result<Self, TypeError> {
        let clean_hex = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let bytes = hex::decode(clean_hex).map_err(|e| TypeError::HexDecodeError(e.to_string()))?;
        Self::from_slice(&bytes)
    }

    #[must_use]
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }
}

impl fmt::Debug for Hash32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash32({})", self.to_hex())
    }
}

impl fmt::Display for Hash32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Serialize for Hash32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Hash32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash32_hex_roundtrip() {
        let raw = [7u8; 32];
        let hash = Hash32::new(raw);
        let hex_str = hash.to_hex();
        assert!(hex_str.starts_with("0x"));
        let parsed = Hash32::from_hex(&hex_str).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn test_hash32_invalid_len() {
        let slice = [1u8; 16];
        assert!(Hash32::from_slice(&slice).is_err());
    }
}
