//! State definitions for launchpad program.

use super::LaunchError;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;
use keystone_common::curve::{CurveConfig, CurveKind, LinearCurve, PricingCurve};
use keystone_common::errors::CommonError;

/// Maximum number of buyers tracked for wallet caps in base state.
pub const MAX_TRACKED_BUYERS: usize = 64;

/// Auction configuration covering English & Dutch auctions.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AuctionConfig {
    /// Auction variant.
    pub kind: AuctionKind,
    /// Starting price (scaled decimal based on quote token decimals).
    pub start_price: u64,
    /// Minimum price (for Dutch auctions) or reserve.
    pub floor_price: u64,
    /// Minimum increment in basis points for English auctions.
    pub min_increment_bps: u16,
    /// Anti-sniping extension (in seconds).
    pub anti_snipe_seconds: i64,
}

/// Auction variants supported.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum AuctionKind {
    /// English auction - ascending bids.
    English,
    /// Dutch auction - descending price with clearing condition.
    Dutch,
}

/// Pricing enum representing sale models.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum LaunchPricing {
    /// Fixed price presale/FCFS.
    Fixed { price: u64 },
    /// Bonding curve sale.
    BondingCurve { curve: CurveConfig },
    /// Auction configuration.
    Auction { config: AuctionConfig },
}

impl LaunchPricing {
    /// Quotes price for requested amount.
    pub fn quote_buy(&self, sold: u64, amount: u64) -> Result<u64> {
        match self {
            LaunchPricing::Fixed { price } => sold
                .checked_add(amount)
                .and_then(|_| price.checked_mul(amount))
                .ok_or(CommonError::ArithmeticOverflow.into()),
            LaunchPricing::BondingCurve { curve } => {
                if matches!(curve.kind, CurveKind::Linear) {
                    let curve_calc =
                        LinearCurve::new(curve).map_err(|_| CommonError::ConstraintViolation)?;
                    Ok(curve_calc.quote_buy(sold, amount)?.quote_amount)
                } else {
                    Err(LaunchError::UnsupportedCurve.into())
                }
            }
            LaunchPricing::Auction { .. } => Err(LaunchError::AuctionBidRequired.into()),
        }
    }

    /// Returns auction config if present.
    pub fn auction_config(&self) -> Option<&AuctionConfig> {
        match self {
            LaunchPricing::Auction { config } => Some(config),
            _ => None,
        }
    }

    /// Quotes sell amount for bonding curves.
    pub fn quote_sell(&self, sold: u64, amount: u64) -> Result<u64> {
        match self {
            LaunchPricing::BondingCurve { curve } if matches!(curve.kind, CurveKind::Linear) => {
                let calc = LinearCurve::new(curve).map_err(|_| CommonError::ConstraintViolation)?;
                Ok(calc.quote_sell(sold, amount)?.quote_amount)
            }
            _ => Err(LaunchError::SellOnlyCurve.into()),
        }
    }
}

/// Primary configuration account for a sale.
#[account]
pub struct LaunchConfig {
    /// Authority with rights to manage the sale.
    pub authority: Pubkey,
    /// Treasury vault storing proceeds.
    pub treasury_vault: Pubkey,
    /// Mint being sold.
    pub mint: Pubkey,
    /// Pricing model for the sale.
    pub pricing: LaunchPricing,
    /// Total tokens available for sale.
    pub global_cap: u64,
    /// Per-wallet purchase cap.
    pub wallet_cap: u64,
    /// Sale start time.
    pub start_time: i64,
    /// Sale end time.
    pub end_time: i64,
    /// Optional whitelist root.
    pub whitelist_root: Option<[u8; 32]>,
    /// Bump used for treasury PDA.
    pub treasury_bump: u8,
    /// Authority bump for config PDA.
    pub config_bump: u8,
    /// Sale state bump.
    pub sale_state_bump: u8,
    /// Initialization flag.
    pub initialized: bool,
}

