use crate::error::TypeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

pub const ADDRESS_LENGTH: usize = 20;
pub const ADDRESS_HRP: &str = "sprax";

/// 20-byte account address in the SPRX Protocol.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Address(pub [u8; ADDRESS_LENGTH]);

impl Address {
    pub const ZERO: Self = Self([0u8; ADDRESS_LENGTH]);

    #[must_use]
    pub const fn new(bytes: [u8; ADDRESS_LENGTH]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; ADDRESS_LENGTH] {
        &self.0
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, TypeError> {
        if slice.len() != ADDRESS_LENGTH {
            return Err(TypeError::InvalidAddress(format!(
                "expected {ADDRESS_LENGTH} bytes, found {}",
                slice.len()
            )));
        }
        let mut arr = [0u8; ADDRESS_LENGTH];
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

    /// Encodes the address into standard Bech32 format (e.g. `sprax1...`).
    pub fn to_bech32(&self) -> Result<String, TypeError> {
        let hrp = bech32::Hrp::parse(ADDRESS_HRP)
            .map_err(|e| TypeError::InvalidAddress(format!("invalid hrp: {e}")))?;
        bech32::encode::<bech32::Bech32>(hrp, &self.0)
            .map_err(|e| TypeError::InvalidAddress(format!("bech32 encode error: {e}")))
    }

    /// Decodes an address from Bech32 format (e.g. `sprax1...`).
    pub fn from_bech32(s: &str) -> Result<Self, TypeError> {
        let (hrp, data) = bech32::decode(s)
            .map_err(|e| TypeError::InvalidAddress(format!("bech32 decode error: {e}")))?;
        if hrp.as_str() != ADDRESS_HRP {
            return Err(TypeError::InvalidAddress(format!(
                "invalid prefix '{}', expected '{}'",
                hrp.as_str(),
                ADDRESS_HRP
            )));
        }
        Self::from_slice(&data)
    }

    /// Parses an address from either Bech32 (`sprax1...`) or Hex (`0x...`) format.
    pub fn parse(s: &str) -> Result<Self, TypeError> {
        if s.starts_with(ADDRESS_HRP) {
            Self::from_bech32(s)
        } else {
            Self::from_hex(s)
        }
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Address({})",
            self.to_bech32().unwrap_or_else(|_| self.to_hex())
        )
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_bech32() {
            Ok(b32) => write!(f, "{b32}"),
            Err(_) => write!(f, "{}", self.to_hex()),
        }
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.to_bech32() {
            Ok(b32) => serializer.serialize_str(&b32),
            Err(_) => serializer.serialize_str(&self.to_hex()),
        }
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_hex_and_bech32() {
        let raw = [42u8; 20];
        let addr = Address::new(raw);
        let bech32_str = addr.to_bech32().unwrap();
        assert!(bech32_str.starts_with("sprax1"));

        let parsed_bech32 = Address::from_bech32(&bech32_str).unwrap();
        assert_eq!(addr, parsed_bech32);

        let hex_str = addr.to_hex();
        let parsed_hex = Address::from_hex(&hex_str).unwrap();
        assert_eq!(addr, parsed_hex);

        let parsed_generic = Address::parse(&bech32_str).unwrap();
        assert_eq!(addr, parsed_generic);
    }
}
