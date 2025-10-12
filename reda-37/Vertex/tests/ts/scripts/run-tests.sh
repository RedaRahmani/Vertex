#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LEDGER_DIR="$(mktemp -d -t vertex_ts_validator_XXXX)"
LOG_FILE="${ROOT_DIR}/validator.log"

cleanup() {
  if [[ -n "${VALIDATOR_PID:-}" ]]; then
    kill "$VALIDATOR_PID" >/dev/null 2>&1 || true
    wait "$VALIDATOR_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$LEDGER_DIR"
}
trap cleanup EXIT

solana-test-validator --reset --ledger "$LEDGER_DIR" --limit-ledger-size 4096 >"$LOG_FILE" 2>&1 &
VALIDATOR_PID=$!

# wait for validator to become ready
for _ in {1..30}; do
  if solana -u localhost cluster-version >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if ! solana -u localhost cluster-version >/dev/null 2>&1; then
  echo "Validator failed to start; see $LOG_FILE" >&2
  exit 1
fi

cd "$ROOT_DIR"
export ANCHOR_WALLET="${ANCHOR_WALLET:-$HOME/.config/solana/id.json}"
export ANCHOR_PROVIDER_URL="${ANCHOR_PROVIDER_URL:-http://localhost:8899}"
npx mocha --require ts-node/register "src/**/*.test.ts" --timeout 120000
