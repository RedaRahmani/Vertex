#![cfg(feature = "bankrun-test")]

use std::str::FromStr;

use anchor_lang::{prelude::*, InstructionData, ToAccountMetas};
use anchor_lang::solana_program::{self, system_program, sysvar};
use crate::{
    CrankArgs,
    InitPolicyArgs,
    FEE_POS_OWNER_SEED,
    POLICY_SEED,
    POSITION_SEED,
    PROGRESS_SEED,
    VAULT_SEED,
};
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account as SolanaAccount,
    instruction::Instruction,
    signature::{Keypair, Signature, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::{instruction as token_instruction, state::{Account as TokenAccount, Mint}};

const DLMM_ID: &str = "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG";

#[tokio::test]
async fn host_integration_flow() {
    let mut test = ProgramTest::new(
        "keystone_fee_router",
        crate::id(),
        processor!(crate::entry),
    );
    test.prefer_builtins = true;

    let mut context = test.start_with_context().await;

    let authority = Keypair::new();
    let creator = Keypair::new();
    let investor = Keypair::new();
    let cp_pool = Keypair::new();
    let cp_position = Keypair::new();
    let quote_mint = Keypair::new();
    let stream_locked = Keypair::new();
    let stream_empty = Keypair::new();

    fund_accounts(
        &mut context,
        &[authority.pubkey(), creator.pubkey(), investor.pubkey(), cp_pool.pubkey(), cp_position.pubkey()],
    )
    .await;

    let rent = context.banks_client.get_rent().await.unwrap();
    let dlmm_program = Pubkey::from_str(DLMM_ID).unwrap();

    create_unchecked_account(
        &mut context,
        &cp_pool,
        &dlmm_program,
        rent.minimum_balance(0),
        0,
    )
    .await;
    create_unchecked_account(
        &mut context,
        &cp_position,
        &dlmm_program,
        rent.minimum_balance(0),
        0,
    )
    .await;

    create_mint(&mut context, &quote_mint, &authority, rent.minimum_balance(Mint::LEN)).await;

    let (policy_pda, _) = Pubkey::find_program_address(
        &[crate::POLICY_SEED, cp_pool.pubkey().as_ref()],
        &crate::id(),
    );
    let (vault_authority, _) = Pubkey::find_program_address(
        &[crate::VAULT_SEED, policy_pda.as_ref()],
        &crate::id(),
    );
    let (progress_pda, _) = Pubkey::find_program_address(
        &[crate::PROGRESS_SEED, cp_pool.pubkey().as_ref()],
        &crate::id(),
    );
    let (position_pda, _) = Pubkey::find_program_address(
        &[crate::POSITION_SEED, policy_pda.as_ref()],
        &crate::id(),
    );
    let (owner_pda, _) = Pubkey::find_program_address(
        &[
            crate::VAULT_SEED,
            policy_pda.as_ref(),
            crate::FEE_POS_OWNER_SEED,
        ],
        &crate::id(),
    );

    let treasury_ata = spl_associated_token_account::get_associated_token_address(&vault_authority, &quote_mint.pubkey());
    let creator_ata = spl_associated_token_account::get_associated_token_address(&creator.pubkey(), &quote_mint.pubkey());
    let investor_ata = spl_associated_token_account::get_associated_token_address(&investor.pubkey(), &quote_mint.pubkey());

    let ata_ixs = vec![
        create_associated_token_account(&context.payer.pubkey(), &vault_authority, &quote_mint.pubkey(), &spl_token::ID),
        create_associated_token_account(&context.payer.pubkey(), &creator.pubkey(), &quote_mint.pubkey(), &spl_token::ID),
        create_associated_token_account(&context.payer.pubkey(), &investor.pubkey(), &quote_mint.pubkey(), &spl_token::ID),
    ];
    process_tx(&mut context, ata_ixs, &[&context.payer]).await;

    init_stream_account(&mut context, &stream_locked, 200_000, rent.minimum_balance(8)).await;
    init_stream_account(&mut context, &stream_empty, 0, rent.minimum_balance(8)).await;

    let init_policy_ix = Instruction {
        program_id: crate::id(),
        accounts: crate::accounts::InitPolicy {
            authority: authority.pubkey(),
            policy: policy_pda,
            cp_pool: cp_pool.pubkey(),
            quote_mint: quote_mint.pubkey(),
            creator_quote_ata: creator_ata,
            treasury_quote_ata: treasury_ata,
            vault_authority,
            token_program: spl_token::ID,
            system_program: system_program::ID,
            rent: sysvar::rent::ID,
        }
        .to_account_metas(None),
        data: crate::instruction::InitPolicy {
            args: InitPolicyArgs {
                y0_total: 1_000_000,
                investor_fee_share_bps: 2_000,
                daily_cap_quote: 0,
                min_payout_lamports: 100,
            },
        }
        .data(),
    };
    process_tx(&mut context, vec![init_policy_ix], &[&context.payer, &authority]).await;

    let init_position_ix = Instruction {
        program_id: crate::id(),
        accounts: crate::accounts::InitHonoraryPosition {
            authority: authority.pubkey(),
            policy: policy_pda,
            cp_pool: cp_pool.pubkey(),
            quote_mint: quote_mint.pubkey(),
            owner_pda,
            cp_position: cp_position.pubkey(),
            honorary_position: position_pda,
            system_program: system_program::ID,
            rent: sysvar::rent::ID,
        }
        .to_account_metas(None),
        data: crate::instruction::InitHonoraryPosition {}.data(),
    };
    process_tx(&mut context, vec![init_position_ix], &[&context.payer, &authority]).await;

    let mint_ix = token_instruction::mint_to(
        &spl_token::ID,
        &quote_mint.pubkey(),
        &treasury_ata,
        &authority.pubkey(),
        &[],
        5_000,
    )
    .unwrap();
    process_tx(&mut context, vec![mint_ix], &[&context.payer, &authority]).await;

    let first_crank_ix = Instruction {
        program_id: crate::id(),
        accounts: crate::accounts::CrankDistribute {
            cp_program: Pubkey::from_str(DLMM_ID).unwrap(),
            cp_pool: cp_pool.pubkey(),
            policy: policy_pda,
            progress: progress_pda,
            payer: authority.pubkey(),
            vault_authority,
            treasury_quote_ata: treasury_ata,
            creator_quote_ata: creator_ata,
            investor_quote_ata: investor_ata,
            stream: stream_locked.pubkey(),
            token_program: spl_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: crate::instruction::CrankDistribute {
            args: CrankArgs {
                page_cursor: 1,
                is_last_page: false,
            },
        }
        .data(),
    };
    let first_sig = process_tx(&mut context, vec![first_crank_ix], &[&context.payer, &authority]).await;
    assert_event(&mut context, first_sig, "InvestorPayoutPage").await;

    let second_crank_ix = Instruction {
        program_id: crate::id(),
        accounts: crate::accounts::CrankDistribute {
            cp_program: Pubkey::from_str(DLMM_ID).unwrap(),
            cp_pool: cp_pool.pubkey(),
            policy: policy_pda,
            progress: progress_pda,
            payer: authority.pubkey(),
            vault_authority,
            treasury_quote_ata: treasury_ata,
            creator_quote_ata: creator_ata,
            investor_quote_ata: investor_ata,
            stream: stream_empty.pubkey(),
            token_program: spl_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: crate::instruction::CrankDistribute {
            args: CrankArgs {
                page_cursor: 2,
                is_last_page: true,
            },
        }
        .data(),
    };
    let second_sig = process_tx(&mut context, vec![second_crank_ix], &[&context.payer, &authority]).await;
    assert_event(&mut context, second_sig, "CreatorPayoutDayClosed").await;

    let progress_account = context
        .banks_client
        .get_account(progress_pda)
        .await
        .unwrap()
        .expect("progress account");
    let mut data: &[u8] = &progress_account.data;
    let mut data_slice: &[u8] = &progress_account.data;
    let progress_state = crate::Progress::try_deserialize(&mut data_slice).unwrap();
    assert_eq!(progress_state.distributed_quote_today, 1_000);
    assert_eq!(progress_state.carry_quote_today, 0);
    assert!(progress_state.day_closed);

    let creator_acc = context
        .banks_client
        .get_account(creator_ata)
        .await
        .unwrap()
        .expect("creator ata");
    let token_state = TokenAccount::unpack(&creator_acc.data).unwrap();
    assert_eq!(token_state.amount, 4_000);
}

async fn fund_accounts(context: &mut ProgramTestContext, recipients: &[Pubkey]) {
    let mut ixs = Vec::with_capacity(recipients.len());
    for recipient in recipients {
        ixs.push(system_instruction::transfer(
            &context.payer.pubkey(),
            recipient,
            5_000_000_000,
        ));
    }
    process_tx(context, ixs, &[&context.payer]).await;
}

async fn create_unchecked_account(
    context: &mut ProgramTestContext,
    keypair: &Keypair,
    owner: &Pubkey,
    lamports: u64,
    space: u64,
) {
    let ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &keypair.pubkey(),
        lamports,
        space,
        owner,
    );
    process_tx(context, vec![ix], &[&context.payer, keypair]).await;
}

async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    authority: &Keypair,
    lamports: u64,
) {
    let create_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &mint.pubkey(),
        lamports,
        Mint::LEN as u64,
        &spl_token::ID,
    );
    let init_ix = token_instruction::initialize_mint(
        &spl_token::ID,
        &mint.pubkey(),
        &authority.pubkey(),
        None,
        6,
    )
    .unwrap();
    process_tx(context, vec![create_ix, init_ix], &[&context.payer, mint]).await;
}

