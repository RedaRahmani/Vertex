#!/usr/bin/env bash
set -euo pipefail

if ! pgrep -f solana-test-validator >/dev/null; then
  echo "Starting local validator..."
  solana-test-validator --reset --limit-ledger-size 5000 &
  sleep 5
fi

anchor deploy
