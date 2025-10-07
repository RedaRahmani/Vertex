# Deployment Runbook

1. **Build artifacts**
   ```bash
   anchor build
   cargo test
   ```
2. **Freeze IDs**
   - Copy generated program IDs from `target/deploy/*.json` into `Anchor.toml`.
   - Commit the updated files.
3. **Local rehearsal**
   ```bash
   ./scripts/localnet.sh
   ```
4. **Deploy to devnet**
   ```bash
   ./scripts/devnet.sh
   ```
5. **Post-deploy validation**
   - Run CLI `star status` to confirm PDAs and vault ownership.
   - Execute integration script to simulate launch + AMM swap + staking claim + vesting claim.
6. **Mainnet**
   - Obtain multi-sig approval recorded in governance.
   - Run `anchor deploy --provider.cluster mainnet`.
   - Distribute signed release notes referencing commit hash and checksums.

Ensure that security checklist items are signed-off prior to mainnet release.
