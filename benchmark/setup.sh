#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source ./env.sh

echo "benchmark setup: toolchain $(rustc --version)"
if ldconfig -p 2>/dev/null | grep -q 'libz\.so\.1'; then
  echo "benchmark setup: zlib available"
elif [ -n "${ZLIB_LIBRARY_PATH:-}" ] && [ -e "$ZLIB_LIBRARY_PATH/libz.so.1" ]; then
  echo "benchmark setup: zlib available from $ZLIB_LIBRARY_PATH"
else
  echo "benchmark setup: zlib missing; env.sh could not resolve libz.so.1" >&2
  exit 1
fi

if ! command -v cargo-llvm-cov >/dev/null 2>&1 && ! cargo llvm-cov --version >/dev/null 2>&1; then
  echo "benchmark setup: cargo-llvm-cov is missing" >&2
  exit 1
fi
if [ -z "${LLVM_COV:-}" ] || [ ! -x "${LLVM_COV:-}" ] \
  || [ -z "${LLVM_PROFDATA:-}" ] || [ ! -x "${LLVM_PROFDATA:-}" ]; then
  echo "benchmark setup: LLVM_COV/LLVM_PROFDATA are missing or not executable" >&2
  exit 1
fi
echo "benchmark setup: coverage tools cargo-llvm-cov + $("$LLVM_COV" --version | head -1)"

echo "benchmark setup: no candidate installs in this change; benchmark-run owns them"

# The change runs the fixture environment only. Candidate toolchains are
# pinned in the manifest; this setup verifies the fixture inputs are present
# instead of installing real candidate binaries on the host.
fixture_root="benchmark/tests/fixtures/e2e"
for path in \
  "$fixture_root/manifest.yaml" \
  "$fixture_root/cassettes/results/results.jsonl" \
  "$fixture_root/cassettes/model/model.jsonl" \
  "$fixture_root/gold/typo.jsonl" \
  "$fixture_root/gold/natural_language.jsonl" \
  "$fixture_root/gold/explanation.jsonl" \
  "$fixture_root/gold/concept.jsonl" \
  "$fixture_root/repo/src/math_utils.py" \
  "$fixture_root/repo/src/text_utils.py"; do
  if [ ! -f "$path" ]; then
    echo "benchmark setup: missing fixture input $path" >&2
    exit 1
  fi
done
echo "benchmark setup: fixture manifest, gold sets, cassettes, and repository present"

# The local OpenAI-compatible endpoint is an axum server bound to loopback on
# an ephemeral port. The e2e runner starts it in-process; no container
# runtime, shared server, or third-party service is involved.
echo "benchmark setup: fake OpenAI-compatible endpoint fixture is ready (loopback, ephemeral port)"
