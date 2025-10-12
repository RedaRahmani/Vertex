//! Time utilities for enforcing schedule constraints.

use anchor_lang::prelude::*;

/// Guard ensuring current unix timestamp is within inclusive range.
pub fn assert_within_window(clock: &Clock, start: i64, end: i64) -> Result<()> {
    let now = clock.unix_timestamp;
    require!(now >= start, crate::errors::CommonError::TimestampInvalid);
    require!(now <= end, crate::errors::CommonError::TimestampInvalid);
    Ok(())
}

/// Checks the sale has not expired yet.
pub fn assert_not_expired(clock: &Clock, end: i64) -> Result<()> {
    let now = clock.unix_timestamp;
    require!(now <= end, crate::errors::CommonError::TimestampInvalid);
    Ok(())
}

/// Ensures the sale has started.
pub fn assert_started(clock: &Clock, start: i64) -> Result<()> {
    let now = clock.unix_timestamp;
    require!(now >= start, crate::errors::CommonError::TimestampInvalid);
    Ok(())
}
