# Launchpad Program

The Launchpad program orchestrates token sales with presale, FCFS, whitelist, auction, and bonding-curve models. All proceeds flow to a treasury PDA derived from the launch configuration, eliminating discretionary authorities.

## Accounts

- `LaunchConfig` – Configuration PDA derived from `(b"launch", mint)` and stores sale details, whitelist root, and PDA bumps.
- `SaleState` – Zero-copy PDA `(b"sale", mint)` tracking totals, per-wallet consumption, and auction state.
- `Treasury Vault` – SPL Token account owned by the treasury PDA; receives sale proceeds.

## Instructions

| Instruction | Notes |
| --- | --- |
| `init_launch` | Initializes config + sale state. Requires treasury vault and mint authorities to be prepared beforehand. |
| `update_config` | Authority-only update for cap or end time extension. |
| `buy` | Executes presale/FCFS/curve purchases with whitelist checks and wallet cap enforcement. |
| `bid` | Places an auction bid, storing highest bid and applying anti-snipe windows. |
| `settle_auction` | Finalizes auctions once the (possibly extended) end time elapses. |
| `withdraw_treasury` | Authority-only transfer of proceeds to downstream accounts. |
| `close` | Closes config + state once settlement completes. |

## Security Notes

- PDA bumps stored on-chain ensure deterministic authority seeds.
- Per-wallet tracking is bounded to prevent unbounded memory growth (default 64 entries, adjustable for production).
- Whitelists use Keccak Merkle roots; CLI provides proof generator.
- Auctions use basis-point increments and anti-snipe logic to reduce MEV.

## Events

- `TreasuryMovement` emitted for all treasury transfers, enabling indexer pipelines.
- `ConfigUpdated` emitted on every configuration change with hashed payload.

Refer to [Threat Model](../security.md) for more extensive analysis.
