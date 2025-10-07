# End-to-End Tests

The `sdk/ts` Vitest suite executes Anchor instruction builders without hitting a validator. For full stack flows:

```bash
./scripts/localnet.sh
pnpm --prefix sdk/ts run test
```

Fixtures under `fixtures/` demonstrate launch configuration payloads consumed by the CLI.
