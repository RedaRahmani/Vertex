//! Fee calculation utilities shared across programs.

use crate::decimals::{Decimal, DecimalError};
use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};

/// Protocol fee configuration using numerator/denominator representation.
#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize,
)]
pub struct FeeConfig {
    /// Numerator for the fee fraction.
    pub numerator: u64,
    /// Denominator for the fee fraction.
    pub denominator: u64,
    /// Destination for protocol fees.
    pub fee_vault: Pubkey,
}

impl FeeConfig {
    /// Creates a new fee configuration ensuring sane defaults.
    pub fn new(numerator: u64, denominator: u64, fee_vault: Pubkey) -> Result<Self> {
        require!(denominator > 0, FeeError::InvalidFee);
        require!(numerator <= denominator / 2, FeeError::InvalidFee); // 50% cap
        Ok(Self {
            numerator,
            denominator,
            fee_vault,
        })
    }

    /// Applies fee to a decimal amount.
    pub fn apply(&self, amount: Decimal) -> Result<Decimal> {
        if self.numerator == 0 {
            return Ok(Decimal::from_scaled(0));
        }
        let numerator = self.numerator as u128;
        let denominator = self.denominator as u128;
        let scaled = amount.as_scaled();
        let fee_scaled = scaled
            .checked_mul(numerator)
            .ok_or_else(|| anchor_lang::error::Error::from(DecimalError))?
            .checked_div(denominator)
            .ok_or_else(|| anchor_lang::error::Error::from(DecimalError))?;
        Ok(Decimal::from_scaled_raw(fee_scaled))
    }

    /// Returns true if fee destination is set.
    pub fn is_enabled(&self) -> bool {
        self.numerator > 0
    }
}

/// Errors thrown by fee helpers.
#[error_code]
pub enum FeeError {
    /// Invalid fee configuration.
    #[msg("Fee numerator/denominator invalid")]
    InvalidFee,
}
