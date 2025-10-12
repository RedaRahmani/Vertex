#!/usr/bin/env bash
set -euo pipefail

solana config set --url https://api.devnet.solana.com
anchor build
anchor deploy --provider.cluster devnet
