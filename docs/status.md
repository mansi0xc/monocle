# Project status

Updated: 2026-10-11.

## Current state

Mansi committed C3 as `06b6a8a` (`verfied base2 certificates with bls and exact stake threshholds`). C4 adds `finality::verify_evidence`, which reruns individual certificate checks and composes direct fast/slow block-finality or genesis migration-anchor claims. Reports expose a typed established scope, requested target, selected path, witness certificate IDs, all individual checks, and explicit trust assumptions.

Missing counterparts/context remain inconclusive. Invalid selected evidence rejects the supplied claim without asserting that the block is non-final. Genesis success establishes only the migration anchor; it is not ordinary block finality or proof that the cluster completed migration. C2 preflight and C3 certificate-only APIs retain their narrower semantics.

## Checkpoint progress

| Checkpoint | State |
| --- | --- |
| C0 - Foundation | Committed by Mansi: `ad561d4` |
| C1 - Protocol/native/WASM feasibility | Committed by Mansi: `08274f5` |
| C2 - Evidence contracts and trust selection | Committed by Mansi: `39399a6` |
| C3 - Native verification and signed fixtures | Committed by Mansi: `06b6a8a` |
| C4 - Finality composition | Implemented; awaiting Mansi's commit confirmation |
| C5 - Browser inspector | Not started; current pages are development harnesses |
| C6 - Tamper lab and replay | Not started; mutation metadata and negative fixtures only |
| C7 - Real capture integration and demo | Not started |

## Evidence and limitations

See [finality.md](finality.md) for C4, [certificate-verification.md](certificate-verification.md) for C3, and [contracts.md](contracts.md) for C2. The suite includes 51 native tests (eight C1, 18 C2, 15 C3, ten C4), six C1 native/browser signature comparisons, 13 C2 report comparisons, 32 C3 report comparisons, 35 C4 full report comparisons, deterministic fixture regeneration, and TypeScript transport/shape checks.

All successful evidence uses synthetic public test keys and invented context. No real testnet certificate has been verified. Historical checkpoint authenticity and compatible qualification for the supplied genesis capture remain unresolved. Base3/fallback, full Agave differential execution, ancestry/execution proofs, authenticated epoch transitions, product worker/UI, tamper/replay, and capture adapters remain outside the implemented scope.

The detailed C4 handoff and validation log are local under ignored `commits/`. No assistant staging, commit, or push occurred. C5 waits for explicit owner commit confirmation.
