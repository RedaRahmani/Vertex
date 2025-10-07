#![allow(clippy::result_large_err)]
#![warn(missing_docs)]
//! Keystone Launchpad program implementing token sale primitives.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};
use keystone_common::authority::assert_signer_is;
use keystone_common::errors::CommonError;
use keystone_common::events::{ConfigUpdated, TreasuryMovement};
use keystone_common::merkle::assert_merkle_proof;
use keystone_common::time::{assert_not_expired, assert_started};
#[cfg(not(target_arch = "bpf"))]
use solana_security_txt::security_txt;

pub mod state;

use crate::state::{AuctionConfig, LaunchConfig, LaunchPricing, SaleState, SaleStatus};

#[cfg(not(target_arch = "bpf"))]
security_txt! {
    name: "Keystone Launchpad",
    project_url: "https://github.com/keystone-labs/keystone-vertex",
    contacts: "email:security@keystonelabs.xyz",
    policy: "https://github.com/keystone-labs/keystone-vertex/security/policy",
    preferred_languages: "en",
    source_code: "https://github.com/keystone-labs/keystone-vertex"
}

declare_id!("5VFvyBybqEMVChCDBd6qncckSFcKUzn1owdjyThyxHx5");

/// Program instructions.
#[program]
pub mod keystone_launchpad {
    use super::*;

    /// Initializes new launch configuration.
    pub fn init_launch(ctx: Context<InitLaunch>, args: InitLaunchArgs) -> Result<()> {
        let config = &mut ctx.accounts.launch_config;
        require!(!config.initialized, LaunchError::AlreadyInitialized);
        let mut new_config = LaunchConfig::try_from_args(
            &ctx.accounts.authority.key(),
            &ctx.accounts.treasury_vault.key(),
            &ctx.accounts.mint.key(),
            args,
        )?;
        new_config.treasury_bump = ctx.bumps.treasury_authority;
        new_config.config_bump = ctx.bumps.launch_config;
        new_config.sale_state_bump = ctx.bumps.sale_state;
        new_config.initialized = true;
        config.set_inner(new_config);
        // Initialize zero-copy sale state in-place
        let mut state = ctx.accounts.sale_state.load_init()?;
        *state = SaleState::default();
        emit!(ConfigUpdated {
            entity: config.key(),
            slot: Clock::get()?.slot,
            config_hash: config.config_hash(),
        });
        Ok(())
    }

    /// Updates mutable configuration fields.
    pub fn update_config(ctx: Context<UpdateConfig>, args: UpdateConfigArgs) -> Result<()> {
        let config = &mut ctx.accounts.launch_config;
        assert_signer_is(config.authority(), &ctx.accounts.authority)?;
        config.update(args)?;
        emit!(ConfigUpdated {
            entity: config.key(),
            slot: Clock::get()?.slot,
            config_hash: config.config_hash(),
        });
        Ok(())
    }

