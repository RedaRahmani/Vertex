# Keystone Vertex – Meteora DAMM v2 Honorary Fee Router

Star’s mission is to make fund‑raising on-chain feel like “Twitch × Kickstarter × NASDAQ.”  
This repository delivers the module we built for the Star bounty: an Anchor compatible
fee router that:

1. Opens a Meteora DLMM v2 honorary LP position owned by our PDA and guaranteed to accrue **quote-only** fees.
2. Provides a once‑per‑day, permissionless crank that claims those fees, pays investors pro‑rata to still-locked Streamflow balances, and routes the carry remainder to the creator.

We packaged the on-chain program, Rust/TypeScript SDKs, a CLI, and a reference Next.js UI so the Star team can drop it directly into their stack.

---

## Repository Layout

```
programs/fee_router      Anchor program for honorary position + distribution crank
apps/ui-temp             Reference Next.js UI (policy setup, honorary position, daily crank)
sdk/rust, sdk/ts         Instruction builders and account helpers
scripts/                 Local validator automation and fixtures
audits/                  Threat model + security checklist
docs/                    MkDocs documentation site
```

---

## Running the Demo Locally

These steps produce the exact flow shown in the screenshots below. Everything runs on a local validator; Phantom (or any Solana wallet) must be set to **Localnet** and connected to `localhost:3000`.

```bash
# 1. Start a clean validator
solana-test-validator

# in a new shell: build + deploy the fee router program
anchor build -p keystone_fee_router
anchor deploy -p keystone_fee_router

# 2. Seed deterministic fixture accounts (quote mint, cp_pool, honorary position PDA seeds)
node scripts/setup-local-fixture.js

# 3. Launch the reference UI
npm --prefix apps/ui-temp run dev
```

Every fixture run rewrites `target/local-fixture.json` and `apps/ui-temp/app/config.ts` with the program ID, Meteora pool, quote mint, PDAs, and ATAs that the UI pre-fills.

---

## UI Walkthrough

> **Where do I find the Program ID and IDL?**  
Both values are copied by `scripts/setup-local-fixture.js`. The script writes:
>
> - Program ID: `LOCAL_FIXTURE.programId`
> - Full IDL JSON: `apps/ui-temp/app/idl.ts`  
> - All PDAs / ATAs needed for the flow: `target/local-fixture.json`

### 1. Dashboard – Environment & State Inspection

![Dashboard](docs/assets/ui-dashboard.png)

<img width="1449" height="591" alt="image" src="https://github.com/user-attachments/assets/08bab249-cbb6-45d0-a9dd-2497ed3d248a" />


1. Paste the Program ID and IDL JSON recorded in `target/local-fixture.json`.  
   (The UI caches these in `localStorage` after you press **Save**.)
2. Enter the Meteora `cp_pool` from the same fixture file and press **Fetch State**.  
   Before the first crank runs you’ll see a policy summary plus a note indicating the Progress account will appear after the initial distribution.

### 2. Policy Setup – Initialize the Daily Policy

![Policy Setup](docs/assets/ui-policy.png)

<img width="1123" height="868" alt="image" src="https://github.com/user-attachments/assets/323ea0e7-a00d-4896-9834-07dfda2db71c" />

All inputs are pre-populated from the fixture:

- `Meteora cp_pool` – `LOCAL_FIXTURE.cpPool`
- `Quote mint` – `LOCAL_FIXTURE.quoteMint`
- `Creator quote ATA` – `LOCAL_FIXTURE.creatorAta`
- `Treasury quote ATA` – `LOCAL_FIXTURE.treasuryAta`
- Economic knobs (`Y0`, `bps`, cap, min payout)

Click **Initialize Policy** and approve the Phantom prompt. The transaction signature is displayed on success (e.g. `Init Policy tx: ...`). This stores the immutable configuration and the PDAs are displayed above the button for reference.

### 3. Honorary Position – Bind the Meteora LP

![Honorary Position](docs/assets/ui-honorary.png)

<img width="1207" height="852" alt="image" src="https://github.com/user-attachments/assets/238089d0-4ead-410e-a098-be38e133b7af" />

Again, values are filled automatically:

- `Policy PDA` – `LOCAL_FIXTURE.policy`
- `Meteora cp_pool` – `LOCAL_FIXTURE.cpPool`
- `Quote mint` – `LOCAL_FIXTURE.quoteMint`
- `cp-position account` – `LOCAL_FIXTURE.cpPosition`

Press **Initialize Honorary Position** to create the position PDA and tie it to the DLMM owner PDA. The UI echoes both the Honorary Position PDA and the fee-owner PDA derived from `[VAULT_SEED, policy, investor_fee_pos_owner]`.

