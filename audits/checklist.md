# Keystone Vertex Security Checklist

## General
- [ ] `solana_security_txt!` present in every program.
- [ ] All programs compiled with `cargo build-sbf -- --features no-entrypoint` and `deny(warnings)`.
- [ ] PDAs documented with seeds and bumps persisted on-chain.

## Launchpad
- [ ] `launch_config` PDA derived from `("launch", mint)` with bump stored.
- [ ] `sale_state` zero-copy account created with deterministic seeds `( "sale", mint )`.
- [ ] Per-wallet caps enforced; buyer tracking capacity reviewed for expected participants.
- [ ] Merkle proofs verified for whitelist modes.
- [ ] Auctions enforce min increment and anti-snipe extension.
- [ ] Treasury withdrawals require authority signer and PDA seeds with stored bump.

## AMM
- [ ] Constant-product swap uses checked math with fee deduction prior to calculating output.
- [ ] LP mint authority owned by pool signer PDA.
- [ ] Fees withdrawn only via `collect_fees` with event emission.

## Staking
- [ ] Lock policy validated before unstake.
- [ ] Reward vault only spendable by vault authority PDA.
- [ ] Emission updates restricted to pool authority.

## Vesting
- [ ] Vesting schedule stores `start`, `cliff`, `end`, and prevents inverted timelines.
- [ ] Claims limited to vested - claimed balance.
- [ ] Revocation guarded by `revocable` flag and returns funds to authority vault.

## Tooling & Ops
- [ ] `anchor test` passes with coverage ≥ 85% for core crates.
- [ ] `pnpm test:e2e` covers JS/TS flows (local validator).
- [ ] `scripts/localnet.sh` verified from clean checkout.
- [ ] Threat model reviewed quarterly.
