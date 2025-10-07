#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(clippy::all)]
#![warn(missing_docs)]
//! Keystone common utilities shared across on-chain programs.

pub mod authority;
pub mod bpf_sort;
pub mod curve;
pub mod decimals;
pub mod errors;
pub mod events;
pub mod fees;
pub mod merkle;
pub mod time;

pub use anchor_lang::prelude::*;

// Provide a tiny IDL stub for this utility crate so `anchor idl build`
// doesn't error when traversing all crates under `programs/`.
#[cfg(feature = "idl-build")]
use anchor_lang::{declare_id, program};

#[cfg(feature = "idl-build")]
declare_id!("11111111111111111111111111111111");

#[cfg(feature = "idl-build")]
#[program]
pub mod __idl_stub {}
