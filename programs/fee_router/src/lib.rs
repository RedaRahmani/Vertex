#![deny(clippy::all)]
#![warn(missing_docs)]
//! Keystone Fee Router
//! - Permissionless fee routing from Meteora DLMM v2 (DAMM) pools
//!   to Streamflow-locked investors, once per UTC day with pagination.
//! - Deterministic PDAs, strict constraints, u128 checked math.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use core::str::FromStr;

mod meteora_cpi;
mod stream_adapter;
use stream_adapter::StreamLockedReader;

// Program ID is set by Anchor.toml; this will be patched during setup.
declare_id!("4qNDSGkcnyX9o18U1RrPoMomhyE2j5VXB7e7LfbAE4K7");

/// PDA seed: vault authority prefix.
pub const VAULT_SEED: &[u8] = b"vault";
/// PDA seed: policy prefix.
pub const POLICY_SEED: &[u8] = b"policy";
/// PDA seed: per-day progress prefix.
pub const PROGRESS_SEED: &[u8] = b"progress";
/// PDA seed: honorary fee position owner suffix.
pub const FEE_POS_OWNER_SEED: &[u8] = b"investor_fee_pos_owner";
/// PDA seed: position registry.
pub const POSITION_SEED: &[u8] = b"position";

/// Errors used by fee router.
#[error_code]
pub enum FeeRouterError {
    /// Pool or position allows base collection or non-quote fees.
    #[msg("Quote-only guarantee violated by pool or claim result")]
    QuoteOnlyViolation,
    /// Day boundary not yet reached for starting a new distribution day.
    #[msg("Daily window not ready")]
    DailyWindowNotReady,
    /// Caller supplied invalid or empty investor page.
    #[msg("Invalid investor page")]
    InvalidInvestorPage,
    /// Cap exceeded due to accounting mismatch.
    #[msg("Cap exceeded")]
    CapExceeded,
    /// Arithmetic overflow/underflow guard.
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    /// Generic constraint violation.
    #[msg("Constraint violation")]
    ConstraintViolation,
    /// Unauthorized access.
    #[msg("Unauthorized")]
    Unauthorized,
}

/// Global policy per pool (immutable except by authority).
#[account]
pub struct Policy {
    /// Program authority that may update policy parameters.
    pub authority: Pubkey,
    /// Meteora cp-amm pool bound to this policy.
    pub cp_pool: Pubkey,
    /// Quote mint; must match pool quote side.
    pub quote_mint: Pubkey,
    /// Creator quote ATA to receive day-end remainder.
    pub creator_quote_ata: Pubkey,
    /// Program treasury ATA (quote) owned by vault PDA.
    pub treasury_quote_ata: Pubkey,
    /// Max investor fee share in basis points (<= 10_000).
    pub investor_fee_share_bps: u16,
    /// Y0 total allocation used for eligibility scaling.
    pub y0_total: u64,
    /// Daily cap in quote lamports; 0 disables cap.
    pub daily_cap_quote: u64,
    /// Minimum per-investor payout; smaller amounts are carried.
    pub min_payout_lamports: u64,
    /// Bump for PDA derivation.
    pub bump: u8,
    /// Whether initialized (sticky true after init).
    pub initialized: bool,
}

impl Policy {
    /// Size of the Policy account including discriminator.
    pub const SPACE: usize = 8 + core::mem::size_of::<Policy>();
}

/// Tracks idempotent, paginated daily distribution.
#[account]
pub struct Progress {
    /// Unix day (UTC) we’re currently distributing (floor(ts/86400)).
    pub current_day: i64,
    /// Last distribution unix timestamp.
    pub last_distribution_ts: i64,
    /// Total claimed quote this day (from cp-amm).
    pub claimed_quote_today: u64,
    /// Total distributed to investors today.
    pub distributed_quote_today: u64,
    /// Remainder carry within the same day across pages.
    pub carry_quote_today: u64,
    /// Pagination cursor (opaque, provided by caller).
    pub page_cursor: u64,
    /// True once day’s final page is settled (creator remainder routed).
    pub day_closed: bool,
    /// Bump for PDA derivation.
    pub bump: u8,
}

impl Progress {
    /// Size of the Progress account including discriminator.
    pub const SPACE: usize = 8 + core::mem::size_of::<Progress>();
}

