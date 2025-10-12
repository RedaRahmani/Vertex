# FAQ

**Q:** How do I enable bonding curves beyond linear?

**A:** Implement the `PricingCurve` trait under `programs/common` and extend `LaunchPricing::BondingCurve` handling. Update tests and SDK builders accordingly.

**Q:** Where do Merkle proofs come from?

**A:** Use the generator in `scripts/merkle_proof.ts` (TBD) or any keccak-based Merkle tree library. Ensure leaves hash the buyer public key.

**Q:** Can the programs be upgraded?

**A:** Yes, but upgrades are expected to be governed via multi-sig and documented publicly. Consider locking upgrade authority once audited.
