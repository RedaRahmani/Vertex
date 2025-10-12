#!/usr/bin/env bash
set -euo pipefail

anchor build
anchor deploy
