# Vertex UI (Fee Router)

Minimal Next.js app for initializing policy, creating honorary position, and running the daily crank for the Keystone Fee Router.

- Paste the program ID and IDL JSON into forms.
- For devnet/localnet, set `NEXT_PUBLIC_RPC_URL` or edit form fields inline.

## Getting started

- Install deps: `pnpm i` (from repo root or `apps/ui`)
- Dev: `pnpm -F apps/ui dev`

Pages:
- `/policy-setup` – initialize `Policy` PDA
- `/honorary-position` – create `HonoraryPosition`
- `/daily-crank` – run crank with one investor per page

Note: This UI uses a mock Streamflow adapter (first 8 bytes of the stream account data as u64). Replace with real adapter when available.
