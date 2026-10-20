#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source ./env.sh

: "${GIVN_COVERAGE_NON_E2E_PATH:=coverage/non-e2e-cobertura.xml}"
: "${GIVN_COVERAGE_E2E_PATH:=coverage/e2e-cobertura.xml}"
mkdir -p coverage

FEATURES="${GIVN_FEATURES:-$(find givn/specs givn/changes/benchmark-gate/specs -name '*.feature' -exec realpath {} + 2>/dev/null || true)}"
export GIVN_FEATURES="$FEATURES"

cargo llvm-cov --test benchmark_features --cobertura \
  --output-path "$GIVN_COVERAGE_NON_E2E_PATH" -- --tags 'not @e2e'

eval "$(cargo llvm-cov show-env --export-prefix)"
export LLVM_PROFILE_FILE="coverage/e2e-%p.profraw"
cargo test --test benchmark_features -- --tags @e2e
cargo llvm-cov report --cobertura --output-path "$GIVN_COVERAGE_E2E_PATH"
