# Architecture decisions

These are design decisions for the proposed MVP. They do not imply that the associated components are implemented.

## D001 - A finality debugger with an explicit checkpoint

Use a separately trusted, fixed epoch checkpoint as the starting context. This bounds the project and makes trust visible. An authenticated evolving light client remains future work; avoid describing the MVP as unconditionally trustless.

## D002 - One Rust core for native and browser verification

Target native Rust and browser WASM from shared verification logic. This reduces divergence and permits offline inspection. C1 passes for the Solana BLS leaf crate on native Rust and browser WASM. Full product integration remains separate; see D009.

## D003 - Pin supported protocol profiles

Tie message interpretation, signer encoding, cryptographic parameters, and thresholds to an immutable upstream revision. Unknown formats remain unsupported rather than being interpreted heuristically. C1 pins a synthetic probe profile to Agave `98535e50e9bb38574e6b0c31ff284cd864d3ca88`; historical capture compatibility is not established by that choice.

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

## D009 - Reuse leaf cryptography with a narrow protocol adapter

C1 uses Solana BLS 4.0.0, blst 0.3.17, and blstrs 0.7.1 under a pinned Rust 1.91.1 toolchain. A small Notar payload adapter is checked against fixed bytes and a wincode reference schema. The full Agave certificate crate is not imported; its broader dependency graph and newer workspace toolchain are unnecessary for this feasibility proof. Complete certificate compatibility remains a later gate.

## D010 - Require a self-contained WASM link and actual execution

Use LLVM archive tooling for the cross-build and reject unresolved runtime imports. A plain successful build previously left BLS symbols unresolved because Apple archive tools did not index the WASM object. Browser execution now checks the actual linked module against native results.

## D011 - Keep the feasibility probe deterministic

The WASM experiment fails closed if a dependency requests entropy and does not enable parallel features. It verifies fixed public test material; it neither generates production keys nor provides an arbitrary signing API. Revisit the entropy backend deliberately if a future feature needs randomness.

## Open decisions

Authentic checkpoint qualification; evidence digest/canonicalization; concrete import limits; supported fallback variants; historical capture source; general product WASM API; project publication license. Resolve these at their corresponding checkpoint with evidence and tradeoffs.