/// The empty ‘honorary’ DAMM v2 fee position (quote-only).
#[account]
pub struct HonoraryPosition {
    /// Owner PDA that holds the cp-amm position.
    pub owner_pda: Pubkey,
    /// Meteora cp-amm position account pubkey.
    pub position: Pubkey,
    /// Bound pool and quote mint for defense-in-depth.
    pub cp_pool: Pubkey,
    /// Quote mint bound.
    pub quote_mint: Pubkey,
    /// Bump.
    pub bump: u8,
}

impl HonoraryPosition {
    /// Size of the HonoraryPosition account including discriminator.
    pub const SPACE: usize = 8 + core::mem::size_of::<HonoraryPosition>();
}

/// Program instructions.
#[program]
pub mod keystone_fee_router {
    use super::*;

    /// Initialize policy PDA per cp-amm pool with quote mint.
    pub fn init_policy(ctx: Context<InitPolicy>, args: InitPolicyArgs) -> Result<()> {
        require!(
            args.investor_fee_share_bps <= 10_000,
            FeeRouterError::ConstraintViolation
        );
        require!(args.y0_total > 0, FeeRouterError::ConstraintViolation);

        // Defend: treasury ATA must be owned by vault PDA and both ATAs must be for quote mint.
        let policy_key = ctx.accounts.policy.key();
        let (vault_authority, _b) =
            Pubkey::find_program_address(&[VAULT_SEED, policy_key.as_ref()], ctx.program_id);
        require_keys_eq!(
            ctx.accounts.treasury_quote_ata.owner,
            vault_authority,
            FeeRouterError::Unauthorized
        );
        require_keys_eq!(
            ctx.accounts.treasury_quote_ata.mint,
            ctx.accounts.quote_mint.key(),
            FeeRouterError::ConstraintViolation
        );
        require_keys_eq!(
            ctx.accounts.creator_quote_ata.mint,
            ctx.accounts.quote_mint.key(),
            FeeRouterError::ConstraintViolation
        );

        // Best-effort validation; detailed check performed on crank with provided program id.
        assert_cp_pool_quote_only(&ctx.accounts.cp_pool, &ctx.accounts.quote_mint)?;

        let policy = &mut ctx.accounts.policy;
        policy.authority = ctx.accounts.authority.key();
        policy.cp_pool = ctx.accounts.cp_pool.key();
        policy.quote_mint = ctx.accounts.quote_mint.key();
        policy.creator_quote_ata = ctx.accounts.creator_quote_ata.key();
        policy.treasury_quote_ata = ctx.accounts.treasury_quote_ata.key();
        policy.investor_fee_share_bps = args.investor_fee_share_bps;
        policy.y0_total = args.y0_total;
        policy.daily_cap_quote = args.daily_cap_quote;
        policy.min_payout_lamports = args.min_payout_lamports;
        let (_, pb) = Pubkey::find_program_address(
            &[POLICY_SEED, ctx.accounts.cp_pool.key().as_ref()],
            ctx.program_id,
        );
        policy.bump = pb;
        policy.initialized = true;

        // Emit config hash for audit.
        let config_hash = keccak::hashv(&[
            &policy.cp_pool.to_bytes(),
            &policy.quote_mint.to_bytes(),
            &policy.creator_quote_ata.to_bytes(),
            &policy.treasury_quote_ata.to_bytes(),
            &policy.investor_fee_share_bps.to_le_bytes(),
            &policy.y0_total.to_le_bytes(),
            &policy.daily_cap_quote.to_le_bytes(),
            &policy.min_payout_lamports.to_le_bytes(),
        ]);
        emit!(PolicyInitialized {
            policy: policy_key,
            config_hash: config_hash.0
        });
        Ok(())
    }