### 4. Daily Crank – Quote Fee Distribution

![Daily Crank](docs/assets/ui-crank.png)

<img width="1243" height="876" alt="image" src="https://github.com/user-attachments/assets/56593374-e7b3-4ef4-9875-f3cad8fe510e" />


This form requires both static and per-investor values:

| Input                         | Default Source                                      | Notes                                                                                         |
|-------------------------------|-----------------------------------------------------|-----------------------------------------------------------------------------------------------|
| Policy PDA                    | `LOCAL_FIXTURE.policy`                              | Static                                                                                         |
| Meteora cp_pool               | `LOCAL_FIXTURE.cpPool`                              | Static                                                                                         |
| Treasury quote ATA            | `LOCAL_FIXTURE.treasuryAta`                         | Static, owned by the vault PDA                                                                 |
| Creator quote ATA             | `LOCAL_FIXTURE.creatorAta`                          | Static, receives end-of-day remainder                                                          |
| Investor quote ATA            | *(must supply per investor page)*                   | Streamflow-provided token account, same quote mint                                             |
| Stream account                | *(must supply per investor page)*                   | Streamflow stream PDA; fee router reads `locked` at crank time                                 |
| Page cursor (u64)             | Free-form                                           | Caller-chosen opaque value for pagination bookkeeping                                          |
| Carry cursor (u64)            | Free-form                                           | We reuse the fixture default `0`; the crank stores it in the Progress account                  |
| “Is last page?” checkbox      | Off by default                                      | Toggle **after** you submit the final investor page of the UTC day (routes remainder to creator) |

For real payout pages you’ll pass the Streamflow stream and investor ATA for each investor. The UI accepts additional investor entries (ATA + stream) via the “Additional Investors” section (not shown in the screenshot) to cover multi-investor pages.

### 5. Dashboard – Fetch Policy State

![Fetch State](docs/assets/ui-dashboard-policy.png)

<img width="1139" height="671" alt="image" src="https://github.com/user-attachments/assets/2c37484e-76ef-4a3a-b196-70702406a248" />


After the policy is initialized, **Fetch State** displays:

- Policy PDA and derived Progress PDA
- Immutable economic configuration
- Quote mint, Y0 total, fee share, caps, dust threshold
- Creator and treasury ATAs

Once you run the crank for the first time, the Progress card is populated with:

- Current UTC day (`floor(ts/86400)`)
- Last crank timestamp
- Claimed/distributed quote totals
- Carry-over, cursor, and `day_closed` flag

---

## Bounty Requirements Checklist

| Requirement                                           | Implementation Reference                                                                    |
|-------------------------------------------------------|----------------------------------------------------------------------------------------------|
| Honorary position owned by PDA, quote-only fees       | `programs/fee_router/src/lib.rs::init_honorary_position` (quote mint validation + Meteora CPI guard) |
| 24h distribution crank with pagination                | `programs/fee_router/src/lib.rs::crank_distribute`                                           |
| Idempotent daily tracking (Progress account)          | `Progress` account schema (`current_day`, `page_cursor`, `carry_quote_today`, `day_closed`)   |
| Per-investor pro-rata math with caps & dust           | `compute_investor_quote` and distribution loop                                               |
| Remainder routed to creator when `is_last_page`       | Final branch in `crank_distribute`                                                           |
| Quote-only guard (base fees rejected)                 | `assert_cp_pool_quote_only` + Meteora CPI result checks                                      |
| Streamflow integration                                | `stream_adapter::StreamLockedReader` (mock adapter)                                          |
| Events emitted                                        | `PolicyInitialized`, `HonoraryPositionInitialized`, `InvestorPayoutPage`, `CreatorPayoutDayClosed` |

Tests and scripts that exercise the happy path live under `tests/`, `scripts/`, and the Next.js UI.

---

## Security Notes

- All PDAs store bumps on-chain to prevent accidental drift.
- Honorary position initialization validates pool mint ordering and quote-only accrual.
- Crank enforces 24h gating, handles retries safely, and never double-pays.
- Arithmetic uses checked 128-bit intermediates to avoid overflow.

See [`audits/threat_model.md`](audits/threat_model.md) and [`audits/checklist.md`](audits/checklist.md) before mainnet deployment.

---

## Documentation & Tooling

- Rust + TypeScript SDKs: instruction builders and account parsers for integrating the module into Star’s backend.
- CLI (`cli/`): operational commands for deployments and configuration.
- Docs (MkDocs) can be served locally:

  ```bash
  pip install mkdocs-material
  mkdocs serve
  ```

---

With this module, Star can spin up a Meteora honorary LP, guarantee quote‑only fees, and stream investor payouts in under a minute—no bespoke scaffolding required. Let’s win that bounty. 🚀