impl LaunchConfig {
    /// Account space including discriminator.
    pub const SPACE: usize = 8 + 512; // generous padding for enum serialization

    /// Creates new configuration from args.
    pub fn try_from_args(
        authority: &Pubkey,
        treasury_vault: &Pubkey,
        mint: &Pubkey,
        args: super::InitLaunchArgs,
    ) -> Result<Self> {
        require!(args.global_cap > 0, CommonError::ConstraintViolation);
        require!(
            args.wallet_cap == 0 || args.wallet_cap <= args.global_cap,
            CommonError::ConstraintViolation
        );
        require!(
            args.end_time > args.start_time,
            CommonError::TimestampInvalid
        );
        Ok(Self {
            authority: *authority,
            treasury_vault: *treasury_vault,
            mint: *mint,
            pricing: args.pricing,
            global_cap: args.global_cap,
            wallet_cap: args.wallet_cap,
            start_time: args.start_time,
            end_time: args.end_time,
            whitelist_root: args.whitelist_root,
            treasury_bump: 0,
            config_bump: 0,
            sale_state_bump: 0,
            initialized: false,
        })
    }

    /// Returns packed PDA seeds for treasury.
    /// Mint accessor.
    pub fn mint(&self) -> &Pubkey {
        &self.mint
    }

    /// Authority accessor for HasAuthority trait.
    pub fn authority(&self) -> &Pubkey {
        &self.authority
    }

    /// Computes configuration hash for audit logging.
    pub fn config_hash(&self) -> [u8; 32] {
        let mut data = Vec::with_capacity(128);
        data.extend_from_slice(self.authority.as_ref());
        data.extend_from_slice(&self.global_cap.to_le_bytes());
        data.extend_from_slice(&self.wallet_cap.to_le_bytes());
        data.extend_from_slice(&self.start_time.to_le_bytes());
        data.extend_from_slice(&self.end_time.to_le_bytes());
        data.extend_from_slice(self.mint.as_ref());
        if let Some(root) = self.whitelist_root {
            data.extend_from_slice(&root);
        }
        data.push(self.treasury_bump);
        data.push(self.sale_state_bump);
        keccak::hash(&data).to_bytes()
    }

    /// Applies update args from authority.
    pub fn update(&mut self, args: super::UpdateConfigArgs) -> Result<()> {
        if let Some(end_time) = args.end_time {
            require!(end_time > self.start_time, CommonError::TimestampInvalid);
            self.end_time = end_time;
        }
        if let Some(wallet_cap) = args.wallet_cap {
            require!(
                wallet_cap == 0 || wallet_cap <= self.global_cap,
                CommonError::ConstraintViolation
            );
            self.wallet_cap = wallet_cap;
        }
        Ok(())
    }
}

impl keystone_common::authority::HasAuthority for LaunchConfig {
    fn authority(&self) -> &Pubkey {
        &self.authority
    }
}

/// Tracks per-wallet purchases to enforce wallet caps.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct BuyerContribution {
    /// Wallet pubkey tracked.
    pub buyer: Pubkey,
    /// Total purchased amount.
    pub purchased: u64,
}

/// Sale lifecycle states.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SaleStatus {
    /// No activity yet.
    Pending,
    /// Sale ongoing.
    Active,
    /// Sale settled/finalized.
    Settled,
}

/// Primary sale state for the launch.
#[account(zero_copy(unsafe))]
#[repr(C)]
pub struct SaleState {
    /// Total tokens sold.
    pub sold: u64,
    /// Total proceeds collected.
    pub proceeds: u64,
    /// Current status of sale.
    pub status: SaleStatus,
    /// Highest bid recorded (auctions).
    pub highest_bid: u64,
    /// Highest bidder.
    pub highest_bidder: Pubkey,
    /// Auction end override (anti-snipe extensions).
    pub auction_end: i64,
    /// Buyer tracker array.
    pub buyers: [BuyerContribution; MAX_TRACKED_BUYERS],
    /// Number of buyer entries used.
    pub buyer_count: u16,
}

