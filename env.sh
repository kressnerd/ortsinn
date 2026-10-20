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
      export LD_LIBRARY_PATH="$ZLIB_DIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    fi
  fi
fi
