# Implementation plan

Prepared: 2026-10-09. This is an ordered delivery plan. See [status.md](status.md) for checkpoint completion and pending owner commits.

## Commit gates

Each checkpoint ends with one reviewable change set, appropriate validation, updated public status, and a detailed local handoff in `commits/`. Mansi makes the commit. The assistant stops at the boundary and resumes only after Mansi explicitly confirms the commit.

Do not start the next checkpoint while the current handoff is awaiting a commit. If a checkpoint proves too large, split it before continuing and document the new boundary. A failed feasibility gate can be a useful, documented checkpoint; it must not be presented as a successful integration.

## Dependency order

```text
C0 Foundation
  -> C1 Protocol and native/WASM feasibility
  -> C2 Evidence contracts and trust selection
  -> C3 Native verification and signed fixture corpus
  -> C4 Finality composition and diagnostics
  -> C5 Browser worker and evidence inspector
  -> C6 Tamper lab and offline evidence replay
  -> C7 Real capture integration and demo hardening
```

The numbers describe logical milestones; each may become a smaller owner-controlled commit if needed. The important constraint is a coherent reviewable increment, not an artificially fixed number of commits.

## C0 - Project foundation (committed)

Deliver the project idea, plan, architecture, cryptographic model, source notes, test strategy, development workflow, status, and persistent assistant instructions. Preserve the existing five-sheet architecture PDF. Exclude private handoff material from normal Git staging.

Acceptance:

- Documentation agrees that no verifier or application is implemented yet.
- Public links and relative file links are valid at the time of checking.
- `commits/` is ignored and no file from it is tracked.
- Local handoff explains what was researched, what is still unknown, and the next gate.
- No commit or push is made by the assistant.

## C1 - Pin the protocol and prove native/WASM feasibility (committed)

The fixed-fixture compatibility experiment is implemented; its results and limitations are in [compatibility.md](compatibility.md). Mansi committed C1 as `08274f5`.

Tasks:

1. Select an immutable Agave source revision for the first supported profile. Record the SHA and why it matches the intended fixtures/capture format.
2. Inspect the associated vote serialization, signer encoding, BLS ciphersuite, point encodings, key qualification, and threshold rules.
3. Select compatible dependency versions and licenses. Preserve source attribution for any adapted code.
4. Build the smallest native Rust program/test that signs and verifies a deterministic non-production example using the selected primitives.
5. Compile and execute the corresponding verification path for browser WASM. Test actual execution, not only target compilation.
6. Record differences between the upstream reference implementation and any smaller adapter required for WASM.
7. Add a reproducible toolchain/build path and commit lockfiles appropriate to the workspace.

Acceptance:

- The same signed input is accepted natively and in WASM; a changed signed byte is rejected by both.
- Test data and logs identify the profile and representation exactly.
- No mock crypto is used as a substitute for this gate.
- If WASM is blocked, the cause and alternatives are documented. Stop at the gate before building a UI around an assumed solution.

C1 selects `solana-bls-signatures 4.0.0` with default and parallel features disabled, a small Notar payload adapter, Rust 1.91.1, and LLVM archive tooling. This proves the leaf-crypto path, not direct browser reuse of the full Agave verifier. C3 now implements the four-kind Base2 adapter and compatibility tests; full upstream differential execution remains unperformed.

## C2 - Define evidence contracts and trusted context (committed)

Implemented scope and constraints are in [contracts.md](contracts.md). Mansi committed C2 as `39399a6`.

Create the Rust workspace modules and TypeScript boundary contracts supported by the C1 result. Define versioned `ProofBundle`, `CertificateEnvelope`, `EpochCheckpoint`, `ProtocolProfile`, `VerifyRequest`, `VerificationReport`, and `MutationRecord` representations.

Implement bounded parsing, unknown-version handling, exact integer transport, raw-byte preservation, digest calculation, and separate checkpoint selection. Display labels supplied by a bundle must never become a trust decision. A digest identifies content; it does not authenticate its producer.

Acceptance:

- Valid round trips preserve evidence bytes.
- Malformed encodings, contradictory duplicate fields, oversized inputs, and lossy numeric values are rejected with specific diagnostics.
- A bundle cannot mark its own checkpoint trusted.
- Missing context produces an inconclusive report with dependent checks not performed.
- Schema/profile compatibility has deterministic behavior across native and browser boundaries.

## C3 - Native verifier and cryptographic fixtures (committed)

