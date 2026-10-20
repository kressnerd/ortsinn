#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source ./env.sh

FEATURES="${GIVN_FEATURES:-$(find givn/specs givn/changes/benchmark-gate/specs -name '*.feature' -exec realpath {} + 2>/dev/null || true)}"
if [ -z "$FEATURES" ]; then
  echo "no feature files found" >&2
  exit 2
fi
export GIVN_FEATURES="$FEATURES"

if [ "${1:-}" = "--e2e" ]; then
  cargo test --test benchmark_features -- --tags @e2e
else
  cargo test --test benchmark_features
fi
