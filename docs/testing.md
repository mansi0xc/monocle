# Validation strategy

Status: C1 provides eight native crypto tests and six browser/native cases. C2 adds 18 native contract tests, 13 complete report comparisons in browser WASM, and TypeScript transport/shape checks. C3 adds 15 native integration tests and 32 complete certificate report comparisons with browser WASM. The broader product validation below remains planned.

## Current executable checks

Run `npm ci --ignore-scripts --no-audit --no-fund`, then `bash scripts/check-certificates.sh` from the repository root. It checks formatting, native tests, Clippy, deterministic fixture regeneration, a WASM release build with no unresolved imports, and actual Chrome/Chromium execution. See [compatibility.md](compatibility.md) for the tested scope and prerequisites.

The current suite covers the synthetic BLS baseline plus bounded input parsing, byte identity, exact integer transport, and explicit context selection. C2 compares actual Rust reports with TypeScript types and native versus browser outputs. Browser WebCrypto independently checks the source digest. C3 exercises four Base2 certificate kinds, point/PoP checks, signatures, and quorum. Its independent generator, wire reference comparisons, subgroup-negative vectors, and exact thresholds are documented in [certificate-verification.md](certificate-verification.md). Finality composition and product user flows remain unimplemented.

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
