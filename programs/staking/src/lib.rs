#![allow(clippy::result_large_err)]
#![warn(missing_docs)]
//! Keystone staking program supporting emissions and lockups.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use keystone_common::errors::CommonError;
#[cfg(not(target_arch = "bpf"))]
use solana_security_txt::security_txt;

#[cfg(not(target_arch = "bpf"))]
security_txt! {
    name: "Keystone Staking",
    project_url: "https://github.com/keystone-labs/keystone-vertex",
    contacts: "email:security@keystonelabs.xyz",
    policy: "https://github.com/keystone-labs/keystone-vertex/security/policy",
    preferred_languages: "en",
    source_code: "https://github.com/keystone-labs/keystone-vertex"
}

declare_id!("2cFCPc4MCUqwd5FTopBprTyr3pwAsXDNcxjgPhGKRdo5");

/// Program instructions.
#[allow(missing_docs)]
#[program]
pub mod keystone_staking {
    use super::*;

    /// Initialize staking pool.
    pub fn init_pool(ctx: Context<InitPool>, args: InitArgs) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(!pool.initialized, StakingError::AlreadyInitialized);

        pool.authority = ctx.accounts.authority.key();
        pool.reward_mint = ctx.accounts.reward_mint.key();
        pool.lock_policy = args.lock_policy;
        pool.emission_rate = args.emission_rate;
        pool.bump = ctx.bumps.vault_authority;
        pool.initialized = true;

        Ok(())
    }

    /// Stake tokens transferring into pool vault.
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        require!(amount > 0, CommonError::ConstraintViolation);
        let pool = &ctx.accounts.pool;
        require!(pool.initialized, StakingError::Uninitialized);

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.staker_token.to_account_info(),
                    to: ctx.accounts.stake_vault.to_account_info(),
                    authority: ctx.accounts.staker.to_account_info(),
                },
            ),
            amount,
        )?;

        let user = &mut ctx.accounts.user;
        user.owner = ctx.accounts.staker.key();
        user.amount_staked = user
            .amount_staked
            .checked_add(amount)
            .ok_or(CommonError::ArithmeticOverflow)?;
        user.last_claim_slot = Clock::get()?.slot;

        emit!(keystone_common::events::StakeEvent {
            pool: ctx.accounts.pool.key(),
            staker: ctx.accounts.staker.key(),
            amount,
        });

        Ok(())
    }

    /// Claim staking rewards based on elapsed slots.
    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let pool = &ctx.accounts.pool;
        let user = &mut ctx.accounts.user;

        require!(
            user.owner == ctx.accounts.staker.key(),
            CommonError::Unauthorized
        );

        let current_slot = Clock::get()?.slot;
        require!(
            current_slot > user.last_claim_slot,
            StakingError::NothingToClaim
        );

        let slots_elapsed = current_slot - user.last_claim_slot;
        let reward = user
            .amount_staked
            .checked_mul(pool.emission_rate)
            .ok_or(CommonError::ArithmeticOverflow)?
            .checked_mul(slots_elapsed as u64)
            .ok_or(CommonError::ArithmeticOverflow)?;
        require!(reward > 0, StakingError::NothingToClaim);

        let pool_key = pool.key();
        let seeds: &[&[u8]] = &[b"vault", pool_key.as_ref(), &[pool.bump]];
        let signer = [seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.reward_vault.to_account_info(),
                to: ctx.accounts.staker_reward_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &signer,
        );
        token::transfer(cpi_ctx, reward)?;

        user.last_claim_slot = current_slot;

        emit!(keystone_common::events::ClaimEvent {
            pool: ctx.accounts.pool.key(),
            staker: ctx.accounts.staker.key(),
            reward,
        });

        Ok(())
    }

    /// Unstake tokens obeying lock policy.
    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        let pool = &ctx.accounts.pool;
        let user = &mut ctx.accounts.user;

        require!(
            user.owner == ctx.accounts.staker.key(),
            CommonError::Unauthorized
        );
        enforce_lock(pool, user)?;

        let amount = user.amount_staked;
        let pool_key = pool.key();
        let seeds: &[&[u8]] = &[b"vault", pool_key.as_ref(), &[pool.bump]];
        let signer = [seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.stake_vault.to_account_info(),
                to: ctx.accounts.staker_token.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &signer,
        );
        token::transfer(cpi_ctx, amount)?;

        user.amount_staked = 0;

        emit!(keystone_common::events::StakeEvent {
            pool: ctx.accounts.pool.key(),
            staker: ctx.accounts.staker.key(),
            amount: 0,
        });

        Ok(())
    }

    /// Admin update for emission rate and policy.
    pub fn admin_update(ctx: Context<AdminUpdate>, args: UpdateArgs) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(
            pool.authority == ctx.accounts.authority.key(),
            CommonError::Unauthorized
        );

        if let Some(rate) = args.emission_rate {
            pool.emission_rate = rate;
        }
        if let Some(policy) = args.lock_policy {
            pool.lock_policy = policy;
        }
        Ok(())
    }
}

