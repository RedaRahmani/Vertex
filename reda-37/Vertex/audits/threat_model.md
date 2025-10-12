# Keystone Vertex Threat Model

## Overview
Keystone Vertex provides token launch, AMM, staking, and vesting programs on Solana. The primary security objective is to prevent unauthorized fund movement, ensure pricing integrity, and maintain deterministic, auditable flows for token distributions.

## Assets
- Token sale proceeds in treasury vaults (SPL Token accounts)
- Liquidity pool reserves and LP mint authority
- Staked funds and reward vaults
- Vesting vaults escrowed for beneficiaries
- Program-derived authorities (PDAs) controlling mints and vaults

## Trust Assumptions
- Program upgrade authority controlled by a multi-signature held by Keystone Labs.
- Off-chain whitelist generation and Merkle proofs are produced honestly.
- Oracle inputs (if enabled) are sourced from verified feeds with freshness checks.

## Threat Agents
- Malicious buyers attempting to bypass caps or manipulate auctions.
- Liquidity providers attempting to drain AMM reserves via arithmetic exploits.
- Stakers attempting to fast-track unlocks or over-claim rewards.
- Administrators abusing privileged instructions without multi-sig oversight.
- External actors compromising RPC endpoints to serve inconsistent state.

## Attack Vectors & Mitigations
### Launchpad
- **Cap bypass / double-purchase**: Per-wallet tracking enforced on-chain with bounded arrays. Mitigation: seeds derived from buyer pubkey, wallet cap rechecked for each purchase.
- **Whitelist forgery**: Merkle proof verification using keccak256; failure triggers immediate rejection.
- **Auction sniping**: Anti-snipe window extends end time; settlement blocked until extended window lapses.
- **Re-entrancy**: No CPIs into user-controlled programs; token transfers executed last.

### AMM
- **Invariant violation**: Swap math uses checked arithmetic and constant-product quotes. Fuzz tests confirm invariant holds across random inputs.
- **Fee drain**: Protocol fees deposited to dedicated vault, withdrawable only by PDA signer and tracked via `TreasuryMovement` events.

### Staking
- **Reward inflation**: Emission rate stored on-chain, per-user claim uses slot delta and saturating math.
- **Lock bypass**: Lock policy enforced before unstake transfers.

### Vesting
- **Premature claim**: Vested amount computed from start/cliff/end; beneficiaries restricted via Merkle inclusion if provided.
- **Revocation misuse**: Revocation allowed only when `revocable` flag is true, rest tokens returned to authority vault.

## Operational Controls
- CI enforces `cargo clippy -- -D warnings`, `cargo audit`, and `cargo udeps`.
- Deployment scripts log final program IDs and derived PDAs.
- Access control validated by multi-sig checks in `authority` helpers.
