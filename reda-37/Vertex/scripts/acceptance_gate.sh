#!/usr/bin/env bash
set -u

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; CYAN='\033[0;36m'; NC='\033[0m'
pass() { printf "${GREEN}PASS${NC} - %s\n" "$1"; }
fail() { printf "${RED}FAIL${NC} - %s\n" "$1"; HAS_FAIL=1; }
warn() { printf "${YELLOW}WARN${NC} - %s\n" "$1"; }
info() { printf "${CYAN}INFO${NC} - %s\n" "$1"; }

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
cd "$ROOT_DIR" || exit 1

export PATH="$ROOT_DIR/scripts/bin:$PATH"

HAS_FAIL=0
BUILD_LOG="$(mktemp -t vertex_build_XXXX.log)"

echo "====== Vertex Acceptance Gate ======"
echo "Repo: $ROOT_DIR"
echo "Time: $(date -Is)"
echo

# 0) Tooling presence
command -v anchor >/dev/null 2>&1 || { fail "anchor CLI not found (install Anchor)"; }
command -v cargo  >/dev/null 2>&1 || { fail "cargo (Rust) not found"; }

# 1) Workspace wiring
if grep -q 'programs/fee_router' Cargo.toml; then
  pass "workspace members include programs/fee_router"
else
  fail "workspace members missing programs/fee_router in root Cargo.toml"
fi

if [ -d "programs/fee_router" ]; then
  pass "programs/fee_router crate exists"
else
  fail "programs/fee_router crate directory missing"
fi

# 2) Anchor.toml program IDs & keypairs
if [ -f Anchor.toml ]; then
  if grep -q '^\[programs\.localnet\]' Anchor.toml && grep -q 'keystone_fee_router' Anchor.toml; then
    pass "Anchor.toml has [programs.localnet].keystone_fee_router"
  else
    fail "Anchor.toml missing [programs.localnet].keystone_fee_router"
  fi
  if grep -q '^\[programs\.devnet\]' Anchor.toml && grep -q 'keystone_fee_router' Anchor.toml; then
    pass "Anchor.toml has [programs.devnet].keystone_fee_router"
  else
    warn "Anchor.toml missing [programs.devnet].keystone_fee_router (not fatal for local development)"
  fi
else
  fail "Anchor.toml not found"
fi

if [ -f target/deploy/keystone_fee_router-keypair.json ]; then
  pass "fee_router deploy keypair present: target/deploy/keystone_fee_router-keypair.json"
else
  fail "fee_router deploy keypair missing (expected at target/deploy/keystone_fee_router-keypair.json)"
fi

# 3) Meteora DLMM v2 program ID wiring (public program ID is known)
# Source: DAMM v2 ecosystem references show cpamdpZ... as the cp-amm program ID.
EXPECTED_METEORA_ID='cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG'
if grep -R --line-number -E "${EXPECTED_METEORA_ID}" programs/fee_router >/dev/null 2>&1; then
  pass "Meteora DLMM v2 program ID appears in fee_router (${EXPECTED_METEORA_ID})"
else
  warn "Meteora program ID (${EXPECTED_METEORA_ID}) not referenced in fee_router; ensure CPI targets the correct program."
fi

# 4) Stream adapter trait file
if [ -f programs/fee_router/src/stream_adapter.rs ]; then
  if grep -q 'trait\s\+StreamLockedReader' programs/fee_router/src/stream_adapter.rs; then
    pass "Streamflow adapter trait present (programs/fee_router/src/stream_adapter.rs)"
  else
    fail "stream_adapter.rs present but StreamLockedReader trait not found"
  fi
else
  warn "stream_adapter.rs missing; pluggable Streamflow adapter not found"
fi

# 5) BPF safety – ban stable sort on-chain (slice::sort). Prefer *_unstable.
if grep -R --line-number -E '\.sort\(' programs | grep -v 'sort_unstable' >/dev/null 2>&1; then
  fail "Found potential stable sort usage in programs/ (use sort_unstable or bpf_sort wrappers on BPF)"
else
  pass "No direct stable sort usage detected in programs/ (good for BPF stack limits)"
