//! Launchpad instruction builders.

use anchor_lang::InstructionData;
use keystone_launchpad::{accounts, instruction};
use solana_program::{instruction::Instruction, pubkey::Pubkey};

/// Builds `init_launch` instruction.
pub fn init_launch(
    program_id: Pubkey,
    accounts: accounts::InitLaunch,
    args: instruction::InitLaunchArgs,
) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::InitLaunch { args }.data(),
    }
}

/// Builds `buy` instruction frame.
pub fn buy(
    program_id: Pubkey,
    accounts: accounts::Buy,
    amount: u64,
    proof: Option<Vec<[u8; 32]>>,
    max_quote: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::Buy {
            amount,
            proof,
            max_quote,
        }
        .data(),
    }
}

/// Builds `bid` instruction for auctions.
pub fn bid(
    program_id: Pubkey,
    accounts: accounts::Bid,
    amount: u64,
    proof: Option<Vec<[u8; 32]>>,
) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::Bid { amount, proof }.data(),
    }
}

/// Builds auction settlement instruction.
pub fn settle(
    program_id: Pubkey,
    accounts: accounts::SettleAuction,
) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction::SettleAuction {}.data(),
    }
}
