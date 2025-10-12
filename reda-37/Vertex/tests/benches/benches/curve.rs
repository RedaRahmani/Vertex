use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use keystone_common::curve::{CurveConfig, CurveKind, LinearCurve, PricingCurve};
use keystone_common::fees::FeeConfig;
use solana_program::pubkey::Pubkey;

fn bench_linear_curve(c: &mut Criterion) {
    let cfg = CurveConfig {
        kind: CurveKind::Linear,
        base_price: 1_000_000_000,
        k: 5_000_000,
        x0: 0,
        max_supply: 1_000_000,
        fee_config: FeeConfig::new(30, 10_000, Pubkey::default()).unwrap(),
    };
    let curve = LinearCurve::new(&cfg).unwrap();
    c.bench_function("linear_quote_buy", |b| {
        b.iter_batched(
            || (curve, 50_000u64, 1_000u64),
            |(curve, supply, amount)| curve.quote_buy(supply, amount).unwrap(),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_linear_curve);
criterion_main!(benches);
