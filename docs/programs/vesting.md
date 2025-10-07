# Vesting Program

Linear or cliff-based vesting schedules manage beneficiary payouts with optional revocation and whitelist support.

## Schedule Lifecycle

1. Authority calls `create_schedule` providing schedule parameters and funding vault.
2. Beneficiary calls `claim` with optional Merkle proof to receive vested tokens.
3. Authority may call `revoke` (if allowed) to retrieve unvested balance.

## Security

- Ensures `start < cliff <= end` and positive totals.
- Claims limited to `vested - claimed` using integer math.
- Merkle proofs prevent unauthorized batch claims.
- Vault authority PDA `(b"vault", schedule)` signs all transfers.

Client SDK utilities provide helpers for instruction building and proof serialization.
