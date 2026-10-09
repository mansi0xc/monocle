#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked --quiet --bin generate_fixture > target/regenerated-fixture.json
cmp experiments/bls-compat/fixtures/aggregate.json target/regenerated-fixture.json
bash scripts/build-wasm.sh
node scripts/check-browser.mjs
