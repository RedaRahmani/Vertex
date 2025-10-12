//! Pricing curve primitives shared by Launchpad and AMM flows.

use crate::decimals::{Decimal, DecimalError};
use crate::fees::FeeConfig;
use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};

/// Curve kinds supported by Keystone launch flows.
#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Default,
)]
#[repr(u8)]
pub enum CurveKind {
    /// Fixed price sale.
    #[default]
    Fixed = 0,
    /// Linear bonding curve.
    Linear = 1,
    /// Exponential bonding curve.
    Exponential = 2,
    /// Sigmoid curve for smoother ramps.
    Sigmoid = 3,
}

// Default is derived above; Fixed is marked as #[default].

/// Parameters for the bonding curve evaluation.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Serialize, Deserialize)]
pub struct CurveConfig {
    /// Kind of curve.
    pub kind: CurveKind,
    /// Base price (scaled by common decimals::SCALE).
    pub base_price: u64,
    /// Slope or multiplier parameter depending on curve.
    pub k: u64,
    /// Optional inflection or midpoint parameter.
    pub x0: u64,
    /// Maximum supply supported by the curve.
    pub max_supply: u64,
    /// Fees applied to trades.
    pub fee_config: FeeConfig,
}

impl CurveConfig {
    /// Validates the config is safe for production usage.
    pub fn assert_valid(&self) -> Result<()> {
        require!(self.max_supply > 0, CurveError::InvalidConfig);
        require!(self.base_price > 0, CurveError::InvalidConfig);
        Ok(())
    }
}

/// Quote for a bonding curve trade.
#[derive(Debug, Clone, PartialEq)]
pub struct CurveQuote {
    /// Amount of base tokens to provide.
    pub base_amount: u64,
    /// Amount of quote tokens to pay/receive.
    pub quote_amount: u64,
    /// Fees charged in quote tokens.
    pub fee_amount: u64,
}

/// Trait implemented by bonding curve calculators.
pub trait PricingCurve {
    /// Computes quote for buying `base_amount` tokens.
    fn quote_buy(&self, supply: u64, base_amount: u64) -> Result<CurveQuote>;
    /// Computes quote for selling `base_amount` tokens back to the curve.
    fn quote_sell(&self, supply: u64, base_amount: u64) -> Result<CurveQuote>;
}

/// Linear bonding curve implementation with integer math.
pub struct LinearCurve<'a> {
    config: &'a CurveConfig,
}

impl<'a> LinearCurve<'a> {
    /// Create new linear curve instance.
    pub fn new(config: &'a CurveConfig) -> Result<Self> {
        config.assert_valid()?;
        Ok(Self { config })
    }

    fn slope(&self) -> Decimal {
        Decimal::from_scaled(self.config.k)
    }
}

impl<'a> PricingCurve for LinearCurve<'a> {
    fn quote_buy(&self, supply: u64, base_amount: u64) -> Result<CurveQuote> {
        require!(base_amount > 0, CurveError::InvalidInput);
        require!(
            supply.saturating_add(base_amount) <= self.config.max_supply,
            CurveError::SupplyExceeded
        );

        let base_amount_dec = Decimal::from(base_amount);
        let supply_dec = Decimal::from(supply);
        let price = Decimal::from_scaled(self.config.base_price).checked_add(
            self.slope()
                .checked_mul(supply_dec.checked_add(base_amount_dec)?)?,
        )?;
        let quote_raw = price.checked_mul(base_amount_dec)?;
        let fee_amount = self.config.fee_config.apply(quote_raw)?;
        let total = quote_raw.checked_add(fee_amount)?;
        Ok(CurveQuote {
            base_amount,
            quote_amount: total.to_u64()?,
            fee_amount: fee_amount.to_u64()?,
        })
    }

    fn quote_sell(&self, supply: u64, base_amount: u64) -> Result<CurveQuote> {
        require!(base_amount > 0, CurveError::InvalidInput);
        require!(base_amount <= supply, CurveError::InsufficientSupply);
        let base_amount_dec = Decimal::from(base_amount);
        let supply_dec = Decimal::from(supply);
        let price = Decimal::from_scaled(self.config.base_price).checked_add(
            self.slope()
                .checked_mul(supply_dec.checked_sub(base_amount_dec)?)?,
        )?;
        let quote_raw = price.checked_mul(base_amount_dec)?;
        let fee_amount = self.config.fee_config.apply(quote_raw)?;
        let net = quote_raw
            .checked_sub(fee_amount)
            .map_err(|_| CurveError::MathOverflow)?;
        Ok(CurveQuote {
            base_amount,
            quote_amount: net.to_u64()?,
            fee_amount: fee_amount.to_u64()?,
        })
    }
}

/// Error codes used by curve helpers.
#[error_code]
pub enum CurveError {
    /// Provided amount would exceed supported supply.
    #[msg("Base amount exceeds remaining supply")]
    SupplyExceeded,
    /// Attempted to sell more tokens than owned by pool.
    #[msg("Insufficient supply for sale")]
    InsufficientSupply,
    /// Curve math overflowed.
    #[msg("Curve arithmetic overflow")]
    MathOverflow,
    /// Invalid curve configuration.
    #[msg("Invalid curve configuration")]
    InvalidConfig,
    /// Invalid trade input.
    #[msg("Invalid trade input")]
    InvalidInput,
}

impl From<DecimalError> for Error {
    fn from(_value: DecimalError) -> Self {
        CurveError::MathOverflow.into()
    }
}

#[cfg(all(test, not(target_arch = "bpf")))]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn cfg(base_price: u64, k: u64, max_supply: u64) -> CurveConfig {
        CurveConfig {
            kind: CurveKind::Linear,
            base_price,
            k,
            x0: 0,
            max_supply,
            fee_config: crate::fees::FeeConfig {
                numerator: 0,
                denominator: 1,
                fee_vault: Pubkey::default(),
            },
        }
    }

    proptest! {
        #[test]
        fn linear_price_monotonic(supply in 0u64..1_000_000, amount in 1u64..10_000, base in 1u64..1_000_000, k in 0u64..1_000) {
            let c = cfg(base, k, u64::MAX);
            let lin = LinearCurve::new(&c).unwrap();
            let q1 = lin.quote_buy(supply, amount).unwrap();
            let q2 = lin.quote_buy(supply + amount, amount).unwrap();
            prop_assert!(q2.quote_amount >= q1.quote_amount);
        }
    }
}
