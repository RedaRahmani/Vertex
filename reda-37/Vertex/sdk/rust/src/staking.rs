//! Staking instruction builders.

use anchor_lang::InstructionData;
use keystone_staking::{accounts, instruction};
use solana_program::{instruction::Instruction, pubkey::Pubkey};

/// Initialize staking pool instruction helper.
pub fn init_pool(program_id: Pubkey, accounts: accounts::InitPool, args: instruction::InitArgs) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::InitPool { args }.data(),
    }
}

/// Stake tokens instruction helper.
pub fn stake(program_id: Pubkey, accounts: accounts::Stake, amount: u64) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::Stake { amount }.data(),
    }
}

/// Claim rewards helper.
pub fn claim(program_id: Pubkey, accounts: accounts::Claim) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::Claim {}.data(),
    }
}
