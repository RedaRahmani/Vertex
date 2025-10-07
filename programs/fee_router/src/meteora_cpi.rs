//! Minimal Meteora DLMM v2 CPI helpers and pool checks.
//! Program ID is pluggable; clients should pass it in accounts/args.
//! Default known id is provided for convenience but not enforced on-chain.

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

/// Known Meteora DLMM v2 program id (devnet+mainnet). Not enforced on-chain.
pub const DEFAULT_DLMM_PROGRAM_ID: &str = "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG";

/// Minimal account bundle for future Meteora fee collection CPI wiring.
#[allow(dead_code)]
pub struct CollectQuoteFeesAccounts<'info, 'a> {
    /// Meteora DLMM pool account (unchecked; ownership validated at runtime).
    pub pool_ai: &'a UncheckedAccount<'info>,
    /// Vault authority PDA that signs for treasury transfers.
    pub vault_authority_pda: &'a UncheckedAccount<'info>,
    /// Treasury quote token account owned by the vault authority.
    pub treasury_quote_ata: &'a TokenAccount,
    /// SPL token program.
    pub token_program: &'a Program<'info, Token>,
    /// Meteora DLMM program account.
    pub meteora_program: &'a UncheckedAccount<'info>,
}

/// Collect quote-side fees from the Meteora DLMM pool into the treasury.
///
/// MVP scaffolding: only enforces that the provided pool is owned by the supplied
/// Meteora program account. The real CPI will be wired once integration details
/// are finalized.
pub fn collect_quote_fees(
    pool_ai: &UncheckedAccount,
    _vault_authority_pda: &UncheckedAccount,
    _treasury_quote_ata: &TokenAccount,
    _token_program: &Program<Token>,
    meteora_program: &UncheckedAccount,
) -> Result<()> {
    require_keys_eq!(
        *pool_ai.owner,
        meteora_program.key(),
        crate::FeeRouterError::QuoteOnlyViolation
    );
    Ok(())
}
