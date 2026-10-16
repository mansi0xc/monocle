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

## D012 - Preserve source bytes and hash them without JSON canonicalization

C2 retains original UTF-8 input, uses SHA-256 over exactly those bytes, and keeps evidence parsing in Rust. TypeScript transports the original JSON text rather than parsing and re-stringifying it. Whitespace changes alter identity; digests do not confer authenticity.

## D013 - Separate local trust decisions from import

A parsed checkpoint is untrusted until explicitly pinned in the local Rust trust store. A request selects an existing digest, and a bundle reference only constrains that selection. Operator reasons and synthetic-only decisions are disclosed. No imported flag bypasses PoP qualification.

## D014 - Report preparation separately from verification

C2 uses a contracts-only profile and `preflight_only` report scope. Unknown versions/formats and missing or mismatched context stay inconclusive. A valid input/context combination still leaves all certificate verification stages unperformed. The C1 BLS probe is not silently promoted into a complete verifier.

## D015 - Bound the initial container and mutation policy

Use explicit file, nesting, entry, and byte limits; require every certificate to be selected exactly once. C2 mutation metadata describes one bounded equal-length replacement per affected certificate. C6 adds stricter archive lineage validation and one replacement per experiment (D025). These are MVP container decisions, not Solana consensus limits.

## D016 - Explicit Base2 certificate profile and separate result scope

C3 accepts full versioned V1 consensus certificate messages for four kinds, with Base2 signers only. It reconstructs exact source votes, qualifies selected keys with actual standard PoPs, and checks real signatures and exact stake. Inspection success is scoped to individual certificates; block and migration claims stay inconclusive until C4. C2's preflight contract remains separate.

## D017 - Source-derived reference encoding, independent runtime decoding

The C3 generator does not depend on the runtime core. It uses reduced pinned wincode schemas and the upstream signer-store encoder. Runtime decoding is bounded and manual; tests compare explicit bytes and the reference schemas. This avoids a self-confirming shared serializer but does not claim full upstream verifier execution. Nonzero unused bitmap padding is intentionally rejected, a documented stricter rule than upstream.

## D018 - Actual PoPs without a registration shortcut

C3 verifies public-key-bound proofs with no extra application payload and never creates a qualified key from an imported Boolean. Trust in the checkpoint and its stake/rank data is still supplied separately. Application-specific registration proofs and historical provenance require additional evidence before real captures can be verified.

## D019 - Separate direct composition from certificate inspection

C4 introduces `finality::verify_evidence` and a pinned composition policy ID while preserving C2/C3 entry-point semantics. It always rechecks original request bytes using the local trust store; no public function composes caller-constructed reports. Established scope is `unestablished` until a supported claim succeeds. The requested target remains a distinct field.

## D020 - Require every selected certificate to pass

A valid fast path does not hide a bad or unsupported extra. Mixed migration/ordinary claims reject as incompatible evidence. Within a fully checked compatible bundle, prefer a fast path and choose redundant same-kind witnesses by lexical ID, with slow witnesses in Notar/Finalize order. No certificate weights are added across aggregates. These are deterministic Monocle bundle policies, narrower than a full node's evolving evidence pool.

## D021 - Genesis success means anchor evidence

A valid 82% genesis certificate establishes migration-anchor evidence under the chosen context, not completion of the transition or ordinary block finality. Activation, ancestry, TowerBFT, and local migration phase checks require evidence outside the current contract. Preserve this distinction in the future inspector.

## D022 - Bounded byte ABI without a new crypto or binding dependency

C5 uses a small Rust-owned input/output bridge around the established core. It does not introduce wasm-bindgen tooling or a second verifier. Original evidence bytes enter authoritative Rust parsing; only locally issued operations can pin a parsed checkpoint. Native tests and full-report comparisons exercise the same boundary. JavaScript must recreate memory views after execution. The application host itself remains trusted.

## D023 - Terminate a fresh worker for every operation

Each import inspection or verification gets a fresh worker and unique request ID. Completion, cancellation, error, timeout, and replacement terminate that worker. Input revision guards also discard late file reads. The app intentionally pays the small startup cost to avoid persistent trust/allocator state across runs. A selected checkpoint is retained in page memory as a separate local decision and reparsed/repinned during each verification.

## D024 - Monochrome inspection before experiment controls

