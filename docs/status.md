# Project status

Updated: 2026-10-11.

## Current state

Mansi committed C4 as `8fe23fa` (`added fast and slow finality with scoped genesis evidence`). C5 adds the product inspector: arbitrary Monocle bundle/checkpoint imports, explicit session trust selection, a cancellable worker running the real Rust/WASM verifier, readable check traces, certificate bytes, signer weights, and trust assumptions. The interface is monochrome and responsive.

The core's C4 scopes remain unchanged: direct fast/slow block finality, individual certificate inspection, and separately scoped genesis migration-anchor evidence. Missing context/counterparts and unsupported formats remain inconclusive. Invalid supplied evidence does not prove a block is non-final. Importing a checkpoint grants no trust; all signature/PoP and exact stake checks run again for every verification.

## Checkpoint progress

| Checkpoint | State |
| --- | --- |
| C0 - Foundation | Committed by Mansi: `ad561d4` |
| C1 - Protocol/native/WASM feasibility | Committed by Mansi: `08274f5` |
| C2 - Evidence contracts and trust selection | Committed by Mansi: `39399a6` |
| C3 - Native verification and signed fixtures | Committed by Mansi: `06b6a8a` |
| C4 - Finality composition | Committed by Mansi: `8fe23fa` |
| C5 - Browser inspector | Implemented; awaiting Mansi's commit confirmation |
| C6 - Tamper lab and replay | Not started; mutation metadata and negative fixtures only |
| C7 - Real capture integration and demo | Not started |

## Evidence and limitations

Run `npm run dev` for the inspector and `bash scripts/check-web.sh` for the full implemented gate. See [browser-inspector.md](browser-inspector.md) for commands, supported inputs, the WASM ABI, worker lifecycle, and test scope.

The suite now includes 56 native Rust tests (51 previous plus five bridge tests), the prior 6/13/32/35 native/browser comparisons, 35 additional full native/arbitrary-input-WASM comparisons, six worker lifecycle tests, and 13 Chrome product tests. These include automated accessibility scans and desktop/narrow screenshots, plus deterministic fixture regeneration, strict TypeScript checks, frontend/Rust formatting, and Clippy.

All successful evidence uses synthetic public test keys and invented context. No real testnet certificate has been verified. Historical checkpoint authenticity and compatible qualification for the supplied genesis capture remain unresolved. Base3/fallback, full Agave differential execution, ancestry/execution proofs, authenticated epoch transitions, tamper/replay, and capture adapters remain outside the implemented scope. Firefox/Safari, real mobile devices, assistive-technology use, and production hosting have not been tested in C5.

The detailed C5 handoff and validation log are local under ignored `commits/`. No assistant staging, commit, or push occurred. C6 waits for explicit owner commit confirmation.
