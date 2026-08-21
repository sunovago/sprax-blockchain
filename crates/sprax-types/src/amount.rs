use crate::error::TypeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

pub const ATTO_SPRX_PER_SPRX: u128 = 1_000_000_000_000_000_000; // 10^18
pub const NANO_SPRX_PER_SPRX: u128 = 1_000_000_000; // 10^9

/// Precise representation of native SPRX value in base atomic units (`atto-SPRX` = 10^-18 SPRX).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Amount(pub u128);

impl Amount {
    pub const ZERO: Self = Self(0);
    pub const ONE_SPRX: Self = Self(ATTO_SPRX_PER_SPRX);

    #[must_use]
    pub const fn from_atto(atto: u128) -> Self {
        Self(atto)
    }

    #[must_use]
    pub const fn from_nano(nano: u128) -> Self {
        Self(nano.saturating_mul(1_000_000_000))
    }

    pub fn from_sprx_whole(sprx: u128) -> Result<Self, TypeError> {
        sprx.checked_mul(ATTO_SPRX_PER_SPRX)
            .map(Self)
            .ok_or_else(|| TypeError::AmountOverflow("whole sprx amount too large".into()))
    }

    pub fn from_atto_str(s: &str) -> Result<Self, TypeError> {
        let clean = s.trim().trim_end_matches("SPRX").trim();
        clean
            .parse::<u128>()
            .map(Self)
            .map_err(|e| TypeError::InvalidAmount(format!("invalid atto amount string '{s}': {e}")))
    }

    #[must_use]
    pub const fn as_atto(&self) -> u128 {
        self.0
    }

    pub fn checked_add(&self, other: Self) -> Result<Self, TypeError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or_else(|| TypeError::AmountOverflow("addition overflow".into()))
    }

    pub fn checked_sub(&self, other: Self) -> Result<Self, TypeError> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .ok_or_else(|| TypeError::AmountUnderflow("subtraction underflow".into()))
    }

    pub fn checked_mul(&self, scalar: u128) -> Result<Self, TypeError> {
        self.0
            .checked_mul(scalar)
            .map(Self)
            .ok_or_else(|| TypeError::AmountOverflow("multiplication overflow".into()))
    }

    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Formats amount in human-readable SPRX (e.g. "12.500000000000000000 SPRX").
    #[must_use]
    pub fn to_sprx_string(&self) -> String {
        let whole = self.0 / ATTO_SPRX_PER_SPRX;
        let frac = self.0 % ATTO_SPRX_PER_SPRX;
        if frac == 0 {
            format!("{whole} SPRX")
        } else {
            let frac_str = format!("{frac:018}");
            let trimmed = frac_str.trim_end_matches('0');
            format!("{whole}.{trimmed} SPRX")
        }
    }
}

impl fmt::Debug for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Amount({} atto-SPRX | {})",
            self.0,
            self.to_sprx_string()
        )
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_sprx_string())
    }
}

impl Serialize for Amount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as string to preserve precision across JSON/JS clients
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let val = s.parse::<u128>().map_err(serde::de::Error::custom)?;
        Ok(Self(val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_arithmetic() {
        let a1 = Amount::from_sprx_whole(10).unwrap();
        let a2 = Amount::from_sprx_whole(5).unwrap();
        let sum = a1.checked_add(a2).unwrap();
        assert_eq!(sum, Amount::from_sprx_whole(15).unwrap());
        assert_eq!(sum.to_sprx_string(), "15 SPRX");

        let diff = a1.checked_sub(a2).unwrap();
        assert_eq!(diff, Amount::from_sprx_whole(5).unwrap());

        assert!(a2.checked_sub(a1).is_err());
    }

    #[test]
    fn test_fractional_sprx_display() {
        let half_sprx = Amount::from_atto(500_000_000_000_000_000);
        assert_eq!(half_sprx.to_sprx_string(), "0.5 SPRX");
    }
}