fn enforce_lock(pool: &Pool, user: &UserStake) -> Result<()> {
    match pool.lock_policy {
        LockPolicy::None => Ok(()),
        LockPolicy::Linear {
            start_slot,
            end_slot,
        } => {
            let current = Clock::get()?.slot;
            require!(current >= start_slot, StakingError::StillLocked);
            if current < end_slot {
                require!(
                    user.last_claim_slot >= start_slot,
                    StakingError::StillLocked
                );
            }
            Ok(())
        }
        LockPolicy::Cliff { release_slot } => {
            let current = Clock::get()?.slot;
            require!(current >= release_slot, StakingError::StillLocked);
            Ok(())
        }
    }
}

/// Initialization arguments for pool.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitArgs {
    /// Emission per slot per token.
    pub emission_rate: u64,
    /// Lock policy for unstaking.
    pub lock_policy: LockPolicy,
}

/// Admin update arguments.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct UpdateArgs {
    /// New emission rate.
    pub emission_rate: Option<u64>,
    /// New lock policy.
    pub lock_policy: Option<LockPolicy>,
}

/// Lock policies supported.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum LockPolicy {
    /// No lockup.
    None,
    /// Linear release between slots.
    Linear { start_slot: u64, end_slot: u64 },
    /// Cliff release at slot.
    Cliff { release_slot: u64 },
}

/// Pool configuration account.
#[account]
pub struct Pool {
    /// Pool authority.
    pub authority: Pubkey,
    /// Reward mint.
    pub reward_mint: Pubkey,
    /// Lock policy.
    pub lock_policy: LockPolicy,
    /// Emission rate per slot.
    pub emission_rate: u64,
    /// Bump for vault authority PDA.
    pub bump: u8,
    /// Initialized flag.
    pub initialized: bool,
}

impl Pool {
    /// Size of the Pool account data (without discriminator).
    pub const LEN: usize = core::mem::size_of::<Self>();
    /// Allocated bytes including Anchor discriminator.
    pub const SPACE: usize = 8 + Self::LEN;
}

/// User stake account (zero copy for compactness).
#[account]
#[repr(C)]
pub struct UserStake {
    /// Owner wallet.
    pub owner: Pubkey,
    /// Total staked amount.
    pub amount_staked: u64,
    /// Slot last claimed.
    pub last_claim_slot: u64,
}

impl Default for UserStake {
    fn default() -> Self {
        Self {
            owner: Pubkey::default(),
            amount_staked: 0,
            last_claim_slot: 0,
        }
    }
}

