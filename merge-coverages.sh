#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source ./env.sh

: "${GIVN_COVERAGE_OUTPUT_PATH:=coverage/cobertura-coverage.xml}"
mkdir -p coverage

# cargo-llvm-cov merges the profiles in the current target directory; the
# e2e profile was written by measure-coverage.sh into coverage/.
cargo llvm-cov report --cobertura --output-path "$GIVN_COVERAGE_OUTPUT_PATH"
