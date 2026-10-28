#!/usr/bin/env bash
# The manual real-evidence run.
#
# Preconditions: benchmark/fetch-repos.sh and benchmark/setup.sh have run, a
# loopback OpenAI-compatible embedding endpoint serves the embedding model,
# and BENCHMARK_EXPLANATION_ENDPOINT (plus its model in setup.lock.json) names
# the operator-permitted external explanation endpoint. BENCHMARK_EXPLANATION_KEY
# is sent as a bearer token when set.
#
# This script is not part of ./run-tests.sh. It records, replays, and leaves
# the evidence under target/real-run/ for the evidence task to commit.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

# shellcheck source=/dev/null
source ./env.sh

manifest="${BENCHMARK_MANIFEST:-benchmark/manifest.yaml}"
lock="${BENCHMARK_SETUP_LOCK:-benchmark/setup.lock.json}"
out="${BENCHMARK_REAL_OUT:-target/real-run}"
embedding="${BENCHMARK_EMBEDDING_ENDPOINT:-http://127.0.0.1:11434/v1}"

if [[ -z "${BENCHMARK_EXPLANATION_ENDPOINT:-}" ]]; then
  echo "run-real: set BENCHMARK_EXPLANATION_ENDPOINT to the operator-permitted explanation endpoint" >&2
  exit 2
fi

echo "run-real: validating the committed manifest and setup lock"
cargo run -q -p benchmark --bin setup-check -- \
  --manifest "$manifest" \
  --lock "$lock" \
  --fixed-inputs

rm -rf "$out"
mkdir -p "$out"

echo "run-real: recording against $embedding and $BENCHMARK_EXPLANATION_ENDPOINT"
cargo run -q -p benchmark --bin benchmark -- run \
  --manifest "$manifest" \
  --mode record \
  --embedding-endpoint "$embedding" \
  --explanation-endpoint "$BENCHMARK_EXPLANATION_ENDPOINT" \
  --out "$out"

echo "run-real: replaying the recorded run without an external call"
cargo run -q -p benchmark --bin benchmark -- run \
  --manifest "$manifest" \
  --mode replay \
  --out "$out"

for evidence in \
  "$out/provenance.json" \
  "$out/cassettes/results/results.jsonl" \
  "$out/cassettes/model/model.jsonl" \
  "$out/report.json" \
  "$out/report.md"; do
  if [[ ! -s "$evidence" ]]; then
    echo "run-real: missing evidence $evidence" >&2
    exit 1
  fi
done

echo "run-real: evidence under $out"