    /// Buy tokens from sale respecting pricing model.
    pub fn buy(
        ctx: Context<Buy>,
        amount: u64,
        proof: Option<Vec<[u8; 32]>>,
        max_quote: u64,
    ) -> Result<()> {
        let config = &ctx.accounts.launch_config;
        let clock = Clock::get()?;
        assert_started(&clock, config.start_time)?;
        assert_not_expired(&clock, config.end_time)?;

        if let Some(root) = config.whitelist_root {
            let buyer = ctx.accounts.buyer.key();
            let leaf = keccak::hashv(&[buyer.as_ref()]).to_bytes();
            let proof_vec = proof.clone().ok_or(LaunchError::WhitelistRequired)?;
            assert_merkle_proof(leaf, &proof_vec, root)?;
        }

        let mut state = ctx.accounts.sale_state.load_mut()?;
        state.assert_allows_purchase(&ctx.accounts.buyer.key(), amount, config)?;

        let quote_amount = config.pricing.quote_buy(state.sold, amount)?;
        require!(quote_amount <= max_quote, LaunchError::SlippageExceeded);

        let cpi_accounts = Transfer {
            from: ctx.accounts.quote_account.to_account_info(),
            to: ctx.accounts.treasury_vault.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, quote_amount)?;

        state.record_purchase(&ctx.accounts.buyer.key(), amount, quote_amount, config)?;
        emit!(TreasuryMovement {
            program: crate::ID,
            entity: ctx.accounts.launch_config.key(),
            amount: quote_amount,
            destination: ctx.accounts.treasury_vault.key(),
        });
        // Also emit a lightweight purchase event via TreasuryMovement already emitted.

        let config_key = ctx.accounts.launch_config.key();
        let seeds: &[&[u8]] = &[b"treasury", config_key.as_ref(), &[config.treasury_bump]];
        let binding = [seeds];
        let mint_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.buyer_receipt.to_account_info(),
            authority: ctx.accounts.treasury_authority.to_account_info(),
        };
        let mint_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            mint_accounts,
            &binding,
        );
        token::mint_to(mint_ctx, amount)?;
        Ok(())
    }

    /// Sell tokens back into bonding curve inventory.
    pub fn sell(ctx: Context<Sell>, amount: u64, min_quote: u64) -> Result<()> {
        let config = &ctx.accounts.launch_config;
        require!(
            matches!(config.pricing, LaunchPricing::BondingCurve { .. }),
            LaunchError::SellOnlyCurve
        );
        let clock = Clock::get()?;
        assert_started(&clock, config.start_time)?;
        assert_not_expired(&clock, config.end_time)?;
        require!(amount > 0, CommonError::ConstraintViolation);

        let mut state = ctx.accounts.sale_state.load_mut()?;
        let quote_amount = config.pricing.quote_sell(state.sold, amount)?;
        require!(quote_amount >= min_quote, LaunchError::SlippageExceeded);

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.seller_receipt.to_account_info(),
                    authority: ctx.accounts.seller.to_account_info(),
                },
            ),
            amount,
        )?;

        let config_key = ctx.accounts.launch_config.key();
        let seeds: &[&[u8]] = &[b"treasury", config_key.as_ref(), &[config.treasury_bump]];
        let binding = [seeds];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury_vault.to_account_info(),
                to: ctx.accounts.seller_quote_account.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
            &binding,
        );
        token::transfer(cpi_ctx, quote_amount)?;
        state.record_sell(&ctx.accounts.seller.key(), amount, quote_amount)?;
        Ok(())
    }

    /// Place auction bid.
    pub fn bid(ctx: Context<Bid>, amount: u64, proof: Option<Vec<[u8; 32]>>) -> Result<()> {
        let config = &ctx.accounts.launch_config;
        require!(
            matches!(config.pricing, LaunchPricing::Auction { .. }),
            LaunchError::NotAuction
        );
        let clock = Clock::get()?;
        assert_started(&clock, config.start_time)?;
        require!(amount > 0, CommonError::ConstraintViolation);

        if let Some(root) = config.whitelist_root {
            let bidder = ctx.accounts.bidder.key();
            let leaf = keccak::hashv(&[bidder.as_ref()]).to_bytes();
            let proof_vec = proof.ok_or(LaunchError::WhitelistRequired)?;
            assert_merkle_proof(leaf, &proof_vec, root)?;
        }

        let mut state = ctx.accounts.sale_state.load_mut()?;
        state.record_bid(&ctx.accounts.bidder.key(), amount, config, &clock)?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.bidder_quote_account.to_account_info(),
                    to: ctx.accounts.treasury_vault.to_account_info(),
                    authority: ctx.accounts.bidder.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }

    /// Settles auction after end time.
    pub fn settle_auction(ctx: Context<SettleAuction>) -> Result<()> {
        let config = &mut ctx.accounts.launch_config;
        require!(
            matches!(config.pricing, LaunchPricing::Auction { .. }),
            LaunchError::NotAuction
        );
        let clock = Clock::get()?;
        let mut state = ctx.accounts.sale_state.load_mut()?;
        state.settle_auction(config, &clock)?;
        Ok(())
    }

    /// Withdraws treasury funds to authority-controlled destination.
    pub fn withdraw_treasury(ctx: Context<WithdrawTreasury>, amount: u64) -> Result<()> {
        let config = &ctx.accounts.launch_config;
        assert_signer_is(config.authority(), &ctx.accounts.authority)?;
        let config_key = ctx.accounts.launch_config.key();
        let seeds: &[&[u8]] = &[b"treasury", config_key.as_ref(), &[config.treasury_bump]];
        let binding = [seeds];
        let cpi_accounts = Transfer {
            from: ctx.accounts.treasury_vault.to_account_info(),
            to: ctx.accounts.destination.to_account_info(),
            authority: ctx.accounts.treasury_authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            &binding,
        );
        token::transfer(cpi_ctx, amount)?;
        emit!(TreasuryMovement {
            program: crate::ID,
            entity: ctx.accounts.launch_config.key(),
            amount,
            destination: ctx.accounts.destination.key(),
        });
        Ok(())
    }

    /// Closes state account after sale completion.
    pub fn close(ctx: Context<CloseState>) -> Result<()> {
        let config = &ctx.accounts.launch_config;
        assert_signer_is(config.authority(), &ctx.accounts.authority)?;
        let state = ctx.accounts.sale_state.load()?;
        require!(state.status == SaleStatus::Settled, LaunchError::SaleActive);
        Ok(())
    }
}

