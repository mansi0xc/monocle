# Validation strategy

Status: C1 provides eight native crypto tests and six browser/native cases. C2 adds 18 native contract tests, 13 complete report comparisons in browser WASM, and TypeScript transport/shape checks. C3 adds 15 native integration tests and 32 complete certificate report comparisons with browser WASM. C4 adds ten native integration tests and 35 end-to-end finality/migration report comparisons. C5 adds five native bridge tests, 35 arbitrary-input WASM/native comparisons, six worker lifecycle tests, and 13 Chrome product tests with automated accessibility checks. C6 adds ten native experiment/replay tests, six native/arbitrary-input-WASM operation comparisons, two additional worker lifecycle tests, and seven additional Chrome product tests. C7 adds eight native capture/CLI tests, eight complete native/WASM capture comparisons, and six browser capture tests. Cryptographic verification of real-network captures remains unestablished.

## Current executable checks

Run `npm ci --ignore-scripts --no-audit --no-fund`, then `bash scripts/check-web.sh` from the repository root. It checks formatting, native tests, Clippy, deterministic fixture regeneration, a WASM release build with no unresolved imports, and actual Chrome/Chromium execution. See [compatibility.md](compatibility.md) for the tested scope and prerequisites.

The current suite covers the synthetic BLS baseline plus bounded input parsing, byte identity, exact integer transport, and explicit context selection. C2 compares actual Rust reports with TypeScript types and native versus browser outputs. Browser WebCrypto independently checks the source digest. C3 exercises four Base2 certificate kinds, point/PoP checks, signatures, and quorum. Its independent generator, wire reference comparisons, subgroup-negative vectors, and exact thresholds are documented in [certificate-verification.md](certificate-verification.md). C4 covers direct composition, missing counterparts, conflicting targets, scope separation, differing signers, and untrusted result injection. C5 exercises product user flows and the worker boundary; see [browser-inspector.md](browser-inspector.md). C6 covers original-byte preservation, complete mutation lineage, forged saved reports, exact checkpoint references, fresh comparison, and loaded-production-session replay with HTTP disabled. See [tamper-replay.md](tamper-replay.md) and [finality.md](finality.md).

## Evidence hierarchy

A successful build shows that code compiles. A synthetic round trip shows that some generator/verifier behavior agrees. Upstream compatibility requires independently checked message/encoding vectors or differential comparison. Browser support requires executing the WASM path. Keep these claims separate in handoffs.

Tests should protect protocol and user-visible behavior rather than mirror implementation details. The native core owns deterministic logic; browser checks concentrate on input handling, worker integration, result provenance, and user flows.

## Required core cases

| Area | Positive case | Negative or boundary cases |
| --- | --- | --- |
| Envelope parsing | Supported, bounded, lossless round trip | Invalid JSON/encoding, inconsistent duplicate fields, unsupported version, oversized input |
| Numeric handling | Full-range exact values cross the boundary | Floating-point precision loss, out-of-range and malformed integers |
| Checkpoint context | Explicit compatible trusted selection | Missing context, wrong epoch/cluster/domain, bundle-supplied trust flag |
| Signer decoding | Canonical selected ranks | Unknown rank, malformed bitmap, duplicate/overlapping forbidden selections, excessive lengths |
| Public-key qualification | Valid scheme-compatible key with accepted qualification | Invalid points, subgroup/identity violations, missing or false PoP qualification |
| Signature verification | Genuine compatible aggregate | Changed signed byte, signature byte, signer selection, or domain |
| Stake comparison | Threshold exactly reached | One unit below/above; overflow; wrong denominator; zero/invalid totals |
| Evidence composition | Supported fast path and paired slow path | Missing counterpart, mismatched slot/context, unsupported kind |
| Genesis scope | Valid dedicated profile | Wrong threshold/profile or labeling it as ordinary block finality |

Add multi-message/fallback cases only for formats deliberately brought into scope. An unsupported path needs an explicit inconclusive result rather than an unchecked success.

## Fixture requirements

- Reproducible fixture generation with clearly non-production keys.
- Separate metadata for synthetic fixtures and captured network evidence.
- Preserved raw message/certificate bytes and selected checkpoint identity.
- At least one correctly signed under-quorum claim.
- Documented expected per-check outcomes, not just an overall green/red value.
- Independent reference vectors or upstream differential verification for compatibility-sensitive encodings.
- No current network dependence for the deterministic test suite.

## Browser and integration checks

1. Load a fixture and display an actual worker-produced report.
2. Import an incomplete capture and show the missing prerequisites.
3. Preserve the baseline while modifying and verifying a derived bundle.
4. Ignore stale worker responses after the user changes input or context.
5. Handle worker failure, cancellation, and repeated runs without reusing an old success.
6. Export, re-import, and recompute results under the same profile/checkpoint.
7. Modify a saved report and confirm it cannot influence verification.
8. Disable network requests after assets are loaded and verify local replay.
9. Check keyboard focus, text labels, narrow screens, and untrusted text rendering.

## Release checks and reporting

Once code exists, define reproducible formatting, lint, unit/integration, native/WASM, and browser commands for the selected stack. Record each command, result, and relevant environment in the corresponding local handoff. A skipped test remains skipped; explain why and what it leaves unverified.

For C0, only document consistency, relative links, artifact presence, clean whitespace, and Git exclusion checks are appropriate. Do not add artificial application tests to imply implementation progress.

## C6 acceptance evidence

`bash scripts/check-web.sh` includes all 66 native tests, the earlier 6/13/32/35 browser/native corpus cases, 35 arbitrary-input full-report comparisons plus six experiment/replay comparisons, eight worker lifecycle tests, and 20 Chrome product tests. Browser tests compare full replayed reports while normalizing only locally generated run IDs. Expected conclusions are never shipped as product verdicts.

