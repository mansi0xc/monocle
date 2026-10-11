# Project status

Updated: 2026-10-11.

## Current state

Mansi committed C6 as `95a990a` (`added immutable tamper experiments and offline evidence replay`). C7 adds a read-only genesis RPC adapter, a native local-file/stdin inspection command, a separately labeled browser capture workflow, the preserved owner-supplied recording, and the graduation demonstration guide.

The response at slot `444625255` can now be inspected and downloaded without altering its source. Its historical checkpoint, signing context, producer compatibility, and qualifying proofs have not been established. Capture inspection therefore remains **INCONCLUSIVE**, and signature, quorum, and finality checks do not run. No placeholder context or unrelated checkpoint is used to manufacture verification input.

The existing genuine Rust/WASM verification paths, immutable tamper experiments, fresh diagnostic comparison, and portable replay remain available for supported Monocle bundles. Fast/slow block finality and genesis-anchor evidence retain their distinct scopes. All successful demonstrations use synthetic public test keys and invented context. No real testnet certificate is cryptographically verified.

## Checkpoint progress

| Checkpoint | State |
| --- | --- |
| C0 - Foundation | Committed by Mansi: `ad561d4` |
| C1 - Protocol/native/WASM feasibility | Committed by Mansi: `08274f5` |
| C2 - Evidence contracts and trust selection | Committed by Mansi: `39399a6` |
| C3 - Native verification and signed fixtures | Committed by Mansi: `06b6a8a` |
| C4 - Finality composition | Committed by Mansi: `8fe23fa` |
| C5 - Browser inspector | Committed by Mansi: `2ab8e72` |
| C6 - Tamper lab and replay | Committed by Mansi: `95a990a` |
| C7 - Capture integration and demo | Implemented with the documented missing-context outcome; awaiting Mansi's review/commit |

## Run and validate

Use `npm run dev` for development, or `npm run build:web` and `npm run preview` for production-session offline use. Wait for **Ready for offline verification** and keep that page open. Local verification, replay, and recorded capture inspection need no further network requests. Offline first visits/refreshes and offline development-server use are unsupported.

Use `npm run inspect:capture -- fixtures/captures/testnet-genesis-rpc.json` for native capture inspection. Exit 0 means parsed input, not cryptographic success. See [capture-import.md](capture-import.md), [demo.md](demo.md), and [fixture-catalog.md](fixture-catalog.md).

`bash scripts/check-web.sh` passes 74 native Rust tests, the prior 6/13/32/35 native/browser comparisons, 35 complete arbitrary-input report comparisons, six experiment/replay comparisons, eight capture comparisons, eight worker lifecycle tests, and 26 Chrome product tests. The gate also includes deterministic fixture regeneration, zero-unresolved-import WASM builds, TypeScript checks, formatting, Clippy, offline zero-HTTP-request tests, and automated accessibility checks. Desktop/narrow capture and comparison screenshots are inspected.

## Boundaries and remaining work

The planned MVP workflows are implemented. Historical checkpoint authentication/qualification and context-backed conversion of raw RPC captures into verification input remain unresolved. Base3/fallback, full Agave differential execution, ancestry/execution proofs, authenticated epoch transitions, node footer export, and production hosting remain outside scope. One equal-length mutation per experiment is supported. Firefox/Safari, real mobile devices, and manual assistive-technology use remain untested.

C7 does not automatically open another implementation phase. The detailed handoff and validation evidence remain local under ignored `commits/`. No assistant staging, commit, push, publication, or deployment occurred. Further work waits for owner review/commit and an agreed next scope.
