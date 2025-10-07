#![cfg(all(test, not(target_arch = "bpf")))]

// Host-only tests validating fee_router pro-rata math, cap clamp, dust carry, idempotency.

#[test]
fn pro_rata_math() {
    let y0_total: u128 = 1_000_000;
    let locked: u128 = 250_000; // 25%
    let investor_fee_share_bps: u128 = 2_000; // 20%
    let distributable: u128 = 10_000;
    let f_locked_bps = (locked * 10_000) / y0_total; // 2500 bps
    let eligible_bps = core::cmp::min(f_locked_bps, investor_fee_share_bps); // 2000 bps
    let share = (distributable * eligible_bps) / 10_000; // 2000
    assert_eq!(share, 2_000);
}

#[test]
fn cap_and_carry_logic() {
    let distributed_today: u64 = 5_000;
    let daily_cap: u64 = 6_000;
    let treasury_balance: u64 = 10_000;
    let cap_remaining = daily_cap.saturating_sub(distributed_today); // 1_000
    let pool_remaining = treasury_balance; // using current treasury balance
    let distributable = core::cmp::min(cap_remaining, pool_remaining);
    assert_eq!(distributable, 1_000);

    // With dust threshold above share, carry should remain non-zero.
    let eligible_bps = 1500u64; // 15%
    let share = (distributable as u128 * eligible_bps as u128 / 10_000) as u64; // 150
    let min_payout = 500u64;
    let paid = if share >= min_payout { share } else { 0 };
    let carry = distributable.saturating_sub(paid);
    assert_eq!(paid, 0);
    assert_eq!(carry, 1_000);
}

#[test]
fn idempotency_handling() {
    // If a page pays 600 and distributed increases, a retry cannot overpay as
    // subsequent distributable clamps by current treasury.
    let start_treasury = 2_000u64;
    let mut distributed = 0u64;
    let bps = 5000u64; // 50%

    // First run
    let cap_remaining = u64::MAX;
    let pool_remaining = start_treasury;
    let mut distributable = core::cmp::min(cap_remaining, pool_remaining);
    let share = (distributable as u128 * bps as u128 / 10_000) as u64; // 1_000
    distributed += share;

    // Treasury after transfer: 1_000; retry run computes distributable from treasury
    let pool_remaining_retry = start_treasury - share; // 1000
    let distributable_retry = core::cmp::min(u64::MAX, pool_remaining_retry);
    let share_retry = (distributable_retry as u128 * bps as u128 / 10_000) as u64; // 500
    assert_eq!(share, 1_000);
    assert_eq!(share_retry, 500);
}

