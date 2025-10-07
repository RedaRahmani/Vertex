#![allow(clippy::result_large_err)]
#![warn(missing_docs)]
//! Keystone vesting program supporting linear & cliff schedules.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use keystone_common::errors::CommonError;
use keystone_common::merkle;
#[cfg(not(target_arch = "bpf"))]
use solana_security_txt::security_txt;

#[cfg(not(target_arch = "bpf"))]
security_txt! {
    name: "Keystone Vesting",
    project_url: "https://github.com/keystone-labs/keystone-vertex",
    contacts: "email:security@keystonelabs.xyz",
    policy: "https://github.com/keystone-labs/keystone-vertex/security/policy",
    preferred_languages: "en",
    source_code: "https://github.com/keystone-labs/keystone-vertex"
}

declare_id!("C3HSSPsKxkFeLvZr6jNujoA1Z1bHH4VzC3dK5HSMTpvq");

/// Program handlers.
#[program]
pub mod keystone_vesting {
    use super::*;

    /// Creates vesting schedule.
    pub fn create_schedule(ctx: Context<CreateSchedule>, args: CreateArgs) -> Result<()> {
        let schedule = &mut ctx.accounts.schedule;
        require!(!schedule.initialized, VestingError::AlreadyInitialized);
        require!(args.total > 0, CommonError::ConstraintViolation);
        require!(args.end > args.start, CommonError::TimestampInvalid);
        require!(args.cliff >= args.start, CommonError::TimestampInvalid);
        schedule.authority = ctx.accounts.authority.key();
        schedule.beneficiary = args.beneficiary;
        schedule.start = args.start;
        schedule.cliff = args.cliff;
        schedule.end = args.end;
        schedule.total = args.total;
        schedule.claimed = 0;
        schedule.revocable = args.revocable;
        schedule.merkle_root = args.merkle_root;
        schedule.bump = ctx.bumps.vault_authority;
        schedule.initialized = true;
        Ok(())
    }

    /// Claims vested tokens for beneficiary.
    pub fn claim(ctx: Context<Claim>, amount: u64, proof: Option<Vec<[u8; 32]>>) -> Result<()> {
        let clock = Clock::get()?;
        let schedule = &mut ctx.accounts.schedule;
        require!(
            schedule.beneficiary == ctx.accounts.beneficiary.key(),
            CommonError::Unauthorized
        );
        if let Some(root) = schedule.merkle_root {
            let leaf = merkle_leaf(ctx.accounts.beneficiary.key(), schedule.total);
            let proof_vec = proof.ok_or(VestingError::MerkleRequired)?;
            merkle::assert_merkle_proof(leaf, &proof_vec, root)?;
        }
        let vested = schedule.vested_amount(clock.unix_timestamp)?;
        let available = vested
            .checked_sub(schedule.claimed)
            .ok_or(CommonError::ArithmeticOverflow)?;
        require!(amount <= available, VestingError::AmountTooLarge);

        schedule.claimed = schedule
            .claimed
            .checked_add(amount)
            .ok_or(CommonError::ArithmeticOverflow)?;

        let sched_key = schedule.key();
        let seeds: &[&[u8]] = &[b"vault", sched_key.as_ref(), &[schedule.bump]];
        let binding = [seeds];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &binding,
        );
        token::transfer(cpi_ctx, amount)?;
        emit!(keystone_common::events::VestingEvent {
            schedule: ctx.accounts.schedule.key(),
            beneficiary_or_authority: ctx.accounts.beneficiary.key(),
            amount,
            is_claim: true,
        });
        Ok(())
    }

    /// Revokes vesting schedule transferring remaining tokens.
    pub fn revoke(ctx: Context<Revoke>) -> Result<()> {
        let schedule = &mut ctx.accounts.schedule;
        require!(
            schedule.authority == ctx.accounts.authority.key(),
            CommonError::Unauthorized
        );
        require!(schedule.revocable, VestingError::NotRevocable);
        let remaining = schedule
            .total
            .checked_sub(schedule.claimed)
            .ok_or(CommonError::ArithmeticOverflow)?;
        let sched_key = schedule.key();
        let seeds: &[&[u8]] = &[b"vault", sched_key.as_ref(), &[schedule.bump]];
        let binding = [seeds];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.refund_destination.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &binding,
        );
        token::transfer(cpi_ctx, remaining)?;
        schedule.total = schedule.claimed;
        emit!(keystone_common::events::VestingEvent {
            schedule: ctx.accounts.schedule.key(),
            beneficiary_or_authority: ctx.accounts.authority.key(),
            amount: remaining,
            is_claim: false,
        });
        Ok(())
    }
}