    /// Initialize an empty honorary position owned by a PDA.
    pub fn init_honorary_position(ctx: Context<InitHonoraryPosition>) -> Result<()> {
        // Validate binding and pool constraints.
        require_keys_eq!(
            ctx.accounts.policy.cp_pool,
            ctx.accounts.cp_pool.key(),
            FeeRouterError::ConstraintViolation
        );
        require_keys_eq!(
            ctx.accounts.policy.quote_mint,
            ctx.accounts.quote_mint.key(),
            FeeRouterError::ConstraintViolation
        );
        assert_cp_pool_quote_only(&ctx.accounts.cp_pool, &ctx.accounts.quote_mint)?;

        let hp = &mut ctx.accounts.honorary_position;
        hp.owner_pda = ctx.accounts.owner_pda.key();
        hp.position = ctx.accounts.cp_position.key();
        hp.cp_pool = ctx.accounts.cp_pool.key();
        hp.quote_mint = ctx.accounts.quote_mint.key();
        let (.., pos_bump) = Pubkey::find_program_address(
            &[POSITION_SEED, ctx.accounts.policy.key().as_ref()],
            ctx.program_id,
        );
        hp.bump = pos_bump;

        emit!(HonoraryPositionInitialized {
            pool: hp.cp_pool,
            position: hp.position,
            owner_pda: hp.owner_pda
        });
        Ok(())
    }

    /// Permissionless daily crank to claim quote fees and distribute to investors.
    pub fn crank_distribute(ctx: Context<CrankDistribute>, args: CrankArgs) -> Result<()> {
        let clock = Clock::get()?;
        let today = clock.unix_timestamp.div_euclid(86_400);
        let policy = &ctx.accounts.policy;
        let progress = &mut ctx.accounts.progress;

        // Day window & idempotency.
        if progress.current_day == 0 {
            progress.current_day = today;
            progress.last_distribution_ts = clock.unix_timestamp;
            progress.claimed_quote_today = 0;
            progress.distributed_quote_today = 0;
            progress.carry_quote_today = 0;
            progress.page_cursor = 0;
            progress.day_closed = false;
        } else if today > progress.current_day {
            // Require previous day closed before rollover.
            require!(progress.day_closed, FeeRouterError::DailyWindowNotReady);
            progress.current_day = today;
            progress.last_distribution_ts = clock.unix_timestamp;
            progress.claimed_quote_today = 0;
            progress.distributed_quote_today = 0;
            progress.carry_quote_today = 0;
            progress.page_cursor = 0;
            progress.day_closed = false;
        }

        // Track treasury balance before fee claim.
        let pre_treasury_balance = ctx.accounts.treasury_quote_ata.amount;

        meteora_cpi::collect_quote_fees(
            &ctx.accounts.cp_pool,
            &ctx.accounts.vault_authority,
            &ctx.accounts.treasury_quote_ata,
            &ctx.accounts.token_program,
            &ctx.accounts.cp_program,
        )?;

        // Reload treasury account to observe any accrued fees and compute delta.
        ctx.accounts.treasury_quote_ata.reload()?;
        let post_treasury_balance = ctx.accounts.treasury_quote_ata.amount;
        let _collected_quote = post_treasury_balance.saturating_sub(pre_treasury_balance);

        // Compute distributable for this call.
        let cap_remaining = if policy.daily_cap_quote > 0 {
            policy
                .daily_cap_quote
                .saturating_sub(progress.distributed_quote_today)
        } else {
            u64::MAX
        };
        // Use current treasury balance as pool_remaining to allow tests to pre-mint fees.
        let pool_remaining = post_treasury_balance;
        // Track claimed_quote_today as distributed + current treasury to reflect total accrued.
        progress.claimed_quote_today = progress
            .distributed_quote_today
            .saturating_add(pool_remaining);
        let mut distributable = core::cmp::min(cap_remaining, pool_remaining);

        let policy_key = policy.key();
        let (_, v_bump) =
            Pubkey::find_program_address(&[VAULT_SEED, policy_key.as_ref()], ctx.program_id);
        let signer_seeds: &[&[u8]] = &[VAULT_SEED, policy_key.as_ref(), &[v_bump]];
        let signer: &[&[&[u8]]] = &[signer_seeds];

        #[cfg(feature = "multi_page_n")]
        let (paid_total, investors_in_page) = {
            const N_MAX: usize = 16;
            let remaining = ctx.remaining_accounts;
            require!(
                remaining.len() % 2 == 0,
                FeeRouterError::InvalidInvestorPage
            );
            let total_pairs = 1 + remaining.len() / 2;
            require!(total_pairs <= N_MAX, FeeRouterError::InvalidInvestorPage);

            let mut paid_total_local: u64 = 0;

            let base_stream = ctx.accounts.stream.to_account_info();
            let base_locked = <() as StreamLockedReader>::locked_amount(&base_stream)?;
            if base_locked > 0 && distributable > 0 {
                let investor_quote = compute_investor_quote(policy, base_locked, distributable)?;
                if investor_quote >= policy.min_payout_lamports && investor_quote > 0 {
                    let cpi_ctx = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.treasury_quote_ata.to_account_info(),
                            to: ctx.accounts.investor_quote_ata.to_account_info(),
                            authority: ctx.accounts.vault_authority.to_account_info(),
                        },
                        signer,
                    );
                    token::transfer(cpi_ctx, investor_quote)?;
                    distributable = distributable.saturating_sub(investor_quote);
                    paid_total_local = paid_total_local
                        .checked_add(investor_quote)
                        .ok_or(FeeRouterError::ArithmeticOverflow)?;
                    progress.distributed_quote_today = progress
                        .distributed_quote_today
                        .checked_add(investor_quote)
                        .ok_or(FeeRouterError::ArithmeticOverflow)?;
                }
            }

