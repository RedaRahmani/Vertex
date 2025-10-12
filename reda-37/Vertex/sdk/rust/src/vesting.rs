//! Vesting instruction builders.

use anchor_lang::InstructionData;
use keystone_vesting::{accounts, instruction};
use solana_program::{instruction::Instruction, pubkey::Pubkey};

/// Create schedule helper.
pub fn create_schedule(
    program_id: Pubkey,
    accounts: accounts::CreateSchedule,
    args: instruction::CreateArgs,
) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::CreateSchedule { args }.data(),
    }
}

/// Claim vested tokens.
pub fn claim(
    program_id: Pubkey,
    accounts: accounts::Claim,
    amount: u64,
    proof: Option<Vec<[u8; 32]>>,
) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::Claim { amount, proof }.data(),
    }
}

/// Revoke schedule.
pub fn revoke(program_id: Pubkey, accounts: accounts::Revoke) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::Revoke {}.data(),
    }
}
