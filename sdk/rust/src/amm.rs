//! AMM instruction builders.

use anchor_lang::InstructionData;
use keystone_amm_cp::{accounts, instruction};
use solana_program::{instruction::Instruction, pubkey::Pubkey};

/// Init pool instruction helper.
pub fn init_pool(
    program_id: Pubkey,
    accounts: accounts::InitPool,
    fee_numerator: u64,
    fee_denominator: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::InitPool {
            fee_numerator,
            fee_denominator,
        }
        .data(),
    }
}

/// Swap instruction helper.
pub fn swap(
    program_id: Pubkey,
    accounts: accounts::Swap,
    amount_in: u64,
    minimum_out: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::Swap {
            amount_in,
            minimum_out,
        }
        .data(),
    }
}
