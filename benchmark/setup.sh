#!/usr/bin/env bash
# Install and pin the real benchmark candidates at the manifest pins, record
# the probes and resolved toolchains in benchmark/setup.lock.json, and verify
# the result with the shared setup validator.
#
# The manifest is the only pin origin. This script installs exactly the
# authored version/commit; it never resolves "latest" and never writes a pin.
set -euo pipefail
cd "$(dirname "$0")/.."
source ./env.sh

manifest="${1:-benchmark/manifest.yaml}"
lock="benchmark/setup.lock.json"
root="$(pwd)"

cargo build -q -p benchmark --bins
check="$root/target/debug/setup-check"
"$check" --manifest "$manifest" --print-json > /tmp/benchmark-manifest.json

mkdir -p benchmark/work/tools

candidate() {
  jq -r --arg id "$1" '.candidates[] | select(.id == $id) | @base64' /tmp/benchmark-manifest.json
}
field() {
  printf '%s' "$1" | base64 -d | jq -r "$2"
}

probe_command() {
  local id="$1" command="$2"
  if eval "$command" >/dev/null 2>&1; then
    echo "ok"
  else
    echo "failed"
    echo "setup probe for $id failed: $command" >&2
    exit 1
  fi
}

# open-codebase-index: the npm release named by the manifest pin.
ocbi_record="$(candidate ocbi)"
ocbi_version="$(field "$ocbi_record" .version)"
ocbi_commit="$(field "$ocbi_record" .commit)"
ocbi_install="$(field "$ocbi_record" .install)"
ocbi_prefix="$root/benchmark/work/tools/ocbi"
mkdir -p "$ocbi_prefix"
npm install -g --silent --prefix "$ocbi_prefix" "opencode-codebase-index@${ocbi_version#v}" >/dev/null
installed_ocbi="$(jq -r .version "$ocbi_prefix/lib/node_modules/opencode-codebase-index/package.json")"
if [ "v$installed_ocbi" != "$ocbi_version" ]; then
  echo "open-codebase-index drift: manifest $ocbi_version, npm installed v$installed_ocbi" >&2
  exit 1
fi
probe_command ocbi "$root/benchmark/$ocbi_install --help"

# CKB: the Go module cannot be installed with `go install` because its tag and
# module path disagree, so build the pinned source commit.
ckb_record="$(candidate ckb)"
ckb_version="$(field "$ckb_record" .version)"
ckb_commit="$(field "$ckb_record" .commit)"
ckb_install="$(field "$ckb_record" .install)"
ckb_src="$root/benchmark/work/tools/ckb-src"
if [ ! -d "$ckb_src/.git" ]; then
  mkdir -p "$ckb_src"
  git -C "$ckb_src" init -q
  git -C "$ckb_src" remote add origin https://github.com/SimplyLiz/ckb.git
fi
if [ "$(git -C "$ckb_src" rev-parse HEAD 2>/dev/null || true)" != "$ckb_commit" ]; then
  git -C "$ckb_src" fetch -q --depth=1 origin "$ckb_commit"
  git -C "$ckb_src" checkout -q FETCH_HEAD
fi
if [ "$(git -C "$ckb_src" rev-parse HEAD)" != "$ckb_commit" ]; then
  echo "CKB drift: build checkout is not $ckb_commit" >&2
  exit 1
fi
(cd "$ckb_src" && GOBIN="$root/benchmark/work/tools/go/bin" GOFLAGS=-mod=mod go build -o "$root/benchmark/$ckb_install" ./cmd/ckb)
probe_command ckb "$root/benchmark/$ckb_install version | grep -q '${ckb_version#v}'"

# Serena: the PyPI release named by the manifest pin.
serena_record="$(candidate serena)"
serena_version="$(field "$serena_record" .version)"
serena_install="$(field "$serena_record" .install)"
UV_TOOL_DIR="$root/benchmark/work/tools/uv" \
  UV_TOOL_BIN_DIR="$root/benchmark/work/tools/uv/bin" \
  uv tool install --quiet "serena-agent==${serena_version#v}" >/dev/null
probe_command serena "$root/benchmark/$serena_install --version | grep -q '${serena_version#v}'"

"$check" --manifest "$manifest" --print-json > /tmp/benchmark-manifest.json

python3 - "$manifest" "$lock" <<'PY'
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

manifest_path, lock_path = sys.argv[1], sys.argv[2]
manifest = json.loads(Path("/tmp/benchmark-manifest.json").read_text())

def output(*command):
    return subprocess.run(command, check=True, capture_output=True, text=True).stdout.strip()

def version(*command):
    return output(*command).splitlines()[0]

toolchains = {
    "rustc": output("rustc", "--version"),
    "cargo": output("cargo", "--version"),
    "python": version("python3", "--version"),
    "node": version("node", "--version"),
    "npm": version("npm", "--version"),
    "go": version("go", "version"),
    "uv": version("uv", "--version"),
}
llvm_cov = os.environ.get("LLVM_COV")
if llvm_cov:
    toolchains["llvm_cov"] = output(llvm_cov, "--version").splitlines()[0]

candidates = {}
for candidate in manifest["candidates"]:
    identifier = candidate["id"]
    install = "benchmark/" + candidate["install"]
    if identifier == "ocbi":
        command = f"{install} --help"
        result = subprocess.run([install, "--help"], capture_output=True, text=True)
        lines = (result.stdout or result.stderr).splitlines()
        detail = lines[0] if lines else "(no output)"
        toolchain = {"node": toolchains["node"], "npm": toolchains["npm"]}
    elif identifier == "ckb":
        command = f"{install} version"
        detail = output(install, "version").replace("\n", " ")
        toolchain = {"go": toolchains["go"]}
    elif identifier == "serena":
        command = f"{install} --version"
        detail = output(install, "--version")
        toolchain = {"uv": toolchains["uv"], "python": toolchains["python"]}
    else:
        raise SystemExit(f"unexpected candidate {identifier}")
    candidates[identifier] = {
        "version": candidate["version"],
        "commit": candidate["commit"],
        "pinned_by": manifest_path,
        "probe": {"status": "ok", "command": command, "detail": detail},
        "toolchain": toolchain,
    }

repositories = {}
for repository in manifest["repositories"]:
    entry = {"pinned_by": manifest_path, "commit": repository["commit"] or None}
    if repository.get("synthetic"):
        entry["commit"] = None
        entry["synthetic"] = {
            "seed": repository["synthetic"]["seed"],
            "generator_version": repository["synthetic"]["generator_version"],
            "content_fingerprint": repository["synthetic"]["content_fingerprint"],
        }
    repositories[repository["id"]] = entry

lock = {
    "schema_version": 1,
    "manifest": manifest_path,
    "recorded_at": datetime.now(timezone.utc).isoformat(),
    "toolchains": toolchains,
    "candidates": candidates,
    "repositories": repositories,
    "endpoints": {
        "embedding": {
            "configured": True,
            "locality": "loopback",
            "base_url": "http://127.0.0.1:11434/v1",
            "model": "nomic-embed-text",
        },
        "explanation": {
            "configured": False,
            "locality": "external",
            "base_url": None,
            "model": "unconfigured",
            "query_budget": 80,
        },
    },
}
Path(lock_path).write_text(json.dumps(lock, indent=2, sort_keys=True) + "\n")
PY

"$check" --manifest "$manifest" --lock "$lock" --fixed-inputs
echo "benchmark setup: candidate pins, probes, toolchains, and fixed inputs verified"
echo "benchmark setup: explanation endpoint is unconfigured; the record run is blocked until the operator supplies BENCHMARK_EXPLANATION_ENDPOINT and its model"
