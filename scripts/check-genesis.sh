#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# The synthetic RPC capture and the checkpoint and bundle built from it are
# committed; regenerating both must reproduce every byte. Only ignored build
# output under target/ is replaced.
out=target/genesis-synthetic
rm -rf "$out"
cargo run --locked --quiet -p monocle-genesis-checkpoint --bin synthetic-capture -- "$out"
cargo run --locked --quiet -p monocle-genesis-checkpoint -- \
  "$out/validators" "$out/built" "$out/genesis-cert"
diff -r fixtures/genesis-capture/synthetic "$out"
echo "Genesis capture pipeline: synthetic capture, checkpoint and bundle reproduce exactly."
