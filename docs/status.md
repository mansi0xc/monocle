# Project status

Updated: 2026-10-11.

## Current state

Mansi committed C2 as `39399a6` (`added bounded evidence contracts and explicit checkpoint trust`). C3 implements individual certificate verification under the explicit `agave-98535e50-base2-v1` profile: bounded wire decoding, upstream Base2 signer decoding, checked BLS points and actual PoPs, aggregate signature verification, and exact stake thresholds.

Certificate inspection can now return `VERIFIED` under the selected checkpoint. Its scope is `individual_certificates`. Block-finality and genesis-migration claims remain `INCONCLUSIVE` pending C4, even when their individual certificates pass. C2 preflight still never returns a verified result.

## Checkpoint progress

| Checkpoint | State |
| --- | --- |
| C0 - Foundation | Committed by Mansi: `ad561d4` |
| C1 - Protocol/native/WASM feasibility | Committed by Mansi: `08274f5` |
| C2 - Evidence contracts and trust selection | Committed by Mansi: `39399a6` |
| C3 - Native verification and signed fixtures | Implemented; awaiting Mansi's commit confirmation |
| C4 - Finality composition | Not started |
| C5 - Browser inspector | Not started; current pages are test harnesses |
| C6 - Tamper lab and replay | Not started; mutation metadata and negative fixtures only |
| C7 - Real capture integration and demo | Not started |

## Evidence and limitations

See [certificate-verification.md](certificate-verification.md) for C3, [contracts.md](contracts.md) for C2, and [compatibility.md](compatibility.md) for C1. The suite includes 15 C3 integration tests, 18 C2 contract tests, eight C1 crypto tests, 32 C3 full native/browser report comparisons, 13 C2 report comparisons, six C1 signature cases, reproducible fixture regeneration, and TypeScript transport/shape checks.

The C3 corpus uses genuine signatures over synthetic context and public test keys. No real testnet certificate has been verified. Authentic historical context and compatible key qualification for the supplied genesis capture remain unresolved. Base3, fallback/multi-message variants, full Agave differential execution, cross-epoch authentication, finality composition, and the product worker/UI are outside this checkpoint.

The detailed C3 handoff and validation log are local under ignored `commits/`. No assistant staging, commit, or push occurred. C4 waits for explicit owner commit confirmation.
