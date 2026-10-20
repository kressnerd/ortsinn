#!/usr/bin/env bash
# Merge the two Cobertura reports (regular and e2e) into the report the
# archive gate reads, then prove a known production path is covered.
set -euo pipefail
cd "$(dirname "$0")"
source ./env.sh

: "${GIVN_COVERAGE_NON_E2E_PATH:=coverage/non-e2e-cobertura.xml}"
: "${GIVN_COVERAGE_E2E_PATH:=coverage/e2e-cobertura.xml}"
: "${GIVN_COVERAGE_OUTPUT_PATH:=coverage/cobertura-coverage.xml}"
mkdir -p coverage

for path in "$GIVN_COVERAGE_NON_E2E_PATH" "$GIVN_COVERAGE_E2E_PATH"; do
  if [ ! -s "$path" ]; then
    echo "missing Cobertura report $path; run ./measure-coverage.sh first" >&2
    exit 2
  fi
done

if [ "$(sha256sum "$GIVN_COVERAGE_NON_E2E_PATH" | cut -d' ' -f1)" = \
     "$(sha256sum "$GIVN_COVERAGE_E2E_PATH" | cut -d' ' -f1)" ]; then
  echo "the regular and e2e Cobertura reports are identical; each scope needs its own run" >&2
  exit 2
fi

python3 benchmark/tools/cobertura.py merge \
  "$GIVN_COVERAGE_NON_E2E_PATH" "$GIVN_COVERAGE_E2E_PATH" "$GIVN_COVERAGE_OUTPUT_PATH"

# Production probes: one regular-scope path, one e2e-only path, plus the
# metric renderer both scopes touch.
python3 benchmark/tools/cobertura.py probe "$GIVN_COVERAGE_OUTPUT_PATH" \
  benchmark/src/metrics/typo.rs \
  benchmark/src/report/render.rs \
  benchmark/src/cli.rs