async fn init_stream_account(
    context: &mut ProgramTestContext,
    stream: &Keypair,
    locked_amount: u64,
    lamports: u64,
) {
    let ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &stream.pubkey(),
        lamports,
        8,
        &crate::id(),
    );
    process_tx(context, vec![ix], &[&context.payer, stream]).await;

    let mut data = locked_amount.to_le_bytes().to_vec();
    let account = SolanaAccount {
        lamports,
        data,
        owner: crate::id(),
        executable: false,
        rent_epoch: 0,
    };
    context.set_account(&stream.pubkey(), &account);
}

async fn assert_event(context: &mut ProgramTestContext, signature: Signature, needle: &str) {
    let tx = context
        .banks_client
        .get_transaction_with_status_meta(signature)
        .await
        .unwrap()
        .expect("transaction meta");
    let meta = tx.meta.expect("meta");
    let logs = meta.log_messages.expect("logs");
    assert!(
        logs.iter().any(|line| line.contains(needle)),
        "missing event {needle} in logs"
    );
}

async fn process_tx(
    context: &mut ProgramTestContext,
    instructions: Vec<Instruction>,
    signers: &[&Keypair],
) -> Signature {
    let mut all_signers: Vec<&Keypair> = vec![&context.payer];
    all_signers.extend_from_slice(signers);
    let bh = context.banks_client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&context.payer.pubkey()),
        &all_signers,
        bh,
    );
    context.banks_client.process_transaction(tx.clone()).await.unwrap();
    tx.signatures[0]
}
