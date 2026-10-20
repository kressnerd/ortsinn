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

write_result() {
  local scope="$1" log="$2"
  [ -n "${GIVN_RESULT_FILE:-}" ] || return 0
  local summary total passed failed skipped scenarios
  summary="$(grep -E '^[0-9]+ scenarios \(' "$log" | tail -1 || true)"
  total="$(printf '%s\n' "$summary" | sed -nE 's/^([0-9]+) scenarios.*/\1/p')"
  passed="$(printf '%s\n' "$summary" | sed -nE 's/.*\(([0-9]+) passed.*/\1/p')"
  failed="$(printf '%s\n' "$summary" | sed -nE 's/.*, ([0-9]+) failed.*/\1/p')"
  skipped="$(printf '%s\n' "$summary" | sed -nE 's/.*, ([0-9]+) skipped.*/\1/p')"
  scenarios="$({ grep -E '^[[:space:]]*Scenario( Outline)?: ' "$log" || true; } \
    | sed -E 's/^[[:space:]]*Scenario( Outline)?: //' \
    | jq -R -s 'split("\n") | map(select(length > 0)) | map({name: ., status: "passed"})')"
  mkdir -p "$(dirname "$GIVN_RESULT_FILE")"
  jq -n \
    --arg scope "$scope" \
    --argjson scenarios "$scenarios" \
    --argjson total "${total:-0}" \
    --argjson passed "${passed:-0}" \
    --argjson failed "${failed:-0}" \
    --argjson skipped "${skipped:-0}" \
    '{scope: $scope, scenarios: $scenarios, passed: $passed, failed: $failed, skipped: $skipped, total: $total}' \
    > "$GIVN_RESULT_FILE"
}

SCOPE=regular
CARGO_ARGS=(--test benchmark_features)
if [ "${1:-}" = "--e2e" ]; then
  SCOPE=e2e
  CARGO_ARGS+=(-- --tags @e2e)
fi

LOG="$(mktemp)"
trap 'rm -f "$LOG"' EXIT
set +e
cargo test "${CARGO_ARGS[@]}" 2>&1 | tee "$LOG"
STATUS=${PIPESTATUS[0]}
set -e
write_result "$SCOPE" "$LOG"
exit "$STATUS"