/// Arguments for creating schedules.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CreateArgs {
    /// Beneficiary account.
    pub beneficiary: Pubkey,
    /// Schedule start.
    pub start: i64,
    /// Cliff timestamp.
    pub cliff: i64,
    /// End timestamp.
    pub end: i64,
    /// Total vested amount.
    pub total: u64,
    /// Schedule revocable.
    pub revocable: bool,
    /// Optional merkle root for batch claims.
    pub merkle_root: Option<[u8; 32]>,
}

/// Vesting schedule state.
#[account]
pub struct VestingSchedule {
    /// Authority owning schedule.
    pub authority: Pubkey,
    /// Beneficiary wallet.
    pub beneficiary: Pubkey,
    /// Start timestamp.
    pub start: i64,
    /// Cliff timestamp.
    pub cliff: i64,
    /// End timestamp.
    pub end: i64,
    /// Total amount to vest.
    pub total: u64,
    /// Amount already claimed.
    pub claimed: u64,
    /// Flag for revocation.
    pub revocable: bool,
    /// Optional merkle root for delegated claims.
    pub merkle_root: Option<[u8; 32]>,
    /// PDA bump.
    pub bump: u8,
    /// Initialization flag.
    pub initialized: bool,
}

impl VestingSchedule {
    /// Size of the `VestingSchedule` account data (without discriminator).
    pub const LEN: usize = core::mem::size_of::<Self>();
    /// Allocated bytes including Anchor discriminator.
    pub const SPACE: usize = 8 + Self::LEN;
    /// Calculates vested amount based on timestamp.
    pub fn vested_amount(&self, timestamp: i64) -> Result<u64> {
        if timestamp <= self.cliff {
            return Ok(0);
        }
        if timestamp >= self.end {
            return Ok(self.total);
        }
        let elapsed = (timestamp - self.start) as u128;
        let duration = (self.end - self.start) as u128;
        let vested = (self.total as u128)
            .checked_mul(elapsed)
            .ok_or(CommonError::ArithmeticOverflow)?
            .checked_div(duration)
            .ok_or(CommonError::ArithmeticOverflow)?;
        Ok(vested as u64)
    }
}

fn merkle_leaf(beneficiary: Pubkey, amount: u64) -> [u8; 32] {
    let mut data = Vec::with_capacity(40);
    data.extend_from_slice(beneficiary.as_ref());
    data.extend_from_slice(&amount.to_le_bytes());
    anchor_lang::solana_program::keccak::hash(&data).to_bytes()
}

/// Accounts for creating schedule.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct CreateSchedule<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = VestingSchedule::SPACE,
        seeds = [b"schedule", beneficiary.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub schedule: Account<'info, VestingSchedule>,
    /// CHECK: Unchecked beneficiary for seeds.
    pub beneficiary: UncheckedAccount<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut, constraint = vault.mint == mint.key())]
    pub vault: Account<'info, TokenAccount>,
    /// CHECK: Derived vault authority PDA.
    #[account(
        seeds = [b"vault", schedule.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

/// Claim context.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(mut)]
    pub schedule: Account<'info, VestingSchedule>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut, constraint = destination.owner == beneficiary.key())]
    pub destination: Account<'info, TokenAccount>,
    /// CHECK: Vault authority PDA.
    #[account(
        seeds = [b"vault", schedule.key().as_ref()],
        bump = schedule.bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

/// Revocation context.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct Revoke<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub schedule: Account<'info, VestingSchedule>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut, constraint = refund_destination.owner == authority.key())]
    pub refund_destination: Account<'info, TokenAccount>,
    /// CHECK: Vault authority PDA.
    #[account(
        seeds = [b"vault", schedule.key().as_ref()],
        bump = schedule.bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

/// Vesting specific errors.
#[error_code]
pub enum VestingError {
    /// Schedule already initialized.
    #[msg("Schedule already initialized")]
    AlreadyInitialized,
    /// Claim amount too large.
    #[msg("Claim exceeds vested amount")]
    AmountTooLarge,
    /// Schedule not revocable.
    #[msg("Schedule is not revocable")]
    NotRevocable,
    /// Merkle proof required for claim.
    #[msg("Merkle proof required")]
    MerkleRequired,
}

#[cfg(all(test, not(target_arch = "bpf")))]
mod tests {
    use super::*;

    #[test]
    fn vested_amount_monotonic() {
        let schedule = VestingSchedule {
            authority: Pubkey::new_unique(),
            beneficiary: Pubkey::new_unique(),
            start: 0,
            cliff: 5,
            end: 10,
            total: 1_000,
            claimed: 0,
            revocable: true,
            merkle_root: None,
            bump: 0,
            initialized: true,
        };
        assert_eq!(schedule.vested_amount(0).unwrap(), 0);
        assert!(schedule.vested_amount(10).unwrap() >= schedule.vested_amount(6).unwrap());
    }
}
