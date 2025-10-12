use keystone_common::curve::{CurveConfig, CurveKind, LinearCurve, PricingCurve};
use keystone_common::fees::FeeConfig;
use proptest::prelude::*;
use solana_program::pubkey::Pubkey;

proptest! {
    #[test]
    fn quote_buy_always_positive(base_price in 1_000_000_000u64..10_000_000_000u64, amount in 1u64..1_000u64) {
        let config = CurveConfig {
            kind: CurveKind::Linear,
            base_price,
            k: 10_000,
            x0: 0,
            max_supply: 1_000_000,
            fee_config: FeeConfig::new(50, 10_000, Pubkey::default()).unwrap(),
        };
        let curve = LinearCurve::new(&config).unwrap();
        let quote = curve.quote_buy(10_000, amount).unwrap();
        prop_assert!(quote.quote_amount > 0);
    }
}