/// Initialize launch configuration accounts.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct InitLaunch<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = LaunchConfig::SPACE,
        seeds = [b"launch", mint.key().as_ref()],
        bump
    )]
    pub launch_config: Account<'info, LaunchConfig>,
    #[account(
        mut,
        constraint = treasury_vault.mint == mint.key(),
        constraint = treasury_vault.owner == treasury_authority.key(),
    )]
    pub treasury_vault: Account<'info, TokenAccount>,
    /// CHECK: derived PDA authority for minting/distribution control.
    #[account(
        seeds = [b"treasury", launch_config.key().as_ref()],
        bump
    )]
    pub treasury_authority: UncheckedAccount<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = authority,
        space = 8 + SaleState::LEN,
        seeds = [b"sale", mint.key().as_ref()],
        bump
    )]
    pub sale_state: AccountLoader<'info, SaleState>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

/// Args for launch initialization.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitLaunchArgs {
    /// Launch parameters.
    pub pricing: LaunchPricing,
    /// Global cap across buyers.
    pub global_cap: u64,
    /// Per-wallet cap for presale.
    pub wallet_cap: u64,
    /// Start timestamp.
    pub start_time: i64,
    /// End timestamp.
    pub end_time: i64,
    /// Optional whitelist root.
    pub whitelist_root: Option<[u8; 32]>,
}

/// Update configuration context.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub launch_config: Account<'info, LaunchConfig>,
}

/// Update configuration args.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct UpdateConfigArgs {
    /// Optional override for end time.
    pub end_time: Option<i64>,
    /// Optional per-wallet cap update.
    pub wallet_cap: Option<u64>,
}

/// Purchase context.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut, constraint = quote_account.owner == buyer.key())]
    pub quote_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = buyer_receipt.owner == buyer.key(), constraint = buyer_receipt.mint == mint.key())]
    pub buyer_receipt: Account<'info, TokenAccount>,
    #[account(mut)]
    pub launch_config: Account<'info, LaunchConfig>,
    #[account(
        mut,
        seeds = [b"treasury", launch_config.key().as_ref()],
        bump = launch_config.treasury_bump,
    )]
    /// CHECK: Treasury authority derived PDA.
    pub treasury_authority: UncheckedAccount<'info>,
    #[account(mut, constraint = treasury_vault.owner == treasury_authority.key())]
    pub treasury_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"sale", launch_config.mint.as_ref()],
        bump = launch_config.sale_state_bump,
    )]
    pub sale_state: AccountLoader<'info, SaleState>,
    #[account(constraint = mint.key() == launch_config.mint)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

/// Auction bid context.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct Bid<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,
    #[account(mut, constraint = bidder_quote_account.owner == bidder.key())]
    pub bidder_quote_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub launch_config: Account<'info, LaunchConfig>,
    #[account(
        mut,
        seeds = [b"treasury", launch_config.key().as_ref()],
        bump = launch_config.treasury_bump,
    )]
    /// CHECK: Treasury authority derived PDA.
    pub treasury_authority: UncheckedAccount<'info>,
    #[account(mut, constraint = treasury_vault.owner == treasury_authority.key())]
    pub treasury_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"sale", launch_config.mint.as_ref()],
        bump = launch_config.sale_state_bump,
    )]
    pub sale_state: AccountLoader<'info, SaleState>,
    pub token_program: Program<'info, Token>,
}

