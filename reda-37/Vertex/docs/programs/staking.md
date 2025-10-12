# Staking Program

Stake pools manage locked staking positions and linear emissions per slot.

## Components

- `Pool` – Stores authority, reward mint, emission rate, and lock policy.
- `UserStake` – Zero-copy user record keyed by `(pool, staker)` storing staked amount and last claim slot.
- `stake_vault` / `reward_vault` – SPL Token accounts controlled by vault authority PDA `(b"vault", pool)`.

## Lock Policies

- `None`: Free entry/exit with reward accrual.
- `Linear`: Unlocks linearly over `[start_slot, end_slot]`.
- `Cliff`: Unlocks entirely at `release_slot`.

## Instruction Summary

- `init_pool`: Configures vaults and policies.
- `stake`: Transfers user tokens into the stake vault, initializing `UserStake`.
- `claim`: Sends rewards accumulated since last claim, using per-slot emission.
- `unstake`: Validates lock policy before returning staked amount.
- `admin_update`: Authority-only update to emission rate or lock policy.

Reward calculations happen entirely on-chain using saturating math. Tests cover lock enforcement and emission accuracy.