C5 exposes source identity, provenance, explicit checkpoint selection, check traces, certificate bytes, signer weights, and assumptions. Native controls and a stacked narrow layout support keyboard and small-screen use. No decorative confidence scores or saved verdicts substitute for the core result. C6 extends this surface with restrained mutation/comparison/replay controls after implementing their core checks.

## D025 - One reproducible mutation from the original

C6 permits one nonempty equal-length replacement of at most 4096 bytes in one certificate. Replacing an experiment always starts from the original, and a no-op or chained mutation fails. The core rebuilds the entire derived document from the original during replay validation; a plausible after-slice alone cannot authorize forged before-bytes or unrelated metadata changes. The guided signature-bit preset uses the pinned decoder's field layout to locate the signature.

## D026 - Preserve source strings, discard imported verdicts

Replay v1 stores original JSON as strings instead of parsed objects, preserving exact byte identity. It records profile/policy IDs and an optional checkpoint digest/bytes; recipients select trust independently. Saved reports are bounded untrusted values and never become current UI results. Compare/export/import validates lineage, and the comparison reruns both inputs under the same checkpoint selection with distinct run IDs.

## D027 - Offline capability belongs to the loaded production session

Inline the production worker with Vite and retain one immutable fetched WASM buffer in page memory. Each new worker receives a copy, preserving cancellation/isolation while avoiding later network dependencies. The readiness label follows successful asset loading. This deliberately avoids a service-worker update/cache lifecycle; refresh, cold load, and development-server offline operation are outside the guarantee. Browser tests disconnect networking and assert zero new HTTP requests throughout replay.

## D028 - Treat an RPC capture as a separate input type

C7's read-only adapter takes a complete single JSON-RPC response, preserves exact source identity, and distinguishes certificate/null/method-error/other-error observations. It does not manufacture a ProofBundle with placeholder epoch or shred version. The product clears prior reports and checkpoint selection; the capture report has its own inconclusive scope. Raw source download/re-import is separate from C6 replay archives. The same parser runs in the native CLI and WASM operation 9.

## D029 - Complete the demo with an honest missing-context branch

Use the owner's recorded response for an endpoint-independent demonstration, with provenance and unknown retrieval time stated explicitly. Retain real cryptographic synthetic examples for signature/quorum/finality experiments. Historical checkpoint authentication, producer compatibility, and proof qualification remain future evidence requirements; no current-bank query or synthetic checkpoint substitutes for them. The demo guide explains both paths and the boundaries of loaded-session offline use.

## D030 - Locate diagnostics in bytes and separate signature failures

A debugger must say which bytes failed and why. Certificate-level paths end in a half-open byte range of the decoded certificate (`certificates[0].raw_hex[42..234]`), so a viewer can highlight them without reimplementing the wire format. The path stays a string, so the report schema is unchanged. Signature rejections are split into malformed, outside-subgroup, identity and failed-verification codes. The checked library conversion still alone decides acceptance; the split only classifies a rejection it has made. Key-qualification codes are unchanged, since those are checkpoint defects that already leave evidence unjudged.

## D031 - Registered-key trust for real validator sets

Vote accounts store a validator's compressed BLS key but not its proof of possession. The vote program verifies the proof once, at registration, over `"ALPENGLOW" ‖ vote account` (SIMD-0387), and Agave never re-verifies it: its rank map wraps vote-state keys with `PopVerified::new_unchecked`. Requiring proofs would leave every real certificate inconclusive. Fetching each registration transaction would cost about 2,000 archival lookups and still rely on the same RPC to say which key is current.

Mansi chose a separate trust basis, `RegisteredKeys`. The caller selects it at pin time with a recorded reason; no file field can grant it. Under it, key qualification still decodes each key with checked conversions (curve, subgroup, not identity) and then accepts it without a proof, reporting `REGISTERED_KEYS_ACCEPTED` and the assumption. This is the only `unsafe` call in `monocle-core`: the library's `PopVerified::new_unchecked` contract, not memory unsafety. Every other basis and all synthetic examples keep full proof verification.

## D032 - Network capture stays outside the verifier

Live RPC access lives only in `scripts/capture-rpc.mjs`. It writes exact response bytes and a manifest of request bodies and SHA-256 digests. The Rust builder reads only those files, re-checks every digest, and is deterministic. The browser and the native verifier keep their offline, zero-request guarantees; a capture is evidence of what an endpoint said, never trust.

## Open decisions

Authentic historical checkpoint acquisition and key qualification; supported fallback variants; historical capture source; project publication license. Resolve these at their corresponding checkpoint with evidence and tradeoffs.