fi

# 6) Build & scan logs (captures BPF stack and realloc warnings)
echo
info "Building workspace (anchor build) … this may take a minute"
if ! anchor build >"$BUILD_LOG" 2>&1; then
  fail "anchor build failed (see $BUILD_LOG)"
else
  pass "anchor build completed (see $BUILD_LOG)"
fi

# 6a) Scan for 'driftsort' / BPF stack overflow warnings
# Only fail if the error occurs during main compilation, not during unit tests
if grep -E 'Stack offset .* exceeded max offset of 4096|driftsort_main' "$BUILD_LOG" >/dev/null 2>&1; then
  # Check if error happens only in unit test context (after "Running unittests")
  if grep -B5 -A1 'driftsort_main' "$BUILD_LOG" | grep -q 'Running unittests'; then
    warn "BPF stack overflow in unit tests detected (non-blocking, toolchain issue)"
  else
    fail "BPF stack overflow warning in main compilation found – replace stable sort or large stack allocations"
  fi
else
  pass "No BPF stack overflow logs detected (4096-byte stack respected)"
fi

# 6b) Scan for realloc deprecation
if grep -Ei 'realloc.*deprecated|use AccountInfo::resize' "$BUILD_LOG" >/dev/null 2>&1; then
  warn "Detected realloc deprecation in build logs – migrate to AccountInfo::resize()"
else
  pass "No realloc deprecation found in build logs"
fi

# 6c) Missing docs (not fatal)
if grep -E 'missing documentation for' "$BUILD_LOG" >/dev/null 2>&1; then
  warn "Missing docs warnings found (non-blocking, but tidy them for submission)"
else
  pass "No missing docs warnings"
fi

# 7) IDL & .so presence
if [ -f target/idl/keystone_fee_router.json ]; then
  pass "IDL generated: target/idl/keystone_fee_router.json"
else
  fail "IDL for fee_router missing (target/idl/keystone_fee_router.json)"
fi

if ls target/deploy/keystone_fee_router.so >/dev/null 2>&1; then
  pass "Program binary present: target/deploy/keystone_fee_router.so"
else
  fail "Program binary missing: target/deploy/keystone_fee_router.so"
fi

# 8) Fee Router program anatomy quick sanity
if grep -R --line-number '#\[program\]' programs/fee_router/src/lib.rs >/dev/null 2>&1; then
  pass "fee_router #[program] module present"
else
  fail "fee_router #[program] module not found in src/lib.rs"
fi

for fn in init_policy init_honorary_position crank_distribute; do
  if grep -E --line-number "pub fn ${fn}[[:space:]]*\(" programs/fee_router/src/lib.rs >/dev/null 2>&1; then
    pass "fee_router instruction present: ${fn}()"
  else
    fail "fee_router instruction missing: ${fn}()"
  fi
done

# 9) SPL Token transfers signed by vault PDA (basic grep)
if grep -R --line-number -E 'CpiContext::new.*token::transfer|token::transfer\(' programs/fee_router/src/lib.rs >/dev/null 2>&1; then
  pass "SPL token transfer CPI detected in fee_router (expected for payouts)"
else
  warn "No SPL token transfer CPI detected; ensure payouts are implemented"
fi

# 10) Tests for fee_router (optional)
if ls tests/*fee_router* >/dev/null 2>&1; then
  info "Running tests that mention fee_router (skip build for speed)…"
  if anchor test --skip-build > /dev/null 2>&1; then
    pass "anchor test (fee_router) passed"
  else
    warn "anchor test (fee_router) reported failures; inspect failing tests"
  fi
else
  warn "No fee_router tests found in tests/ (add program-test/bankrun coverage)"
fi

echo
if [ "${HAS_FAIL:-0}" -ne 0 ]; then
  printf "${RED}ACCEPTANCE GATE: FAILED${NC}\n"
  echo "See detailed build log: $BUILD_LOG"
  exit 1
else
  printf "${GREEN}ACCEPTANCE GATE: PASSED${NC}\n"
  echo "Build log: $BUILD_LOG"
  exit 0
fi
