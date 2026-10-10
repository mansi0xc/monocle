#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
npm run check:contracts
node packages/contracts/test/check-shapes.mjs
bash scripts/check-compat.sh
node scripts/check-browser.mjs contracts
