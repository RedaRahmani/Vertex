//! Shared error codes across Keystone programs.

use anchor_lang::prelude::*;

/// Common error namespace reused across programs.
#[error_code]
pub enum CommonError {
    /// Account constraints not satisfied.
    #[msg("Constraint violation")]
    ConstraintViolation,
    /// Timestamp outside permissible bounds.
    #[msg("Timestamp outside expected bounds")]
    TimestampInvalid,
    /// Provided Merkle proof invalid.
    #[msg("Invalid Merkle proof")]
    InvalidMerkleProof,
    /// Overflow in math operations.
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    /// Account already initialized.
    #[msg("Account already initialized")]
    AlreadyInitialized,
    /// Unauthorized actor attempted action.
    #[msg("Unauthorized")]
    Unauthorized,
}