The replay tests alter before-bytes, parent identity, claims, source whitespace, policy/profile/version, duplicate fields, saved verdicts, and checkpoint bytes. They cover full-range decimal offsets, no-ops/chaining, a checkpoint string larger than 1 MiB without relaxing normal bundle limits, and both 10-byte/42-byte signature offsets. Production browser tests block networking after asset readiness and assert zero subsequent HTTP requests while creating, exporting, importing, and verifying an experiment. Desktop/narrow comparison screenshots and automated accessibility checks cover the new controls. This remains Chrome testing on a desktop host, not coverage of every browser, mobile device, or assistive technology.

## C7 acceptance evidence

The full gate now runs 74 native tests, the earlier 6/13/32/35 browser/native corpus cases, 35 arbitrary-input report comparisons, six experiment/replay comparisons, eight capture comparisons, eight worker lifecycle tests, and 26 Chrome product tests. Capture coverage distinguishes successful-null, certificate-present, unavailable-method, and other RPC errors; preserves exact source bytes and full-range u64 slots; rejects duplicate/unknown/contradictory fields and footer envelopes; and verifies that a capture cannot inherit a prior success or selected trust.

Native CLI exit status reflects parsing, never cryptographic validity. The browser test re-imports its exact downloaded response with networking disabled and asserts no HTTP requests after readiness. Capture views and missing-prerequisite wording are inspected at desktop/narrow widths with automated accessibility coverage. A recorded response is not a successful network signature test.

## C8 acceptance evidence

The landing and product redesign passes `npm run typecheck:web`, production Vite compilation, `npm run format:check`, and `npm run test:web` (eight worker tests and 33 Chrome browser tests). Seven new browser tests cover the landing preview, keyboard entry, fragment/history navigation, in-memory evidence and trust preservation, explicit trust removal, and accessibility/overflow at 375px and 768px. Existing product tests now enter at `/#/inspect` and continue executing the real WASM verifier.

Desktop/mobile screenshots were inspected for the landing page, verification reports, recorded capture, and replay comparison. Automated checks include the current contrast, keyboard, and reduced-motion behavior. These checks do not establish Firefox/Safari, real-device, or manual screen-reader compatibility. No Rust, contracts, fixtures, or cryptographic code changed, so C8 did not rerun the full native gate; the C7 results above are historical evidence, not a fresh C8 run.

## C9 reference/palette acceptance evidence

The supplied feather/lens reference and burgundy/navy/cream palette pass TypeScript, formatting, Vite production compilation, eight worker lifecycle tests and all 33 Chrome browser tests. Existing accessibility scans cover the changed landing, report, capture and comparison colours. Offline replay and recorded-capture tests still assert zero subsequent HTTP requests after readiness; the live logo is inline vector code. Desktop/mobile screenshots were visually inspected. No new tests or native cryptographic change were introduced. Other browsers, physical devices and manual assistive-technology use remain unverified.

The C11 Dusk implementation passes `npm run typecheck:web`, `npm run format:check`, production Vite compilation, `npm run check:contracts`, and `npm run test:web` (eight worker lifecycle tests and 33 Chrome browser tests). The landing tests were rewritten for the new page. They check the synthetic-fixture caption and every outcome code, that the landing page has no evidence or trust controls, outcome-anchor navigation, history and trust preservation, and accessibility/overflow at 375px and 768px. All inspector, capture and replay tests are unchanged and pass, including offline zero-HTTP behaviour, explicit trust, hostile input, cancellation, and the automated accessibility scans. Native code and cryptographic inputs are unchanged, so the native suites were not rerun. Desktop, tablet and phone screenshots were visually inspected. Other browsers, physical devices and manual assistive-technology use remain unverified.

C12 (display typeface) is CSS and font assets only. It passes `npm run typecheck:web`, `npm run format:check`, production compilation, and `npm run test:web`: eight worker tests and 33 Chrome browser tests, including all accessibility scans. Desktop and inspector screenshots were inspected.

C13 (motion and interaction) passes `npm run typecheck:web`, `npm run format:check`, production compilation, and `npm run test:web`: eight worker tests and 34 Chrome tests, with one new landing test covering keyboard scenario switching and the byte-42 tamper demo. One existing landing assertion changed from "no buttons" to "no evidence, trust or file controls". The axe scans first failed on mid-animation opacity and on a row-highlight background; reveals now clip instead of fading, the highlight is an edge marker, and every scan passes.

C14 (step-by-step inspector) passes `npm run typecheck:web`, `npm run format:check`, production compilation, and `npm run test:web` (8 worker tests, 34 Chrome tests) with no test changes. The existing suite exercises every control in the new arrangement, including hidden file inputs in disclosures, trust selection, experiments, replay, capture, offline behaviour and the accessibility scans.

C16 adds `.github/workflows/check.yml`, which runs `scripts/check-web.sh` in GitHub Actions on Ubuntu 24.04. The workflow passes actionlint 1.7.7 and structural checks: read-only permissions, SHA-pinned actions, and the gate command. Its first GitHub run (38897922360, commit `9531c59`) passed the full gate, confirming the Linux-specific steps.

C17 adds a drift check to `scripts/check-contracts.sh`: it regenerates `packages/contracts/src/generated.ts` from Rust and compares it byte-for-byte with the committed file. A deliberately altered copy fails the comparison. A new bridge test pins operation numbers 1–9 and checks that unknown numbers (0, 10, `u32::MAX`) fail closed with `INVALID_SCHEMA`.
