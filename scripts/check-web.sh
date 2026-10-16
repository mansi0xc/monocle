#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
bash scripts/check-finality.sh
bash scripts/check-genesis.sh
npm run build:web
npm run format:check
node scripts/check-wasm-bridge.mjs
npm run test:web
