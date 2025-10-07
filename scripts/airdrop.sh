#!/usr/bin/env bash
set -euo pipefail

amount=${1:-2}
recipient=${2:-$(solana address)}

solana airdrop "$amount" "$recipient"
