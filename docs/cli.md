# CLI Usage

Install dependencies then link locally:

```bash
pnpm install --prefix cli
pnpm --prefix cli run build
pnpm --prefix cli link --global
```

## Commands

- `star init` – Scaffold workspace configuration files and environment variables.
- `star launch create` – Ingest a JSON config file, derive PDAs, and send `init_launch`.
- `star launch buy` – Builds purchase instruction enforcing max quote and optional whitelist proof.
- `star status` – Aggregates program metadata (config hashes, treasury balances).

Refer to `star --help` for additional options including cluster overrides and dry-run mode.
