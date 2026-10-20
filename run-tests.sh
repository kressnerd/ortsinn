#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source ./env.sh

FEATURES="$(givn_resolve_features)"
if [ -z "$FEATURES" ]; then
  echo "no feature files found under givn/specs or givn/changes/*/specs" >&2
  exit 2
fi
export GIVN_FEATURES="$FEATURES"

if [ "${1:-}" = "--e2e" ]; then
  cargo test --test benchmark_features -- --tags @e2e
else
  cargo test --test benchmark_features
fi
