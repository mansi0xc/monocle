# Architecture decisions

These are design decisions for the proposed MVP. They do not imply that the associated components are implemented.

## D001 - A finality debugger with an explicit checkpoint

Use a separately trusted, fixed epoch checkpoint as the starting context. This bounds the project and makes trust visible. An authenticated evolving light client remains future work; avoid describing the MVP as unconditionally trustless.

## D002 - One Rust core for native and browser verification

Target native Rust and browser WASM from shared verification logic. This reduces divergence and permits offline inspection. Adoption is conditional on the C1 execution test; the exact dependency and build choices are open.

## D003 - Pin supported protocol profiles

Tie message interpretation, signer encoding, cryptographic parameters, and thresholds to an immutable upstream revision. Unknown formats remain unsupported rather than being interpreted heuristically. Pinning is required before implementation claims compatibility.

## D004 - Three conclusions with per-check results

Use `VERIFIED`, `REJECTED`, and `INCONCLUSIVE` as distinct overall conclusions, plus `PASS`, `FAIL`, and `NOT_CHECKED` per check. Always attach evidence scope and trust assumptions. This preserves the distinction between bad evidence and missing evidence.

## D005 - Immutable evidence and derived experiments

Retain original bytes and record transformations in a separate derived bundle. Recompute all imported results. This makes experiments reproducible and prevents a modified report or mutable baseline from becoming an implicit source of truth.

## D006 - A static browser application for the initial product

Keep verification local and avoid introducing a backend, database, wallet, or on-chain program without a demonstrated need. Capturing external evidence remains a separate read-only integration.

## D007 - User-controlled commits and private checkpoint reports

Mansi creates every commit and push. Each assistant work checkpoint ends with a detailed ignored local report and a pause. This is an owner requirement, not an optional workflow convention.

## D008 - Documentation first; feasibility before interface breadth

Establish the proposal and acceptance criteria, then test actual native/WASM cryptographic compatibility before building an extensive UI. The first product increment must have a genuine verification path, not mocked success indicators.

## Open decisions

Exact BLS dependency and license; upstream profile SHA; Rust/WASM build tooling; checkpoint qualification mechanism; evidence digest/canonicalization specification; concrete import limits; supported fallback variants; capture/export source for historical context. Resolve these at their corresponding checkpoint with evidence and tradeoffs.
