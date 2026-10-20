#!/usr/bin/env bash
# Measure the regular and e2e scopes in separate instrumentation runs.
# Each scope uses its own CARGO_TARGET_DIR so the two Cobertura reports come
# from distinct runs and cannot overwrite each other's profiles.
set -euo pipefail
cd "$(dirname "$0")"
source ./env.sh

: "${GIVN_COVERAGE_NON_E2E_PATH:=coverage/non-e2e-cobertura.xml}"
: "${GIVN_COVERAGE_E2E_PATH:=coverage/e2e-cobertura.xml}"
NON_E2E_TARGET_DIR="${GIVN_COVERAGE_NON_E2E_TARGET_DIR:-target/coverage/non-e2e}"
E2E_TARGET_DIR="${GIVN_COVERAGE_E2E_TARGET_DIR:-target/coverage/e2e}"
mkdir -p coverage

FEATURES="$(givn_resolve_features)"
if [ -z "$FEATURES" ]; then
  echo "no feature files found under givn/specs or givn/changes/*/specs" >&2
  exit 2
fi
export GIVN_FEATURES="$FEATURES"

if [ -z "${LLVM_COV:-}" ] || [ -z "${LLVM_PROFDATA:-}" ]; then
  echo "LLVM_COV and LLVM_PROFDATA must point at matching LLVM tools (source ./env.sh)" >&2
  exit 2
fi

rm -f "$GIVN_COVERAGE_NON_E2E_PATH" "$GIVN_COVERAGE_E2E_PATH"

# Regular scope: every scenario except the @e2e ones.
CARGO_TARGET_DIR="$NON_E2E_TARGET_DIR" cargo llvm-cov \
  --test benchmark_features --cobertura \
  --output-path "$GIVN_COVERAGE_NON_E2E_PATH" -- --tags 'not @e2e'
python3 benchmark/tools/cobertura.py probe "$GIVN_COVERAGE_NON_E2E_PATH" \
  benchmark/src/metrics/typo.rs \
  benchmark/src/report/render.rs

# E2E scope: the @e2e scenarios, including the benchmark CLI child process.
CARGO_TARGET_DIR="$E2E_TARGET_DIR" cargo llvm-cov \
  --test benchmark_features --cobertura \
  --output-path "$GIVN_COVERAGE_E2E_PATH" -- --tags @e2e
python3 benchmark/tools/cobertura.py probe "$GIVN_COVERAGE_E2E_PATH" \
  benchmark/src/cli.rs