/// Sell context for bonding curve exits.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut, constraint = seller_receipt.owner == seller.key(), constraint = seller_receipt.mint == mint.key())]
    pub seller_receipt: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seller_quote_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub launch_config: Account<'info, LaunchConfig>,
    #[account(
        mut,
        seeds = [b"treasury", launch_config.key().as_ref()],
        bump = launch_config.treasury_bump,
    )]
    /// CHECK: Treasury PDA signer.
    pub treasury_authority: UncheckedAccount<'info>,
    #[account(mut, constraint = treasury_vault.owner == treasury_authority.key())]
    pub treasury_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"sale", launch_config.mint.as_ref()],
        bump = launch_config.sale_state_bump,
    )]
    pub sale_state: AccountLoader<'info, SaleState>,
    #[account(constraint = mint.key() == launch_config.mint)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

/// Auction settlement accounts.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct SettleAuction<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub launch_config: Account<'info, LaunchConfig>,
    #[account(
        mut,
        seeds = [b"sale", launch_config.mint.as_ref()],
        bump = launch_config.sale_state_bump,
    )]
    pub sale_state: AccountLoader<'info, SaleState>,
}

/// Treasury withdrawal context.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct WithdrawTreasury<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub launch_config: Account<'info, LaunchConfig>,
    #[account(
        mut,
        seeds = [b"treasury", launch_config.key().as_ref()],
        bump = launch_config.treasury_bump,
    )]
    /// CHECK: PDA authority
    pub treasury_authority: UncheckedAccount<'info>,
    #[account(mut, constraint = treasury_vault.owner == treasury_authority.key())]
    pub treasury_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Close state context.
#[allow(missing_docs)]
#[derive(Accounts)]
pub struct CloseState<'info> {
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority, close = authority)]
    pub launch_config: Account<'info, LaunchConfig>,
    #[account(
        mut,
        seeds = [b"sale", launch_config.mint.as_ref()],
        bump = launch_config.sale_state_bump,
    )]
    pub sale_state: AccountLoader<'info, SaleState>,
}

/// Custom launch errors.
#[error_code]
pub enum LaunchError {
    /// Config already initialized.
    #[msg("Launch configuration already initialized")]
    AlreadyInitialized,
    /// Slippage exceeded user tolerance.
    #[msg("Max quote exceeded user supplied limit")]
    SlippageExceeded,
    /// Sale still active, cannot settle/close.
    #[msg("Sale still active")]
    SaleActive,
    /// Whitelist required but proof missing.
    #[msg("Whitelist proof required")]
    WhitelistRequired,
    /// Instruction only valid for auctions.
    #[msg("Instruction restricted to auction pricing")]
    NotAuction,
    /// Sale already settled and cannot accept new actions.
    #[msg("Sale already settled")]
    SaleSettled,
    /// Purchase would exceed global cap.
    #[msg("Global cap exceeded")]
    CapExceeded,
    /// Wallet-specific cap exceeded.
    #[msg("Wallet cap exceeded")]
    WalletCapExceeded,
    /// Pricing variant requires auction flow.
    #[msg("Place bid through auction instruction")]
    AuctionBidRequired,
    /// Curve variant not yet supported.
    #[msg("Unsupported curve variant")]
    UnsupportedCurve,
    /// Bid too low versus reserve/increment.
    #[msg("Bid amount too low")]
    BidTooLow,
    /// Bid below reserve price.
    #[msg("Bid below reserve price")]
    BidBelowReserve,
    /// Auction still active.
    #[msg("Auction still active")]
    AuctionStillActive,
    /// Auction closed for new bids.
    #[msg("Auction closed")]
    AuctionClosed,
    /// Selling only supported on bonding curves.
    #[msg("Selling only supported for bonding curves")]
    SellOnlyCurve,
    /// Sell amount exceeds recorded holdings.
    #[msg("Sell amount exceeds tracked balance")]
    SellTooLarge,
    /// Seller record not found.
    #[msg("Buyer record not found")]
    UnknownBuyer,
}