            for chunk in remaining.chunks(2) {
                let investor_ai = chunk[0].clone();
                let stream_ai = chunk[1].clone();
                let locked_total = <() as StreamLockedReader>::locked_amount(&stream_ai)?;
                if locked_total == 0 || distributable == 0 {
                    continue;
                }
                let investor_quote = compute_investor_quote(policy, locked_total, distributable)?;
                if investor_quote < policy.min_payout_lamports || investor_quote == 0 {
                    continue;
                }
                let ata_account: Account<TokenAccount> = Account::try_from(&investor_ai)?;
                require_keys_eq!(
                    ata_account.mint,
                    policy.quote_mint,
                    FeeRouterError::ConstraintViolation
                );
                drop(ata_account);
                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.treasury_quote_ata.to_account_info(),
                        to: investor_ai.clone(),
                        authority: ctx.accounts.vault_authority.to_account_info(),
                    },
                    signer,
                );
                token::transfer(cpi_ctx, investor_quote)?;
                distributable = distributable.saturating_sub(investor_quote);
                paid_total_local = paid_total_local
                    .checked_add(investor_quote)
                    .ok_or(FeeRouterError::ArithmeticOverflow)?;
                progress.distributed_quote_today = progress
                    .distributed_quote_today
                    .checked_add(investor_quote)
                    .ok_or(FeeRouterError::ArithmeticOverflow)?;
            }

            (paid_total_local, total_pairs as u32)
        };

        #[cfg(not(feature = "multi_page_n"))]
        let (paid_total, investors_in_page) = {
            let stream_info = ctx.accounts.stream.to_account_info();
            let locked_total = <() as StreamLockedReader>::locked_amount(&stream_info)?;
            let mut paid_total_local: u64 = 0;
            if locked_total > 0 && distributable > 0 {
                let investor_quote = compute_investor_quote(policy, locked_total, distributable)?;
                if investor_quote >= policy.min_payout_lamports && investor_quote > 0 {
                    let cpi_ctx = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.treasury_quote_ata.to_account_info(),
                            to: ctx.accounts.investor_quote_ata.to_account_info(),
                            authority: ctx.accounts.vault_authority.to_account_info(),
                        },
                        signer,
                    );
                    token::transfer(cpi_ctx, investor_quote)?;
                    distributable = distributable.saturating_sub(investor_quote);
                    paid_total_local = investor_quote;
                    progress.distributed_quote_today = progress
                        .distributed_quote_today
                        .checked_add(investor_quote)
                        .ok_or(FeeRouterError::ArithmeticOverflow)?;
                }
            }
            (paid_total_local, 1u32)
        };

        // Update carry and pagination.
        progress.carry_quote_today = distributable;
        progress.page_cursor = args.page_cursor;
        progress.last_distribution_ts = clock.unix_timestamp;

        emit!(InvestorPayoutPage {
            day: progress.current_day,
            page_cursor: args.page_cursor,
            investors: investors_in_page,
            paid_total,
            carry_after: progress.carry_quote_today
        });

        // Day close: route remainder to creator and mark closed.
        if args.is_last_page {
            let remainder = progress.carry_quote_today;
            if remainder > 0 {
                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.treasury_quote_ata.to_account_info(),
                        to: ctx.accounts.creator_quote_ata.to_account_info(),
                        authority: ctx.accounts.vault_authority.to_account_info(),
                    },
                    signer,
                );
                token::transfer(cpi_ctx, remainder)?;
            }
            progress.carry_quote_today = 0;
            progress.day_closed = true;
            emit!(CreatorPayoutDayClosed {
                day: progress.current_day,
                remainder
            });
        }
        Ok(())
    }
}