impl Default for SaleState {
    fn default() -> Self {
        Self {
            sold: 0,
            proceeds: 0,
            status: SaleStatus::Pending,
            highest_bid: 0,
            highest_bidder: Pubkey::default(),
            auction_end: 0,
            buyers: [BuyerContribution::default(); MAX_TRACKED_BUYERS],
            buyer_count: 0,
        }
    }
}

impl SaleState {
    /// Raw size of zero-copy struct (without discriminator).
    pub const LEN: usize = core::mem::size_of::<SaleState>();

    /// Ensures sale allows requested purchase.
    pub fn assert_allows_purchase(
        &self,
        buyer: &Pubkey,
        amount: u64,
        config: &LaunchConfig,
    ) -> Result<()> {
        require!(self.status != SaleStatus::Settled, LaunchError::SaleSettled);
        let new_sold = self
            .sold
            .checked_add(amount)
            .ok_or(CommonError::ArithmeticOverflow)?;
        require!(new_sold <= config.global_cap, LaunchError::CapExceeded);
        if config.wallet_cap > 0 {
            let existing = self
                .find_buyer(*buyer)
                .map(|(_, contrib)| contrib.purchased)
                .unwrap_or(0);
            let updated = existing
                .checked_add(amount)
                .ok_or(CommonError::ArithmeticOverflow)?;
            require!(updated <= config.wallet_cap, LaunchError::WalletCapExceeded);
        }
        Ok(())
    }

    /// Records purchase, updating per-wallet totals.
    pub fn record_purchase(
        &mut self,
        buyer: &Pubkey,
        amount: u64,
        quote_amount: u64,
        config: &LaunchConfig,
    ) -> Result<()> {
        self.sold = self
            .sold
            .checked_add(amount)
            .ok_or(CommonError::ArithmeticOverflow)?;
        self.proceeds = self
            .proceeds
            .checked_add(quote_amount)
            .ok_or(CommonError::ArithmeticOverflow)?;
        self.upsert_buyer(buyer, amount, config)?;
        if self.status == SaleStatus::Pending {
            self.status = SaleStatus::Active;
        }
        Ok(())
    }

    /// Records a sellback into curve inventory.
    pub fn record_sell(&mut self, buyer: &Pubkey, amount: u64, quote_amount: u64) -> Result<()> {
        self.sold = self
            .sold
            .checked_sub(amount)
            .ok_or(LaunchError::SellTooLarge)?;
        self.proceeds = self
            .proceeds
            .checked_sub(quote_amount)
            .ok_or(CommonError::ArithmeticOverflow)?;
        if let Some((idx, entry)) = self.find_buyer(*buyer) {
            let new_total = entry
                .purchased
                .checked_sub(amount)
                .ok_or(LaunchError::SellTooLarge)?;
            self.buyers[idx].purchased = new_total;
        } else {
            return Err(LaunchError::UnknownBuyer.into());
        }
        Ok(())
    }

    /// Settlement handler for auctions.
    pub fn settle_auction(&mut self, config: &LaunchConfig, clock: &Clock) -> Result<()> {
        require!(self.status != SaleStatus::Settled, LaunchError::SaleSettled);
        let auction = config
            .pricing
            .auction_config()
            .ok_or(LaunchError::NotAuction)?;
        let end = self.current_auction_end(config);
        require!(clock.unix_timestamp >= end, LaunchError::AuctionStillActive);
        if matches!(auction.kind, AuctionKind::Dutch) {
            self.sold = config.global_cap;
        }
        self.status = SaleStatus::Settled;
        Ok(())
    }

    /// Returns current auction end including extensions.
    pub fn current_auction_end(&self, config: &LaunchConfig) -> i64 {
        if self.auction_end == 0 {
            config.end_time
        } else {
            self.auction_end
        }
    }

