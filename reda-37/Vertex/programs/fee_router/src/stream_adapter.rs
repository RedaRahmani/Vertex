//! Pluggable adapter for reading Streamflow "still-locked" amounts.
//! - Default mock feature: reads first 8 bytes LE as u64.
//! - `streamflow` feature: outline real layout parsing (skeleton, unimplemented).

use anchor_lang::prelude::*;

/// Trait for reading still-locked amount from a Streamflow stream account.
pub trait StreamLockedReader {
    /// Returns the still-locked amount in quote lamports for the provided stream account.
    fn locked_amount(stream_ai: &AccountInfo) -> anchor_lang::Result<u64>;
}

#[cfg(not(feature = "streamflow"))]
impl StreamLockedReader for () {
    fn locked_amount(stream_ai: &AccountInfo) -> anchor_lang::Result<u64> {
        let data = stream_ai.data.borrow();
        if data.len() < 8 {
            return Ok(0);
        }
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&data[..8]);
        Ok(u64::from_le_bytes(arr))
    }
}

#[cfg(feature = "streamflow")]
impl StreamLockedReader for () {
    fn locked_amount(stream_ai: &AccountInfo) -> anchor_lang::Result<u64> {
        use bytemuck::{Pod, Zeroable};

        #[repr(C)]
        #[derive(Clone, Copy, Pod, Zeroable)]
        struct StreamflowHeaderMock {
            #[allow(dead_code)]
            still_locked: u64,
        }

        let header = {
            let data = stream_ai.data.borrow();
            let span = data
                .get(..core::mem::size_of::<StreamflowHeaderMock>())
                .ok_or_else(|| error!(crate::FeeRouterError::ConstraintViolation))?;
            *bytemuck::try_from_bytes::<StreamflowHeaderMock>(span)
                .map_err(|_| error!(crate::FeeRouterError::ConstraintViolation))?
        };

        let _ = header;
        Err(error!(crate::FeeRouterError::ConstraintViolation))
    }
}
