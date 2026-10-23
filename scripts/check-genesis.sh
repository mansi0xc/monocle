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
# The recorded mainnet capture cannot be regenerated, but the checkpoint and
# bundle built from its bytes can.
mainnet=fixtures/genesis-capture/mainnet
out=target/genesis-mainnet
rm -rf "$out"
cargo run --locked --quiet -p monocle-genesis-checkpoint -- \
  "$mainnet/validators" "$out" "$mainnet/genesis-cert"
diff -r "$mainnet/built" "$out"
echo "Genesis capture pipeline: mainnet checkpoint and bundle rebuild exactly from the recorded capture."
