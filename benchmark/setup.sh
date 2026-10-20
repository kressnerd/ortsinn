#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source ./env.sh

echo "benchmark setup: toolchain $(rustc --version)"
echo "benchmark setup: candidate installs land under benchmark/.tools/"
