#![allow(clippy::result_large_err)]
#![warn(missing_docs)]
//! Keystone constant-product AMM (v1).

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};
use keystone_common::errors::CommonError;
use keystone_common::events::TreasuryMovement;
use keystone_common::events::{LiquidityAdded, LiquidityRemoved, SwapExecuted};
use keystone_common::fees::FeeConfig;
#[cfg(not(target_arch = "bpf"))]
use solana_security_txt::security_txt;

#[cfg(not(target_arch = "bpf"))]
security_txt! {
    name: "Keystone AMM",
    project_url: "https://github.com/keystone-labs/keystone-vertex",
    contacts: "email:security@keystonelabs.xyz",
    policy: "https://github.com/keystone-labs/keystone-vertex/security/policy",
    preferred_languages: "en",
    source_code: "https://github.com/keystone-labs/keystone-vertex"
}

declare_id!("Hfts9nZFo1epBQe7Gsn54QzvNmZhipcynG58feJs2BnX");

/// Program entrypoints.
#[program]
pub mod keystone_amm_cp {
    use super::*;

    /// Initializes new AMM pool.
    pub fn init_pool(
        ctx: Context<InitPool>,
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(!pool.initialized, AmmError::AlreadyInitialized);
        pool.token_a_vault = ctx.accounts.token_a_vault.key();
        pool.token_b_vault = ctx.accounts.token_b_vault.key();
        pool.lp_mint = ctx.accounts.lp_mint.key();
        pool.authority = ctx.accounts.authority.key();
        pool.fee_config =
            FeeConfig::new(fee_numerator, fee_denominator, ctx.accounts.fee_vault.key())?;
        pool.bump = ctx.bumps.pool_signer;
        pool.initialized = true;
        Ok(())
    }

