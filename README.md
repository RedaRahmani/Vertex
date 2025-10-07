# Keystone Vertex

Keystone Vertex is a production-oriented Solana program suite for token launches, automated market making, staking, and vesting. The repository ships with Anchor programs, SDKs (Rust + TypeScript), a CLI, reference UI, fuzz/bench harnesses, and a mkdocs documentation site.

## Repository Layout

```
programs/         # Anchor on-chain logic (launchpad, AMM, staking, vesting)
sdk/rust          # Rust instruction builders and program wrappers
sdk/ts            # TypeScript helpers (Anchor compatible)
cli               # `star` CLI for deployments and ops
app-demo/web      # Next.js demo for sanity checks
scripts           # Localnet/devnet automation
audits            # Threat model and security checklist
docs              # mkdocs documentation site
```

## Quickstart

```bash
# toolchains
rustup component add clippy rustfmt
cargo install --locked anchor-cli

# install JS deps
pnpm install --prefix sdk/ts
pnpm install --prefix cli
pnpm install --prefix app-demo/web

# build and test
anchor build
cargo test
pnpm --prefix sdk/ts run test
```

## Fee Router Demo (Meteora DLMM v2 + Streamflow mock)

- Deploy local validator and programs, then run a demo: `make demo`
- Run the acceptance gate (build, scan logs, IDL/binary presence): `make gate`
- Next.js UI for the fee router (apps/ui): `make ui`

Notes
- Fee Router program lives at `programs/fee_router` and has its own IDL and Program ID.
- Meteora DLMM program id is configurable via accounts/clients; a known ID is documented but not hardcoded in logic.
- Streamflow adapter is pluggable; the default mock reads the first 8 bytes of the stream account as a u64 still-locked amount.

## Security Controls

- PDA bumps persisted on-chain and reused for signer derivations.
- Merkle proofs enforced for whitelist sales.
- Auctions store highest bid and anti-snipe extension to reduce MEV.
- AMM math uses checked 128-bit arithmetic and fuzz testing.
- Staking lock policies validated before unstake transfers.
- Vesting enforces chronological schedules and supports revocation toggles.

Refer to [`audits/threat_model.md`](audits/threat_model.md) and [`audits/checklist.md`](audits/checklist.md) before deploying.

## Documentation

Docs live under `docs/` and can be served locally:

```bash
pip install mkdocs-material
mkdocs serve
```

## Happy Path Demo

See [docs/operations/deployments.md](docs/operations/deployments.md) for a scripted run that covers:
1. Launch configuration initialization and whitelist sale.
2. Auction bid + settlement.
3. AMM pool creation with swap.
4. Staking stake/claim/unstake cycle.
5. Vesting schedule create/claim.
