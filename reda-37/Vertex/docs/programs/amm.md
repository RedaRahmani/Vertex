# AMM Program

The constant-product AMM (`amm_cp`) is a v1 implementation with protocol fees and LP tokenization.

## Highlights

- Deterministic PDA signer `(b"pool", pool.key())` secures vault withdrawals and LP mint authority.
- Fees defined via numerator/denominator pairs with a hard cap of 50%.
- Swap math uses 128-bit checked arithmetic and deducts protocol fee before computing output amount.
- Criterion benchmarks cover hot paths and fuzzing ensures invariant preservation.

## Instructions

- `init_pool`: Seeds pool account and signer PDA, sets fee vault.
- `add_liquidity` / `remove_liquidity`: Transfers tokens and mints/burns LP with proportional reserves.
- `swap`: Executes token swap with slippage guard enforced client-side.
- `collect_fees`: Moves accumulated protocol fees to treasury-controlled account.

Integrations should use the Rust or TypeScript SDK helpers to build instructions and maintain consistent math.
