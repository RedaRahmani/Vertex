# Monitoring & Observability

- **Solana Logs**: Subscribe to program logs via `solana logs` or WebSocket to detect rejected purchases, slippage violations, and auction bids.
- **Events**: `TreasuryMovement` and `ConfigUpdated` events enable building indexers. Sample TypeScript indexer is provided under `sdk/ts` examples.
- **Metrics**: Aggregate sale progress from `SaleState` and publish to Grafana dashboards.
- **Alerts**: Trigger alerts on failed settlements, abnormal treasury withdrawals, or AMM invariant deviations detected by fuzz tests.

Integration with Helius or Switchboard WebSocket streaming is recommended for high-volume token launches.
