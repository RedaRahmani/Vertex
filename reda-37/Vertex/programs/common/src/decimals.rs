//! Deterministic fixed-point decimal math built on top of u128.

use anchor_lang::prelude::*;
use num_traits::{FromPrimitive, ToPrimitive};

/// Scaling factor for fixed-point math (1e9 precision).
pub const SCALE: u128 = 1_000_000_000u128;

/// Error emitted when decimal math fails.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("decimal overflow")]
pub struct DecimalError;

/// Fixed-point decimal backed by u128 with 1e9 precision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Decimal(u128);

impl Decimal {
    /// Creates decimal from raw scaled integer (already multiplied by SCALE).
    pub fn from_scaled(value: u64) -> Self {
        Self(value as u128)
    }

    /// Creates decimal from raw u128 scaled integer.
    pub fn from_scaled_raw(value: u128) -> Self {
        Self(value)
    }

    /// Creates decimal from integer, applying scale.
    pub fn from_integer(value: u64) -> Self {
        Self(value as u128 * SCALE)
    }

    /// Adds two decimals with overflow checks.
    pub fn checked_add(self, other: Self) -> Result<Self> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or_else(|| DecimalError.into())
    }

    /// Subtracts decimals.
    pub fn checked_sub(self, other: Self) -> Result<Self> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .ok_or_else(|| DecimalError.into())
    }

    /// Multiplies decimals returning scaled result.
    pub fn checked_mul(self, other: Self) -> Result<Self> {
        self.0
            .checked_mul(other.0)
            .and_then(|v| v.checked_div(SCALE))
            .map(Self)
            .ok_or_else(|| DecimalError.into())
    }

    /// Multiplies decimal by integer.
    pub fn mul_int(self, rhs: u64) -> Result<Self> {
        self.0
            .checked_mul(rhs as u128)
            .map(Self)
            .ok_or_else(|| DecimalError.into())
    }

    /// Divides decimal by integer.
    pub fn div_int(self, rhs: u64) -> Result<Self> {
        self.0
            .checked_div(rhs as u128)
            .map(Self)
            .ok_or_else(|| DecimalError.into())
    }

    /// Converts to u64 by truncating fractional portion.
    pub fn to_u64(self) -> Result<u64> {
        self.0
            .checked_div(SCALE)
            .and_then(|v| u64::try_from(v).ok())
            .ok_or_else(|| DecimalError.into())
    }

    /// Returns inner scaled representation.
    pub const fn as_scaled(&self) -> u128 {
        self.0
    }
}

impl From<u64> for Decimal {
    fn from(value: u64) -> Self {
        Self::from_integer(value)
    }
}

impl From<u128> for Decimal {
    fn from(value: u128) -> Self {
        Self::from_scaled_raw(value * SCALE)
    }
}

impl FromPrimitive for Decimal {
    fn from_i64(n: i64) -> Option<Self> {
        if n < 0 {
            None
        } else {
            Some(Self::from_integer(n as u64))
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(Self::from_integer(n))
    }
}

impl ToPrimitive for Decimal {
    fn to_i64(&self) -> Option<i64> {
        <Decimal>::to_u64(*self)
            .ok()
            .and_then(|v| i64::try_from(v).ok())
    }

    fn to_u64(&self) -> Option<u64> {
        <Decimal>::to_u64(*self).ok()
    }
}

#[cfg(all(test, not(target_arch = "bpf")))]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn mul_int_then_div_int_roundtrips(n in 0u64..1_000_000, k in 1u64..1_000) {
            let d = Decimal::from_integer(n);
            let scaled = d.mul_int(k).unwrap();
            let back = scaled.div_int(k).unwrap();
            prop_assert!(back.to_u64().unwrap() <= n);
        }
    }
}

/// Convenience wrapper representing decimal ratio in u64 basis points.
#[derive(Clone, Copy, Debug)]
pub struct DecimalRatio(pub u64);

impl DecimalRatio {
    /// Multiplies a u64 by decimal ratio in **basis points** (10_000), returning u64 (floored).
    pub fn apply(&self, value: u64) -> Result<u64> {
        let scaled = (value as u128)
            .checked_mul(self.0 as u128)
            .ok_or_else(|| anchor_lang::error::Error::from(DecimalError))?;
        (scaled / 10_000u128)
            .try_into()
            .map_err(|_| DecimalError.into())
    }
}
