# Keystone Fee Router

Permissionless fee routing from Meteora DLMM v2 (DAMM) pool quote fees to Streamflow‑locked investors once per 24h UTC window. Deterministic PDAs, strict constraints, and u128 checked math. Each program keeps its own ID and IDL.

Meteora DLMM v2 program ID (devnet+mainnet): `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG`.

## PDAs & Seeds
- `Policy` PDA: seeds `[b"policy", cp_pool]`.
- `Progress` PDA: seeds `[b"progress", cp_pool]`.
- Vault authority PDA (no account): seeds `[b"vault", policy]`.
- Honorary position registry: `[b"position", policy]`.
- Honorary position owner PDA: `[b"vault", policy, b"investor_fee_pos_owner"]`.

## Accounts (state)
- `Policy` (SPACE = `8 + size_of::<Policy>()`):
  - `authority: Pubkey`
  - `cp_pool: Pubkey`
  - `quote_mint: Pubkey`
  - `creator_quote_ata: Pubkey`
  - `treasury_quote_ata: Pubkey`
  - `investor_fee_share_bps: u16`
  - `y0_total: u64`
  - `daily_cap_quote: u64`
  - `min_payout_lamports: u64`
  - `bump: u8`
  - `initialized: bool`

- `Progress` (SPACE = `8 + size_of::<Progress>()`):
  - `current_day: i64`, `last_distribution_ts: i64`
  - `claimed_quote_today: u64`, `distributed_quote_today: u64`, `carry_quote_today: u64`
  - `page_cursor: u64`, `day_closed: bool`, `bump: u8`

- `HonoraryPosition` (SPACE = `8 + size_of::<HonoraryPosition>()`):
  - `owner_pda: Pubkey`, `position: Pubkey`, `cp_pool: Pubkey`, `quote_mint: Pubkey`, `bump: u8`

## Events
- `PolicyInitialized { policy, config_hash }`
- `HonoraryPositionInitialized { pool, position, owner_pda }`
- `InvestorPayoutPage { day, page_cursor, investors, paid_total, carry_after }`
- `CreatorPayoutDayClosed { day, remainder }`

## Errors
`QuoteOnlyViolation`, `DailyWindowNotReady`, `InvalidInvestorPage`, `CapExceeded`, `ArithmeticOverflow`, `ConstraintViolation`, `Unauthorized`.

## Instruction Semantics
- `init_policy`:
  - Validates: `bps <= 10_000`, `y0_total > 0`, ATAs use `quote_mint`, `treasury_quote_ata.owner == vault_authority` PDA.
  - Persists bumps and marks `initialized = true`.
  - Emits `PolicyInitialized` with a keccak config hash.

- `init_honorary_position`:
  - Binds `cp_pool`, `quote_mint`, `position` to vault PDA; enforces pool owner == Meteora DLMM v2.
  - Emits `HonoraryPositionInitialized`.

- `crank_distribute`:
  - Enforces UTC day window via `floor(ts/86400)`; rollover requires previous `day_closed == true`.
  - Collects Meteora quote fees (CPI stub; tests pre‑mint). Distributable clamped by cap and the current treasury balance. 
  - Reads still‑locked amounts via pluggable Streamflow adapter; computes `eligible_bps = min(policy_bps, locked * 10_000 / y0_total)`.
  - Pays only if `share >= min_payout_lamports`; otherwise dust carries within day.
  - Last page: routes remainder to `creator_quote_ata`, marks `day_closed = true` and emits `CreatorPayoutDayClosed`.

## Streamflow Adapter (pluggable)
- Trait `StreamLockedReader` is used by the program. Default mock reads first 8 bytes as `u64`.
- Feature `streamflow` includes a skeleton for the real layout (unimplemented, failing closed).

## BPF Safety Notes
- No on‑chain stable sort; use `programs/common/src/bpf_sort.rs` if needed.
- Small stack frames; u128 checked math; events for audit.

## Build & Run
- Build programs: `anchor build`
- Run tests (host + program‑test): `anchor test`
- UI dev (after installing deps): `pnpm -F apps/ui dev`