    /// Records auction bid ensuring increment rules.
    pub fn record_bid(
        &mut self,
        bidder: &Pubkey,
        bid_amount: u64,
        config: &LaunchConfig,
        clock: &Clock,
    ) -> Result<()> {
        let auction = config
            .pricing
            .auction_config()
            .ok_or(LaunchError::NotAuction)?;
        require!(
            bid_amount >= auction.floor_price,
            LaunchError::BidBelowReserve
        );
        require!(
            clock.unix_timestamp < self.current_auction_end(config),
            LaunchError::AuctionClosed
        );
        if self.highest_bid > 0 {
            let min_increment = self
                .highest_bid
                .checked_mul(auction.min_increment_bps as u64)
                .ok_or(CommonError::ArithmeticOverflow)?
                .checked_div(10_000)
                .ok_or(CommonError::ArithmeticOverflow)?;
            let required = self
                .highest_bid
                .checked_add(min_increment.max(1))
                .ok_or(CommonError::ArithmeticOverflow)?;
            require!(bid_amount >= required, LaunchError::BidTooLow);
        }
        self.highest_bid = bid_amount;
        self.highest_bidder = *bidder;
        self.proceeds = bid_amount;
        if self.status == SaleStatus::Pending {
            self.status = SaleStatus::Active;
        }
        if matches!(auction.kind, AuctionKind::English) {
            let end = self.current_auction_end(config);
            let remaining = end.saturating_sub(clock.unix_timestamp);
            if remaining <= auction.anti_snipe_seconds {
                self.auction_end = clock
                    .unix_timestamp
                    .checked_add(auction.anti_snipe_seconds)
                    .ok_or(CommonError::ArithmeticOverflow)?;
            }
        }
        Ok(())
    }

    fn find_buyer(&self, buyer: Pubkey) -> Option<(usize, &BuyerContribution)> {
        (0..self.buyer_count as usize)
            .find(|&i| self.buyers[i].buyer == buyer)
            .map(|idx| (idx, &self.buyers[idx]))
    }

    fn upsert_buyer(&mut self, buyer: &Pubkey, amount: u64, config: &LaunchConfig) -> Result<()> {
        if config.wallet_cap == 0 {
            return Ok(());
        }
        if let Some((idx, entry)) = self.find_buyer(*buyer) {
            let new_total = entry
                .purchased
                .checked_add(amount)
                .ok_or(CommonError::ArithmeticOverflow)?;
            self.buyers[idx].purchased = new_total;
            return Ok(());
        }
        let idx = self.buyer_count as usize;
        require!(
            idx < MAX_TRACKED_BUYERS,
            SaleStateError::BuyerCapacityExceeded
        );
        self.buyers[idx] = BuyerContribution {
            buyer: *buyer,
            purchased: amount,
        };
        self.buyer_count = self
            .buyer_count
            .checked_add(1)
            .ok_or(CommonError::ArithmeticOverflow)?;
        Ok(())
    }
}

/// Custom sale errors.
#[error_code]
pub enum SaleStateError {
    /// Buyer tracking is full.
    #[msg("Buyer tracking capacity exceeded")]
    BuyerCapacityExceeded,
}

#[cfg(all(test, not(target_arch = "bpf")))]
mod tests {
    use super::*;

    #[test]
    fn enforce_wallet_cap() {
        let config = LaunchConfig {
            authority: Pubkey::new_unique(),
            treasury_vault: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            pricing: LaunchPricing::Fixed {
                price: 1_000_000_000,
            },
            global_cap: 100,
            wallet_cap: 10,
            start_time: 0,
            end_time: 10,
            whitelist_root: None,
            treasury_bump: 0,
            config_bump: 0,
            sale_state_bump: 0,
            initialized: true,
        };
        let mut state = SaleState::default();
        let buyer = Pubkey::new_unique();
        state.assert_allows_purchase(&buyer, 5, &config).unwrap();
        state.record_purchase(&buyer, 5, 5, &config).unwrap();
        assert!(state.assert_allows_purchase(&buyer, 6, &config).is_err());
    }
}
