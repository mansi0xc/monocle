# Project status

Updated: 2026-10-10.

## Current state

Mansi committed C1 as `08274f5` (`tested native and browser wasm bls verification compatibility`). C2 implements bounded Rust evidence contracts, preserved raw bytes and SHA-256 identities, exact u64 transport, an explicit local checkpoint trust store, and deterministic preflight reports. TypeScript contracts and a native/browser corpus exercise the boundary.

C2 never establishes certificate validity or finality. Even a structurally valid bundle with a matching selected checkpoint stays `INCONCLUSIVE`, with decoding, signature, key qualification, quorum, and finality not performed. C1's genuine synthetic BLS experiment remains a separate passing baseline.

## Checkpoint progress

| Checkpoint | State |
| --- | --- |
| C0 - Foundation | Committed by Mansi: `ad561d4` |
| C1 - Protocol/native/WASM feasibility | Committed by Mansi: `08274f5` |
| C2 - Evidence contracts and trust selection | Implemented; awaiting Mansi's commit confirmation |
| C3 - Native verification and signed fixtures | Not started beyond C1's fixed primitive probe |
| C4 - Finality composition | Not started |
| C5 - Browser inspector | Not started; current pages are test harnesses |
| C6 - Tamper lab and replay | Not started; mutation metadata contract only |
| C7 - Real capture integration and demo | Not started |

## Evidence and limitations

See [contracts.md](contracts.md) for C2, and [compatibility.md](compatibility.md) for C1. The current suite covers 18 contract integration tests, eight C1 crypto tests, 13 C2 native/browser report comparisons, six C1 native/browser signature cases, and TypeScript transport/shape checks.

The C2 bundle fixture contains deliberately opaque invalid certificate bytes. Its declared format is a routing label only. No real testnet certificate has been verified. Historical context for the supplied genesis capture, complete certificate/bitmap decoding, BLS integration, quorum/finality logic, and product worker/UI integration remain open.

The detailed C2 handoff and actual validation log are local under ignored `commits/`. No assistant staging, commit, or push occurred. C3 waits for explicit owner commit confirmation.
