#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
bash scripts/check-contracts.sh
cargo run --locked --quiet -p monocle-certificate-fixtures > target/regenerated-certificates.json
cmp fixtures/certificates/corpus.json target/regenerated-certificates.json
node scripts/check-browser.mjs certificates
