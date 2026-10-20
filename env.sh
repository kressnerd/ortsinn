#!/usr/bin/env bash
# Shared environment for the benchmark harness.
# The host's default musl toolchain is broken; pin the gnu toolchain and
# provide libz.so.1 from the system or the Nix store.

export RUSTUP_TOOLCHAIN="stable-x86_64-unknown-linux-gnu"

if [ -z "${LD_LIBRARY_PATH:-}" ] || ! ldconfig -p 2>/dev/null | grep -q 'libz\.so\.1'; then
  if [ -e /usr/lib/x86_64-linux-gnu/libz.so.1 ] || [ -e /usr/lib/libz.so.1 ]; then
    :
  else
    ZLIB_DIR="$(nix build --no-link --print-out-paths nixpkgs#zlib 2>/dev/null | head -1)/lib"
    if [ -n "$ZLIB_DIR" ] && [ -e "$ZLIB_DIR/libz.so.1" ]; then
      export ZLIB_LIBRARY_PATH="$ZLIB_DIR"
      export LD_LIBRARY_PATH="$ZLIB_DIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    fi
  fi
fi

# Source-based coverage tools. The host PATH has no LLVM binaries. rustc
# 1.99.1 is built with LLVM 23.1.1 and writes raw profile format version 11;
# `nixpkgs#llvmPackages.llvm` resolves to 21.1.8 on this nixpkgs and refuses
# that profile. Resolve the LLVM major that matches rustc and use its tools.
# An explicit LLVM_COV/LLVM_PROFDATA pair wins.
if [ -z "${LLVM_COV:-}" ] || [ -z "${LLVM_PROFDATA:-}" ]; then
  LLVM_MAJOR="$(rustc -vV 2>/dev/null | awk '/LLVM version/ {split($3, parts, "."); print parts[1]}')"
  LLVM_PREFIX=""
  if [ -n "$LLVM_MAJOR" ]; then
    LLVM_PREFIX="$(nix build --no-link --print-out-paths "nixpkgs#llvmPackages_${LLVM_MAJOR}.llvm" 2>/dev/null | head -1)"
  fi
  if [ -z "$LLVM_PREFIX" ]; then
    LLVM_PREFIX="$(nix build --no-link --print-out-paths nixpkgs#llvmPackages.llvm 2>/dev/null | head -1)"
  fi
  if [ -n "$LLVM_PREFIX" ] && [ -x "$LLVM_PREFIX/bin/llvm-cov" ]; then
    : "${LLVM_COV:=$LLVM_PREFIX/bin/llvm-cov}"
    : "${LLVM_PROFDATA:=$LLVM_PREFIX/bin/llvm-profdata}"
    export LLVM_COV LLVM_PROFDATA
  fi
fi

# Resolve the Gherkin corpus for the executable-spec runner. The permanent
# corpus is `givn/specs`; every active change's delta specs are added without
# naming a change id. After archive the change directory is gone, so the
# permanent corpus is the only input. GIVN_FEATURES overrides the resolution.
givn_resolve_features() {
  if [ -n "${GIVN_FEATURES:-}" ]; then
    printf '%s\n' "$GIVN_FEATURES"
    return 0
  fi
  {
    if [ -d givn/specs ]; then
      find givn/specs -name '*.feature' -exec realpath {} +
    fi
    for specs in givn/changes/*/specs; do
      if [ -d "$specs" ]; then
        find "$specs" -name '*.feature' -exec realpath {} +
      fi
    done
  } | sort -u
}
