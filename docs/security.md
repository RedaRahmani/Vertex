# Security Practices

- Adopt the provided [Threat Model](../audits/threat_model.md) as part of every release review.
- Run `cargo audit` and `cargo udeps` in CI to detect vulnerable dependencies.
- Enforce `deny(warnings)` and `clippy` pedantic settings before merging.
- Use multi-sig authorities for all privileged accounts and store `sale_state_bump`/`treasury_bump` on-chain for reproducibility.
- Rotate authority keypairs regularly and document custody in runbook logs.

Incident response guide:
1. Halt new instructions (freeze frontends & CLI) if exploit suspected.
2. Capture state via `solana account` dumps for forensic analysis.
3. Broadcast upgrade or lock instructions through governance.
4. Coordinate with partners and publish RCA in repository.
