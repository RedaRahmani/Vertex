//! Authority and role helpers for Keystone programs.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use strum::EnumString;

/// Roles supported across programs.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, EnumString)]
#[non_exhaustive]
pub enum Role {
    /// Global administrator with full control.
    Admin,
    /// Operational role for day-to-day actions.
    Ops,
    /// Treasury operator for fund movements.
    Treasury,
}

impl Role {
    /// Returns PDA seed prefix used to derive authority records.
    pub const fn seed_prefix(&self) -> &'static [u8] {
        match self {
            Role::Admin => b"admin",
            Role::Ops => b"ops",
            Role::Treasury => b"treasury",
        }
    }
}

/// Trait for Anchor accounts to declare canonical authority fields.
pub trait HasAuthority {
    /// Returns the authority responsible for admin actions.
    fn authority(&self) -> &Pubkey;
}

/// Validates signer matches expected authority.
pub fn assert_signer_is(expected: &Pubkey, actual: &Signer) -> Result<()> {
    require_keys_eq!(*expected, actual.key(), ErrorCode::UnauthorizedAuthority);
    Ok(())
}

/// Error codes emitted by authority helper.
#[error_code]
pub enum ErrorCode {
    /// The provided signer does not match required authority.
    #[msg("Provided authority does not match required role")]
    UnauthorizedAuthority,
    /// Multisig requirement failed.
    #[msg("Not enough multisig approvals supplied")]
    MultisigInsufficientApprovals,
}

/// Multisig helper verifying m-of-n approvals via remaining accounts.
pub fn assert_multisig(min_signers: u8, remaining_accounts: &[AccountInfo]) -> Result<()> {
    let approvals = remaining_accounts
        .iter()
        .filter(|acc| acc.is_signer)
        .count() as u8;
    require_gte!(
        approvals,
        min_signers,
        ErrorCode::MultisigInsufficientApprovals
    );
    Ok(())
}

/// Checks whether provided clock slot is past guard slot.
pub fn assert_not_before_slot(current_slot: u64, guard_slot: u64) -> Result<()> {
    require_gte!(current_slot, guard_slot, ErrorCode::UnauthorizedAuthority);
    Ok(())
}