/// Accounts for initializing the pool.
/// NOTE: `vault_authority` must appear *before* any constraints that reference it.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct InitPool<'info> {
    /// Pool authority who manages the pool.
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = Pool::SPACE,
        seeds = [b"pool", reward_mint.key().as_ref(), authority.key().as_ref()],
        bump
    )]
    /// Pool account storing configuration.
    pub pool: Account<'info, Pool>,

    /// Derived vault authority PDA.
    /// CHECK: PDA used as authority for vaults.
    #[account(
        seeds = [b"vault", pool.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    /// Token account holding staked tokens. Must be owned by vault authority.
    #[account(mut, constraint = stake_vault.owner == vault_authority.key())]
    pub stake_vault: Account<'info, TokenAccount>,

    /// Token account holding reward tokens. Must be owned by vault authority.
    #[account(mut, constraint = reward_vault.owner == vault_authority.key())]
    pub reward_vault: Account<'info, TokenAccount>,

    /// Mint of the reward token.
    pub reward_mint: Account<'info, Mint>,

    /// SPL token program.
    pub token_program: Program<'info, Token>,
    /// System program.
    pub system_program: Program<'info, System>,
    /// Rent sysvar.
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts for staking.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct Stake<'info> {
    /// Staker signing the transaction.
    #[account(mut)]
    pub staker: Signer<'info>,

    /// Source token account of the staker.
    #[account(mut, constraint = staker_token.owner == staker.key())]
    pub staker_token: Account<'info, TokenAccount>,

    /// Pool configuration.
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Derived vault authority.
    /// CHECK: PDA authority for the stake_vault.
    #[account(
        seeds = [b"vault", pool.key().as_ref()],
        bump = pool.bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    /// Stake vault which receives staked tokens.
    #[account(mut, constraint = stake_vault.owner == vault_authority.key())]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = staker,
        space = 8 + core::mem::size_of::<UserStake>(),
        seeds = [b"user", pool.key().as_ref(), staker.key().as_ref()],
        bump
    )]
    /// User stake account.
    pub user: Account<'info, UserStake>,

    /// System program.
    pub system_program: Program<'info, System>,
    /// SPL token program.
    pub token_program: Program<'info, Token>,
}

/// Accounts for claiming rewards.
/// NOTE: `vault_authority` must precede `reward_vault` due to constraint reference.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct Claim<'info> {
    /// Staker signing the transaction.
    #[account(mut)]
    pub staker: Signer<'info>,

    /// Pool configuration.
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Vault authority PDA.
    /// CHECK: PDA authority for reward_vault.
    #[account(
        seeds = [b"vault", pool.key().as_ref()],
        bump = pool.bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    /// Reward token vault owned by the program.
    #[account(mut, constraint = reward_vault.owner == vault_authority.key())]
    pub reward_vault: Account<'info, TokenAccount>,

    /// Destination token account for rewards.
    #[account(mut, constraint = staker_reward_account.owner == staker.key())]
    pub staker_reward_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"user", pool.key().as_ref(), staker.key().as_ref()],
        bump
    )]
    /// User stake account.
    pub user: Account<'info, UserStake>,

    /// SPL token program.
    pub token_program: Program<'info, Token>,
}

/// Accounts for unstaking.
/// NOTE: `vault_authority` must precede `stake_vault` due to constraint reference.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct Unstake<'info> {
    /// Staker signing the transaction.
    #[account(mut)]
    pub staker: Signer<'info>,

    /// Pool configuration.
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Vault authority PDA.
    /// CHECK: PDA authority for stake_vault.
    #[account(
        seeds = [b"vault", pool.key().as_ref()],
        bump = pool.bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    /// Stake vault owned by the program.
    #[account(mut, constraint = stake_vault.owner == vault_authority.key())]
    pub stake_vault: Account<'info, TokenAccount>,

    /// Destination token account to receive unstaked tokens.
    #[account(mut, constraint = staker_token.owner == staker.key())]
    pub staker_token: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"user", pool.key().as_ref(), staker.key().as_ref()],
        bump
    )]
    /// User stake account.
    pub user: Account<'info, UserStake>,

    /// SPL token program.
    pub token_program: Program<'info, Token>,
}

/// Accounts for admin updates.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct AdminUpdate<'info> {
    /// Pool authority.
    pub authority: Signer<'info>,
    /// Pool configuration.
    #[account(mut, has_one = authority)]
    pub pool: Account<'info, Pool>,
}

/// Staking errors.
#[error_code]
pub enum StakingError {
    /// Pool already initialized.
    #[msg("Pool already initialized")]
    AlreadyInitialized,
    /// Pool not initialized.
    #[msg("Pool not initialized")]
    Uninitialized,
    /// Lock still applies.
    #[msg("Stake still locked")]
    StillLocked,
    /// No rewards available.
    #[msg("Nothing to claim")]
    NothingToClaim,
}

#[cfg(all(test, not(target_arch = "bpf")))]
mod tests {
    use super::*;

    #[test]
    fn pool_init_sets_flags() {
        let mut pool = Pool {
            authority: Pubkey::default(),
            reward_mint: Pubkey::default(),
            lock_policy: LockPolicy::None,
            emission_rate: 1,
            bump: 0,
            initialized: false,
        };
        pool.initialized = true;
        assert!(pool.initialized);
    }
}
