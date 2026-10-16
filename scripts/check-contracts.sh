#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Rust owns every enumeration and limit; fail if the committed TypeScript copy is stale.
mkdir -p target
cargo run --locked --quiet -p monocle-wasm --bin generate-contracts > target/generated-contracts.ts
cmp packages/contracts/src/generated.ts target/generated-contracts.ts
npm run check:contracts
node packages/contracts/test/check-shapes.mjs
bash scripts/check-compat.sh
node scripts/check-browser.mjs contracts
