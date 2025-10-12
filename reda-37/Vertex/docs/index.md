# Keystone Vertex

Keystone Vertex is a modular Solana program suite for token launches, automated market making, staking, and vesting. The suite emphasizes auditable flows, deterministic pricing, and secure defaults suitable for production token launches.

## Features

- **Launchpad** – presale, FCFS, whitelist, auctions, and bonding curves with PDA-enforced treasuries.
- **AMM** – constant-product pools with protocol fees and routing hooks.
- **Staking** – emission schedules, lock policies, and multisig-controlled updates.
- **Vesting** – linear or cliff unlocks with optional Merkle-gated claims.
- **SDKs** – Rust and TypeScript clients plus CLI automation.
- **Docs & Tests** – mkdocs site, threat model, fuzzing, and Criterion benchmarks.

## Getting Started

```bash
# install toolchains
rustup component add rustfmt clippy
cargo install --locked anchor-cli

# bootstrap workspace
pnpm install --prefix sdk/ts
pnpm install --prefix cli
pnpm install --prefix app-demo/web

# build programs
anchor build
```

Refer to [Operations](operations/deployments.md) for deployment flow and [CLI](cli.md) for scripted workflows.
