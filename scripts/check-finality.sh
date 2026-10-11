#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
bash scripts/check-certificates.sh
cargo run --locked --quiet -p monocle-certificate-fixtures --bin generate_finality > target/regenerated-finality.json
cmp fixtures/finality/corpus.json target/regenerated-finality.json
node scripts/check-browser.mjs finality
