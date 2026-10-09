# Project status

Updated: 2026-10-09.

## Current state

Mansi committed the foundation as `ad561d4` (`docs ready for project init yoo`). C1 now contains an executable synthetic BLS compatibility experiment: the same Rust library verifies the same stored aggregate natively and in Chrome WebAssembly, with real proof-of-possession checks and five negative cases.

The product's bounded evidence parser, trusted-checkpoint selection, stake verification, finality evaluator, inspector UI, and tamper/replay flows are not implemented. No real testnet certificate or block finality has been verified by Monocle.

## Checkpoint progress

| Checkpoint | State |
| --- | --- |
| C0 - Foundation | Committed by Mansi: `ad561d4` |
| C1 - Protocol/native/WASM feasibility | Implemented; awaiting Mansi's commit confirmation |
| C2 - Evidence contracts and trust selection | Not started; next after confirmation |
| C3 - Native verification and signed fixtures | Not started beyond C1's fixed primitive probe |
| C4 - Finality composition | Not started |
| C5 - Browser inspector | Not started; C1 page is only a test harness |
| C6 - Tamper lab and replay | Not started |
| C7 - Real capture integration and demo | Not started |

## Evidence

See [compatibility.md](compatibility.md) for exact scope, build commands, selected sources/dependencies, browser results, and the archive-linking issue resolved during C1. Eight native integration tests and six native/browser cases pass. C1 uses synthetic public test keys with no stake or validator-authenticity claim.

## Open dependencies

- Authentic historical validator context for the supplied genesis capture.
- General untrusted evidence contracts and bounds.
- Full certificate/bitmap decoding, stake arithmetic, and evidence composition.
- Full upstream certificate differential validation and browser/product integration.

A detailed C1 handoff is stored locally under the ignored `commits/` directory. No assistant commit, staging, or push has occurred. Remaining implementation waits for Mansi's explicit commit confirmation.