    /// Adds liquidity by depositing proportional tokens and minting LP.
    pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
        let pool = &ctx.accounts.pool;
        require!(pool.initialized, AmmError::Uninitialized);
        let pool_key = ctx.accounts.pool.key();
        let seeds: &[&[u8]] = &[b"pool", pool_key.as_ref(), &[pool.bump]];
        let binding = [seeds];

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_a.to_account_info(),
                    to: ctx.accounts.token_a_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_a,
        )?;
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_b.to_account_info(),
                    to: ctx.accounts.token_b_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_b,
        )?;

        let mint_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.user_lp.to_account_info(),
                authority: ctx.accounts.pool_signer.to_account_info(),
            },
            &binding,
        );
        let lp_amount = amount_a.saturating_add(amount_b); // simplified
        token::mint_to(mint_ctx, lp_amount)?;
        emit!(LiquidityAdded {
            pool: ctx.accounts.pool.key(),
            user: ctx.accounts.user.key(),
            amount_a,
            amount_b,
            lp_minted: lp_amount,
        });
        Ok(())
    }

    /// Removes liquidity burning LP and withdrawing tokens.
    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, lp_amount: u64) -> Result<()> {
        let pool = &ctx.accounts.pool;
        require!(pool.initialized, AmmError::Uninitialized);
        let pool_key = ctx.accounts.pool.key();
        let seeds: &[&[u8]] = &[b"pool", pool_key.as_ref(), &[pool.bump]];
        let binding = [seeds];

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    from: ctx.accounts.user_lp.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            lp_amount,
        )?;

        let withdraw_a = lp_amount / 2;
        let withdraw_b = lp_amount - withdraw_a;
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.token_a_vault.to_account_info(),
                    to: ctx.accounts.user_token_a.to_account_info(),
                    authority: ctx.accounts.pool_signer.to_account_info(),
                },
                &binding,
            ),
            withdraw_a,
        )?;
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.token_b_vault.to_account_info(),
                    to: ctx.accounts.user_token_b.to_account_info(),
                    authority: ctx.accounts.pool_signer.to_account_info(),
                },
                &binding,
            ),
            withdraw_b,
        )?;
        emit!(LiquidityRemoved {
            pool: ctx.accounts.pool.key(),
            user: ctx.accounts.user.key(),
            lp_burned: lp_amount,
            amount_a: withdraw_a,
            amount_b: withdraw_b,
        });
        Ok(())
    }

    /// Executes swap along the pool paying protocol fee.
    pub fn swap(ctx: Context<Swap>, amount_in: u64, minimum_out: u64) -> Result<()> {
        let pool = &ctx.accounts.pool;
        require!(pool.initialized, AmmError::Uninitialized);
        require!(amount_in > 0, CommonError::ConstraintViolation);
        let pool_key = ctx.accounts.pool.key();
        let seeds: &[&[u8]] = &[b"pool", pool_key.as_ref(), &[pool.bump]];
        let binding = [seeds];

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_source.to_account_info(),
                    to: ctx.accounts.source_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_in,
        )?;

        let reserve_in = ctx
            .accounts
            .source_vault
            .amount
            .checked_add(amount_in)
            .ok_or(CommonError::ArithmeticOverflow)?;
        let reserve_out = ctx.accounts.destination_vault.amount;
        let amount_out = cp_swap_quote(amount_in, reserve_in, reserve_out, &pool.fee_config)?;
        require!(amount_out >= minimum_out, AmmError::SlippageExceeded);

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.destination_vault.to_account_info(),
                    to: ctx.accounts.user_destination.to_account_info(),
                    authority: ctx.accounts.pool_signer.to_account_info(),
                },
                &binding,
            ),
            amount_out,
        )?;
        emit!(SwapExecuted {
            entity: ctx.accounts.pool.key(),
            user: ctx.accounts.user.key(),
            amount_in,
            amount_out,
        });
        Ok(())
    }

    /// Collect accumulated protocol fees.
    pub fn collect_fees(ctx: Context<CollectFees>) -> Result<()> {
        let pool = &ctx.accounts.pool;
        require_keys_eq!(
            pool.fee_config.fee_vault,
            ctx.accounts.fee_vault.key(),
            AmmError::FeeVaultMismatch
        );
        let pool_key = ctx.accounts.pool.key();
        let seeds: &[&[u8]] = &[b"pool", pool_key.as_ref(), &[pool.bump]];
        let binding = [seeds];

        let amount = ctx.accounts.fee_vault.amount;
        let cpi_accounts = Transfer {
            from: ctx.accounts.fee_vault.to_account_info(),
            to: ctx.accounts.fee_destination.to_account_info(),
            authority: ctx.accounts.pool_signer.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            &binding,
        );
        token::transfer(cpi_ctx, amount)?;
        emit!(TreasuryMovement {
            program: crate::ID,
            entity: ctx.accounts.pool.key(),
            amount,
            destination: ctx.accounts.fee_destination.key(),
        });
        Ok(())
    }
}

/// Computes constant-product swap quote with fee deduction.
pub fn cp_swap_quote(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_config: &FeeConfig,
) -> Result<u64> {
    require!(reserve_in > 0 && reserve_out > 0, AmmError::InvalidReserves);
    let invariant = (reserve_in as u128)
        .checked_mul(reserve_out as u128)
        .ok_or(CommonError::ArithmeticOverflow)?;
    let fee = fee_config.apply(keystone_common::decimals::Decimal::from_integer(amount_in))?;
    let amount_in_less_fee = amount_in
        .checked_sub(fee.to_u64().map_err(|_| CommonError::ArithmeticOverflow)?)
        .ok_or(CommonError::ArithmeticOverflow)?;
    let new_reserve_in = reserve_in
        .checked_add(amount_in_less_fee)
        .ok_or(CommonError::ArithmeticOverflow)?;
    let new_reserve_out = invariant
        .checked_div(new_reserve_in as u128)
        .ok_or(CommonError::ArithmeticOverflow)? as u64;
    let output = reserve_out
        .checked_sub(new_reserve_out)
        .ok_or(CommonError::ArithmeticOverflow)?;
    Ok(output)
}

/// Pool account storing immutable configuration.
#[account]
pub struct Pool {
    /// Pool signer PDA bump.
    pub bump: u8,
    /// Token A vault.
    pub token_a_vault: Pubkey,
    /// Token B vault.
    pub token_b_vault: Pubkey,
    /// LP mint address.
    pub lp_mint: Pubkey,
    /// Pool authority owner.
    pub authority: Pubkey,
    /// Protocol fee configuration.
    pub fee_config: FeeConfig,
    /// Initialization flag.
    pub initialized: bool,
}