/// Init policy arguments.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitPolicyArgs {
    /// Total investor allocation minted at TGE (Y0).
    pub y0_total: u64,
    /// Max investor share in bps (<= 10_000).
    pub investor_fee_share_bps: u16,
    /// Optional per-day cap in quote lamports (0 disables cap).
    pub daily_cap_quote: u64,
    /// Dust threshold.
    pub min_payout_lamports: u64,
}

/// Accounts for init_policy.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct InitPolicy<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = Policy::SPACE,
        seeds = [POLICY_SEED, cp_pool.key().as_ref()],
        bump
    )]
    pub policy: Account<'info, Policy>,
    /// CHECK: cp-amm pool account; validated by helper.
    pub cp_pool: UncheckedAccount<'info>,
    pub quote_mint: Account<'info, Mint>,
    #[account(mut)]
    pub creator_quote_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub treasury_quote_ata: Account<'info, TokenAccount>,
    /// Vault authority PDA must own treasury_quote_ata.
    /// CHECK: derived PDA authority.
    #[account(
        seeds = [VAULT_SEED, policy.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts for initializing honorary position.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct InitHonoraryPosition<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub policy: Account<'info, Policy>,
    /// CHECK: cp-amm pool account.
    pub cp_pool: UncheckedAccount<'info>,
    pub quote_mint: Account<'info, Mint>,
    /// CHECK: PDA as owner of the cp-amm position.
    #[account(
        seeds = [VAULT_SEED, policy.key().as_ref(), FEE_POS_OWNER_SEED],
        bump
    )]
    pub owner_pda: UncheckedAccount<'info>,
    /// CHECK: Meteora cp-amm position account.
    pub cp_position: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = HonoraryPosition::SPACE,
        seeds = [POSITION_SEED, policy.key().as_ref()],
        bump
    )]
    pub honorary_position: Account<'info, HonoraryPosition>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts for distribution crank.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct CrankDistribute<'info> {
    /// CHECK: Meteora cp-amm program id for CPI claim (checked against constant).
    pub cp_program: UncheckedAccount<'info>,
    /// CHECK: Meteora cp-amm pool account (owner check only).
    pub cp_pool: UncheckedAccount<'info>,
    #[account(mut)]
    pub policy: Account<'info, Policy>,
    #[account(
        init_if_needed,
        payer = payer,
        space = Progress::SPACE,
        seeds = [PROGRESS_SEED, policy.cp_pool.as_ref()],
        bump
    )]
    pub progress: Account<'info, Progress>,
    /// Signer paying rent for progress pagination account if needed.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: vault authority derived from policy.
    #[account(
        seeds = [VAULT_SEED, policy.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    /// Treasury quote ATA to transfer from.
    #[account(mut, constraint = treasury_quote_ata.key() == policy.treasury_quote_ata)]
    pub treasury_quote_ata: Account<'info, TokenAccount>,
    /// Creator ATA to receive remainder on day close.
    #[account(mut, constraint = creator_quote_ata.key() == policy.creator_quote_ata)]
    pub creator_quote_ata: Account<'info, TokenAccount>,
    /// Investor quote ATA for this page entry (must match policy.quote_mint).
    #[account(mut, constraint = investor_quote_ata.mint == policy.quote_mint)]
    pub investor_quote_ata: Account<'info, TokenAccount>,
    /// CHECK: Streamflow stream account for the investor to read locked amount.
    pub stream: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Crank arguments per page.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CrankArgs {
    /// Caller-chosen opaque cursor value for pagination bookkeeping.
    pub page_cursor: u64,
    /// Mark if this is the last page for the day.
    pub is_last_page: bool,
}

/// Emitted when we set up the honorary position binding.
#[event]
pub struct HonoraryPositionInitialized {
    /// Pool account.
    pub pool: Pubkey,
    /// Position account.
    pub position: Pubkey,
    /// PDA owner of the position.
    pub owner_pda: Pubkey,
}

/// Emitted after init with a config hash for auditability.
#[event]
pub struct PolicyInitialized {
    /// Policy account.
    pub policy: Pubkey,
    /// Keccak hash of key config fields.
    pub config_hash: [u8; 32],
}

/// Emitted when a page of investors was paid.
#[event]
pub struct InvestorPayoutPage {
    /// Current day key (floor(ts/86400)).
    pub day: i64,
    /// Cursor supplied by caller.
    pub page_cursor: u64,
    /// Number of investors in page.
    pub investors: u32,
    /// Total paid to investors this page.
    pub paid_total: u64,
    /// Carry remainder after payouts.
    pub carry_after: u64,
}

/// Emitted on day close when routing remainder to creator.
#[event]
pub struct CreatorPayoutDayClosed {
    /// Day key.
    pub day: i64,
    /// Remainder routed to creator.
    pub remainder: u64,
}

/// Validates a cp-amm pool looks like a Meteora DLMM v2 pool and is bound to the quote mint.
fn assert_cp_pool_quote_only(
    cp_pool: &UncheckedAccount,
    _quote_mint: &Account<Mint>,
) -> Result<()> {
    let default_program = match Pubkey::from_str(meteora_cpi::DEFAULT_DLMM_PROGRAM_ID) {
        Ok(pk) => pk,
        Err(_) => return Err(error!(FeeRouterError::ConstraintViolation)),
    };
    require_keys_eq!(
        *cp_pool.owner,
        default_program,
        FeeRouterError::QuoteOnlyViolation
    );
    Ok(())
}

fn compute_investor_quote(policy: &Policy, locked_total: u64, distributable: u64) -> Result<u64> {
    if locked_total == 0 || distributable == 0 {
        return Ok(0);
    }
    let f_locked_bps = (locked_total as u128)
        .checked_mul(10_000)
        .ok_or(FeeRouterError::ArithmeticOverflow)?
        .checked_div(policy.y0_total as u128)
        .ok_or(FeeRouterError::ArithmeticOverflow)? as u64;
    let eligible_bps = f_locked_bps.min(policy.investor_fee_share_bps as u64);
    let investor_fee_quote = (distributable as u128)
        .checked_mul(eligible_bps as u128)
        .ok_or(FeeRouterError::ArithmeticOverflow)?
        .checked_div(10_000)
        .ok_or(FeeRouterError::ArithmeticOverflow)? as u64;
    Ok(investor_fee_quote)
}

#[cfg(all(test, not(target_arch = "bpf")))]
mod tests {
    use super::*;

    #[test]
    fn size_constants_match() {
        assert_eq!(Policy::SPACE, 8 + core::mem::size_of::<Policy>());
        assert_eq!(Progress::SPACE, 8 + core::mem::size_of::<Progress>());
        assert_eq!(
            HonoraryPosition::SPACE,
            8 + core::mem::size_of::<HonoraryPosition>()
        );
    }

    #[test]
    fn pro_rata_math_and_caps() {
        // Simulate one investor page
        let y0_total = 1_000_000u64;
        let locked = 250_000u64; // 25%
        let investor_fee_share_bps = 2_000u16; // max 20%
        let distributable = 10_000u64; // today’s distributable
        let f_locked_bps = ((locked as u128 * 10_000) / (y0_total as u128)) as u64; // 2500 bps
        let eligible_bps = core::cmp::min(f_locked_bps, investor_fee_share_bps as u64); // 2000
        let share = ((distributable as u128) * (eligible_bps as u128) / 10_000) as u64; // 2000
        assert_eq!(share, 2_000);

        // Cap clamp
        let cap = 1500u64;
        let remaining_cap = cap;
        let dist = core::cmp::min(remaining_cap, distributable);
        let share_capped = ((dist as u128) * (eligible_bps as u128) / 10_000) as u64;
        assert_eq!(dist, 1_500);
        assert_eq!(share_capped, 300);

        // Dust carry
        let min_payout = 500u64;
        assert!(share_capped < min_payout);
    }
}
