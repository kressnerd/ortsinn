#!/usr/bin/env bash
# Materialize the fixed reference repositories and the synthetic monorepo at
# the manifest pins, then create the per-candidate writable copies.
#
# The manifest is the only pin origin: every checkout is fetched at the
# authored 40-character commit. No branch or "latest" is resolved.
set -euo pipefail
cd "$(dirname "$0")/.."
source ./env.sh

manifest="${1:-benchmark/manifest.yaml}"
root="$(pwd)"
bench="$root/benchmark"

cargo build -q -p benchmark --bins
check="$root/target/debug/setup-check"
"$check" --manifest "$manifest" --print-json > /tmp/benchmark-manifest.json

mkdir -p benchmark/work/repos

clone_pin() {
  local name="$1" url="$2" commit="$3"
  local dir="$bench/work/repos/$name"
  if [ -d "$dir/.git" ] && [ "$(git -C "$dir" rev-parse HEAD 2>/dev/null || true)" = "$commit" ]; then
    echo "fetch-repos: $name already at $commit"
    return
  fi
  rm -rf "$dir"
  mkdir -p "$dir"
  git -C "$dir" init -q
  git -C "$dir" remote add origin "$url"
  git -C "$dir" fetch -q --depth=1 origin "$commit"
  git -C "$dir" checkout -q FETCH_HEAD
  if [ "$(git -C "$dir" rev-parse HEAD)" != "$commit" ]; then
    echo "fetch-repos: $name checkout does not match pin $commit" >&2
    exit 1
  fi
  echo "fetch-repos: $name at $commit"
}

jq -c '.repositories[]' /tmp/benchmark-manifest.json | while IFS= read -r repository; do
  id="$(printf '%s' "$repository" | jq -r .id)"
  if [ "$(printf '%s' "$repository" | jq -r '.synthetic != null')" = "true" ]; then
    continue
  fi
  url="$(printf '%s' "$repository" | jq -r .url)"
  commit="$(printf '%s' "$repository" | jq -r .commit)"
  clone_pin "$id" "$url" "$commit"
done

# Generate the fixed synthetic monorepo and compare its content fingerprint
# with the pin recorded in the manifest.
synthetic_json="$("$root/target/debug/generate-synthetic" --root "$bench/work/synthetic")"
expected_fingerprint="$(jq -r '.repositories[] | select(.id == "synthetic-monorepo") | .synthetic.content_fingerprint' /tmp/benchmark-manifest.json)"
actual_fingerprint="$(printf '%s' "$synthetic_json" | jq -r .content_fingerprint)"
if [ "$actual_fingerprint" != "$expected_fingerprint" ]; then
  echo "fetch-repos: synthetic content fingerprint drift: expected $expected_fingerprint, actual $actual_fingerprint" >&2
  exit 1
fi
echo "fetch-repos: synthetic-monorepo at $actual_fingerprint"

# Per-candidate writable copies. Public repositories keep their `.git`
# metadata; the synthetic copy is a plain tree because it has no commit.
jq -c '.candidates[]' /tmp/benchmark-manifest.json | while IFS= read -r candidate; do
  candidate_id="$(printf '%s' "$candidate" | jq -r .id)"
  jq -c '.repositories[]' /tmp/benchmark-manifest.json | while IFS= read -r repository; do
    repository_id="$(printf '%s' "$repository" | jq -r .id)"
    destination="$bench/work/$candidate_id/$repository_id"
    rm -rf "$destination"
    mkdir -p "$(dirname "$destination")"
    if [ "$(printf '%s' "$repository" | jq -r '.synthetic != null')" = "true" ]; then
      cp -a "$bench/work/synthetic" "$destination"
    else
      git clone -q --shared "$bench/work/repos/$repository_id" "$destination"
      git -C "$destination" checkout -q "$(printf '%s' "$repository" | jq -r .commit)"
    fi
    echo "fetch-repos: $candidate_id/$repository_id ready"
  done
done

echo "fetch-repos: fixed inputs match the manifest pins"
