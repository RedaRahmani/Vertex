//! Shared event definitions used across Keystone programs.
//!
//! This module is compiled with `#![allow(missing_docs)]` because the Anchor
//! `#[event]` macro generates helper items that otherwise trigger the
//! crate-level `#![deny(missing_docs)]` in `common/src/lib.rs`. We still keep
//! human-readable docs on the public structs/fields below for clarity.

#![allow(missing_docs)]

use anchor_lang::prelude::*;

/// Emitted when liquidity is added to an AMM pool.
#[event]
pub struct LiquidityAdded {
    /// Pool account public key.
    pub pool: Pubkey,
    /// User who provided the liquidity.
    pub user: Pubkey,
    /// Amount of token A deposited.
    pub amount_a: u64,
    /// Amount of token B deposited.
    pub amount_b: u64,
    /// Amount of LP tokens minted to the user.
    pub lp_minted: u64,
}

/// Emitted when liquidity is removed from an AMM pool.
#[event]
pub struct LiquidityRemoved {
    /// Pool account public key.
    pub pool: Pubkey,
    /// User who removed the liquidity.
    pub user: Pubkey,
    /// Amount of LP tokens burned.
    pub lp_burned: u64,
    /// Amount of token A withdrawn.
    pub amount_a: u64,
    /// Amount of token B withdrawn.
    pub amount_b: u64,
}

/// Emitted when a swap is executed on an AMM pool.
#[event]
pub struct SwapExecuted {
    /// Pool account public key.
    pub entity: Pubkey,
    /// User who performed the swap.
    pub user: Pubkey,
    /// Amount of the input token sent by the user.
    pub amount_in: u64,
    /// Amount of the output token received by the user.
    pub amount_out: u64,
}

/// Emitted whenever protocol/treasury tokens are moved by a program.
#[event]
pub struct TreasuryMovement {
    /// Program ID that emitted the movement (e.g., AMM, Launchpad).
    pub program: Pubkey,
    /// Entity account related to the transfer (pool, config, etc.).
    pub entity: Pubkey,
    /// Amount transferred.
    pub amount: u64,
    /// Destination token account.
    pub destination: Pubkey,
}

/// Emitted when a launch configuration has been updated.
#[event]
pub struct ConfigUpdated {
    /// The entity/config public key.
    pub entity: Pubkey,
    /// Slot at which the update occurred.
    pub slot: u64,
    /// Keccak-256 hash of the effective config values.
    pub config_hash: [u8; 32],
}

/// Emitted when a user stakes tokens into a staking pool (or fully unstakes
/// when `amount` is 0 per current implementation).
#[event]
pub struct StakeEvent {
    /// Staking pool public key.
    pub pool: Pubkey,
    /// Staker wallet public key.
    pub staker: Pubkey,
    /// Amount staked (or 0 on full unstake).
    pub amount: u64,
}

/// Emitted when staking rewards are claimed.
#[event]
pub struct ClaimEvent {
    /// Staking pool public key.
    pub pool: Pubkey,
    /// Staker wallet public key.
    pub staker: Pubkey,
    /// Reward amount transferred to the staker.
    pub reward: u64,
}

/// Emitted for vesting actions: claims or revocations.
#[event]
pub struct VestingEvent {
    /// Vesting schedule account public key.
    pub schedule: Pubkey,
    /// The beneficiary (on claim) or the authority (on revoke).
    pub beneficiary_or_authority: Pubkey,
    /// Amount transferred as part of the action.
    pub amount: u64,
    /// `true` if this event represents a claim; `false` for revoke/refund.
    pub is_claim: bool,
}
