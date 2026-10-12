# Project status

Updated: 2026-10-12.

## Current state

Mansi committed C7 as `e396ade`. C8 adds the landing page and redesigns the existing inspector on `codex/monocle-ui`. The complementary ocean-blue/persimmon palette, local typography, responsive evidence workspace, and home/inspector navigation are ready for owner review. Colosseum feature exploration is paused; no new consensus or cryptographic functionality is included.

The landing page's Verify/Tamper/Replay preview is explicitly illustrative and synthetic. It cannot load evidence or grant trust. The actual inspector preserves its in-memory evidence and checkpoint choices when navigating home and back; refreshing still discards the session.

The recorded response at slot `444625255` remains **INCONCLUSIVE**. Its historical checkpoint, signing context, producer compatibility, and qualifying proofs have not been established. Capture inspection does not run signature, quorum, or finality checks and never substitutes an unrelated checkpoint.

The genuine Rust/WASM verification paths, immutable tamper experiments, fresh diagnostic comparison, and portable replay remain available for supported Monocle bundles. Fast/slow block finality and genesis-anchor evidence retain their distinct scopes. Successful demonstrations use synthetic public test keys and invented context. No real testnet certificate is cryptographically verified.

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
| C7 - Capture integration | Committed by Mansi: `e396ade` |
| C8 - Landing page and product UI | Implemented; awaiting Mansi's review/commit |

## Run and validate

Use `npm run dev` for development, or `npm run build:web` and `npm run preview` for production-session offline use. Open the inspector and wait for **Ready for offline verification**. Local verification, replay, and recorded capture inspection need no further network requests while that session stays open. Visiting only the landing page does not load the verifier. Offline first visits/refreshes and offline development-server use are unsupported.

Use `npm run inspect:capture -- fixtures/captures/testnet-genesis-rpc.json` for native capture inspection. Exit 0 means parsed input, not cryptographic success. See [capture-import.md](capture-import.md), [interface.md](interface.md), and [fixture-catalog.md](fixture-catalog.md).

C8 validation passes TypeScript checking, production Vite compilation, formatting, eight worker lifecycle tests, and 33 Chrome browser tests. Browser coverage includes landing navigation/history, checkpoint selection, real WASM verification, tampering, replay, captures, offline zero-HTTP behavior, and automated accessibility checks. Desktop/mobile landing and product screenshots were visually inspected.

The full C7 gate previously passed 74 native Rust tests and the native/browser comparison corpora described in [testing.md](testing.md). Native code, contracts, fixtures, and cryptographic inputs are unchanged in C8; those native suites were not rerun for this presentation checkpoint.

## Boundaries and remaining work

Historical checkpoint authentication/qualification and context-backed conversion of raw RPC captures remain unresolved. Base3/fallback, full Agave differential execution, ancestry/execution proofs, authenticated epoch transitions, node footer export, and production hosting remain outside scope. One equal-length mutation per experiment is supported. Firefox/Safari, real mobile devices, and manual assistive-technology use remain untested.

C8 is one reviewable checkpoint. The detailed handoff and validation evidence remain local under ignored `commits/`. No assistant staging, commit, push, merge, publication, or deployment occurred. Further implementation waits for owner review and commit confirmation.