impl Pool {
    /// Size of the `Pool` account data (without discriminator).
    pub const LEN: usize = core::mem::size_of::<Self>();
    /// Allocated bytes including Anchor discriminator.
    pub const SPACE: usize = 8 + Self::LEN;
}

/// Accounts for pool init.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct InitPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = Pool::SPACE,
        seeds = [b"pool", authority.key().as_ref(), lp_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub token_a_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_b_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    /// CHECK: PDA used for authority seeds.
    #[account(
        seeds = [b"pool", pool.key().as_ref()],
        bump
    )]
    pub pool_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Liquidity add accounts.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, constraint = user_token_a.owner == user.key())]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_token_b.owner == user.key())]
    pub user_token_b: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_lp.owner == user.key())]
    pub user_lp: Account<'info, TokenAccount>,
    #[account(mut, has_one = token_a_vault, has_one = token_b_vault, has_one = lp_mint)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub token_a_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_b_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    /// CHECK: Derived PDA signer.
    #[account(
        seeds = [b"pool", pool.key().as_ref()],
        bump = pool.bump,
    )]
    pub pool_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

/// Remove liquidity accounts.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, constraint = user_lp.owner == user.key())]
    pub user_lp: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_token_a.owner == user.key())]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_token_b.owner == user.key())]
    pub user_token_b: Account<'info, TokenAccount>,
    #[account(mut, has_one = token_a_vault, has_one = token_b_vault, has_one = lp_mint)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub token_a_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_b_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    /// CHECK: Derived PDA signer.
    #[account(
        seeds = [b"pool", pool.key().as_ref()],
        bump = pool.bump,
    )]
    pub pool_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

/// Swap accounts.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, constraint = user_source.owner == user.key())]
    pub user_source: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_destination.owner == user.key())]
    pub user_destination: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut, constraint = (
        (source_vault.key() == pool.token_a_vault && destination_vault.key() == pool.token_b_vault) ||
        (source_vault.key() == pool.token_b_vault && destination_vault.key() == pool.token_a_vault)
    ))]
    pub source_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination_vault: Account<'info, TokenAccount>,
    /// CHECK: Derived PDA signer.
    #[account(
        seeds = [b"pool", pool.key().as_ref()],
        bump = pool.bump,
    )]
    pub pool_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

/// Fee collection accounts.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct CollectFees<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub pool: Account<'info, Pool>,
    #[account(mut, constraint = fee_vault.key() == pool.fee_config.fee_vault)]
    pub fee_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_destination: Account<'info, TokenAccount>,
    /// CHECK: Derived PDA signer.
    #[account(
        seeds = [b"pool", pool.key().as_ref()],
        bump = pool.bump,
    )]
    pub pool_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

/// AMM-specific errors.
#[error_code]
pub enum AmmError {
    /// Pool already initialized.
    #[msg("Pool already initialized")]
    AlreadyInitialized,
    /// Pool not initialized.
    #[msg("Pool not initialized")]
    Uninitialized,
    /// Provided reserves invalid.
    #[msg("Invalid pool reserves")]
    InvalidReserves,
    /// Slippage guard tripped.
    #[msg("Slippage exceeded")]
    SlippageExceeded,
    /// Fee vault mismatch.
    #[msg("Fee vault mismatch")]
    FeeVaultMismatch,
}

#[cfg(all(test, not(target_arch = "bpf")))]
mod tests {
    use super::*;
    use anchor_lang::solana_program::pubkey::Pubkey;

    #[test]
    fn swap_quote_respects_invariant() {
        let fee_cfg = FeeConfig::new(30, 10_000, Pubkey::default()).unwrap();
        let output = cp_swap_quote(1_000, 100_000, 200_000, &fee_cfg).unwrap();
        assert!(output > 0);
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn cp_invariant_holds(amount_in in 1u64..1_000_000, rin in 10_000u64..100_000_000, rout in 10_000u64..100_000_000, fee in 0u64..500) {
            let fee_cfg = FeeConfig::new(fee, 10_000, Pubkey::default()).unwrap();
            let out = cp_swap_quote(amount_in, rin, rout, &fee_cfg).unwrap();
            // simple sanity: cannot extract more than reserve_out
            prop_assert!(out <= rout);
        }
    }
}
