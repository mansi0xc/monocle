# Project status

Updated: 2026-10-11.

## Current state

Mansi committed C5 as `2ab8e72` (`added worker-backed browser inspector with explicit checkpoint trust`). C6 adds an immutable-baseline tamper lab, fresh original/experiment report comparison, and portable replay. The monochrome inspector supports a signature-bit preset or one bounded raw byte replacement; every experiment records before/after bytes and exact source identities.

Replay import validates the complete mutation relationship against the supplied original, clears session trust, and ignores saved verdicts. Optional checkpoint bytes remain an untrusted candidate. Both reports are recomputed with the same explicitly selected checkpoint. Production assets stay in page memory, allowing local import/experiment/export/replay after disconnection while the page remains open.

The core's C4 scopes remain unchanged: direct fast/slow block finality, individual certificate inspection, and separately scoped genesis migration-anchor evidence. Missing context/counterparts and unsupported formats remain inconclusive. Invalid supplied evidence does not prove a block is non-final. No new cryptographic primitive, threshold, or finality path is introduced by C6.

## Checkpoint progress

| Checkpoint | State |
| --- | --- |
| C0 - Foundation | Committed by Mansi: `ad561d4` |
| C1 - Protocol/native/WASM feasibility | Committed by Mansi: `08274f5` |
| C2 - Evidence contracts and trust selection | Committed by Mansi: `39399a6` |
| C3 - Native verification and signed fixtures | Committed by Mansi: `06b6a8a` |
| C4 - Finality composition | Committed by Mansi: `8fe23fa` |
| C5 - Browser inspector | Committed by Mansi: `2ab8e72` |
| C6 - Tamper lab and replay | Implemented; awaiting Mansi's commit confirmation |
| C7 - Real capture integration and demo | Not started |

## Evidence and limitations

Run `npm run dev` for development, or `npm run build:web` followed by `npm run preview` for production-session offline use. Wait for **Ready for offline verification** and keep that page open. Offline first visits/refreshes and offline development-server use are unsupported. See [tamper-replay.md](tamper-replay.md) for the experiment and replay contract and [browser-inspector.md](browser-inspector.md) for setup and trust semantics.

`bash scripts/check-web.sh` passes 66 native Rust tests, the prior 6/13/32/35 native/browser comparisons, 35 full native/arbitrary-input-WASM report comparisons plus six experiment/replay comparisons, eight worker lifecycle tests, and 20 Chrome product tests. These include replayed full-report comparison, forged verdict isolation and history rejection, omitted checkpoints, actual offline operation with zero HTTP requests, automated accessibility scans, and desktop/narrow screenshots. Deterministic fixture regeneration, TypeScript checks, frontend/Rust formatting, and Clippy also pass.

All successful evidence uses synthetic public test keys and invented context. No real testnet certificate has been verified. Historical checkpoint authenticity and compatible qualification for the supplied genesis capture remain unresolved. Base3/fallback, full Agave differential execution, ancestry/execution proofs, authenticated epoch transitions, and capture adapters remain outside the implemented scope. C6 supports one equal-length patch rather than chained/multiple changes. Firefox/Safari, real mobile devices, assistive-technology use, and production hosting remain untested.

The detailed C6 handoff and validation log are local under ignored `commits/`. No assistant staging, commit, or push occurred. C7 waits for explicit owner commit confirmation.