Implemented scope and compatibility evidence are in [certificate-verification.md](certificate-verification.md). Four certificate kinds with Base2 signers are covered. Mansi committed C3 as `06b6a8a`.

Implement profile-specific message construction, signer decoding, rank lookup, public-key/PoP qualification, aggregate signature verification, and checked stake accumulation. Prefer upstream verification behavior or a differential reference harness to independently establish compatibility.

Prepare deterministic, non-production fixture data for a valid signature, a changed payload, a changed bitmap, malformed points, wrong context, and a correctly signed under-quorum claim. Fixture generation stays separate from untrusted evidence verification.

Acceptance:

- Genuine positive and negative cryptographic cases pass their expected assertions.
- Boundary stake comparisons use exact integers; no floating-point threshold decisions exist.
- Unexpected ranks, duplicates, invalid points, invalid/missing PoP qualification, and unsupported encodings cannot yield a successful check.
- Fixture provenance, selected ranks, signed payload bytes, and expected results are reproducible.
- Tests do not rely solely on a generator and verifier sharing the same unexamined serialization logic.

## C4 - Finality composition and diagnostics (current handoff)

Implemented policy, API, and validation are in [finality.md](finality.md). C5 waits for Mansi to commit C4.

Implement the semantic evaluator above individually checked certificates. Separate fast finality, the paired slow path, and migration/genesis evidence. Preserve per-certificate outcomes and a typed overall evidence scope.

Acceptance:

- A valid notarization alone remains insufficient for a block-finality conclusion.
- The slow path requires compatible context, matching slot, and both required valid certificates. Its finalize vote is not misrepresented as signing a block ID.
- Genesis success is labeled migration evidence.
- Unsupported kinds and missing counterparts are explained without being mistaken for invalid signatures.
- `REJECTED` means the supplied claim failed, not that the chain proves the block can never finalize.

## C5 - Browser inspector backed by the actual core

Build a restrained React/TypeScript interface after the crypto path has been established. Run WASM in a worker, give each request a run ID, and prevent stale results from overwriting newer selections.

The initial screen needs file import, fixture selection, checkpoint selection, run controls, a check trace, certificate details, signer/stake inspection, and visible trust/provenance information. Use a simple black-and-white presentation with text/icons for state; color must not carry meaning alone.

Acceptance:

- A user can load a fixture, run the real verifier, and inspect its result without opening developer tools.
- Large or malformed inputs cannot freeze the main interaction path indefinitely; worker failure and cancellation are handled.
- The UI renders untrusted fields as text and never executes them.
- Exact values survive the JavaScript/WASM boundary.
- Keyboard use, focus, loading, error, and empty states work.

## C6 - Tamper lab and portable replay

Add immutable baseline state, derived experiments, explicit mutation records, comparable diagnostics, and export/import of evidence. Preserve report provenance and recompute imported results. Make checkpoint availability and profile version requirements explicit during replay.

Acceptance:

- Changing a signed input affects the expected check; the original remains byte-for-byte unchanged.
- Under-quorum examples use newly signed test evidence rather than claiming a changed bitmap preserves an old signature.
- Export/re-import reproduces deterministic check results and conclusions under the same profile/context.
- Tampering with a saved report cannot create a verified outcome.
- Verification completes with network requests disabled once the app assets and necessary local inputs are loaded.

## C7 - Capture adapter, documentation, and demonstration

Add a read-only adapter for a supported real capture format. Start with genesis RPC capture ingestion; verify only if the correct trusted historical checkpoint and signed context can be obtained. A custom node/footer exporter is a stretch dependency, not an assumed public API.

Finish the demonstration, build/test commands, fixture catalog, attribution, limitations, and reproducibility instructions. Run the appropriate native, WASM, integration, and browser checks for the implemented paths.

Acceptance:

- Real and synthetic evidence are always distinguishable.
- A capture missing its historical context is honestly reported as inconclusive.
- The application never confuses a containing footer's slot with the certificate's certified slot.
- The demo works from recorded local evidence even if a public endpoint is unavailable.
- Remaining limitations are explicit and no production-readiness claim is made.

## Scope pressure and fallback

Protect the cryptographic baseline, diagnostic trace, one controlled mutation, and reproducible export first. Cut live streaming, custom exporter work, decorative visualization, and broad format coverage before weakening verification. If an ordinary finality path cannot be implemented reliably within the available time, narrow the displayed supported scope and document that decision; never present a synthetic format as a network-compatible certificate.

No day estimate overrides the owner's commit gate. Progress is measured by the acceptance evidence above, not by filling folders or screens.
