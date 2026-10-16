# C2: evidence contracts and explicit checkpoint selection

Status: implemented as a Rust library and TypeScript transport package. The browser compatibility harness executes the same Rust preflight logic. No certificate decoder, signature/quorum verifier, finality evaluator, product worker, or inspector UI is introduced by C2. The separate [C3 API](certificate-verification.md) now implements individual certificate checks; this document describes the preflight boundary.

## Entry points and source layout

- [Rust public API](../crates/monocle-core/src/lib.rs): `parse_bundle`, `parse_checkpoint`, `parse_request`, `preflight`, and `TrustStore`.
- [Wire types](../crates/monocle-core/src/model.rs): versioned input and output contracts.
- [TypeScript contracts/helpers](../packages/contracts/src/index.ts): browser transport; the Rust core remains authoritative.
- [Generated TypeScript values](../packages/contracts/src/generated.ts): written by `cargo run -p monocle-wasm --bin generate-contracts`; do not edit by hand.
- [Fixture notes](../fixtures/contracts/README.md): synthetic parser examples with opaque, deliberately invalid certificate bytes.
- [Cross-target harness](../experiments/contracts-compat/src/lib.rs): fixed preflight corpus, not a general product WASM API.

`Parsed<T>` owns the original bytes, their digest, and an immutable validated document. Its fields are private. Use the parsing entry points rather than treating a direct Serde deserialization of a transport type as validated input. The trust store accepts only a parsed checkpoint plus a separate caller decision.

## Versioned objects

| Object | Representation and responsibility |
| --- | --- |
| `ProofBundle` | Version 1, profile identity, context, explicit claim and certificate selection, certificate envelopes, optional checkpoint reference, provenance, optional parent digest and mutations |
| `CertificateEnvelope` | Version 1, unique local ID, claimed kind and format, original certificate bytes as lowercase hex |
| `EpochCheckpoint` | Version 1, profile/context, ordered rank entries, exact stake denominator, public keys, optional raw PoPs, provenance |
| `ProtocolProfile` | Compiled-in version 1 metadata; importing a JSON profile cannot install policy |
| `VerifyRequest` | Version 1, run ID, selected profile, preserved bundle JSON text, optional explicitly selected checkpoint digest |
| `VerificationReport` | Version 1, request/input/context identities, conclusion, evidence scope, ordered check results, assumptions |
| `MutationRecord` | Version 1, certificate ID, bounded equal-length byte replacement, offset, before/after bytes, parent/result certificate digests |

All input structs reject unknown fields. In particular, `trusted`, `pop_verified`, embedded checkpoints, and imported verification reports are not accepted as shortcuts. Optional fields accept omission or JSON `null`; serialization emits them explicitly as `null` when absent.

## Generated values

Every enumerated wire value is generated from Rust: diagnostic codes, stages, check statuses, conclusions, provenance kinds, certificate and claim kinds, evidence scopes, finality paths and capture observations. So are the size limits, profile, policy, wire-format and adapter identifiers, the Agave revision, the bridge ABI version and its named operations. Each Rust enum is declared through a `listed_enum!` macro, which also produces its complete variant list; the generator serializes that list with Serde, so the TypeScript strings are exactly what Rust emits. `scripts/check-contracts.sh` regenerates the file and fails if the committed copy differs.

Structural interfaces such as `ProofBundle` and `EvidenceVerificationReport` remain hand-written. `check-shapes.mjs` compiles real Rust reports and fixtures against them, so a field mismatch still fails the gate.

There are no duplicate normalized signature, bitmap, or block fields in a certificate envelope. Its raw bytes are decoded by the separate C3 API under a supported source format. The envelope's kind and the bundle's slot/block claim are unverified assertions until that decoding and semantic binding occurs.

Every certificate must be selected by the claim exactly once. C2 rejects duplicate IDs, absent references, or unselected extra certificates. This avoids silently changing which evidence a run considers. The order of the raw certificate list is retained.

## Transport and byte identity

Slots, epochs, stake values, and mutation offsets use canonical decimal **strings** covering the full `u64` range, from `"0"` to `"18446744073709551615"`. JSON numbers are rejected for these fields, even if a small number would be exactly representable. Leading zeroes, signs, spaces, decimal points, scientific notation, and overflow are rejected. Schema versions, shred versions, and rank indices use bounded JSON integers that are exactly representable by JavaScript.

Binary fields use even-length lowercase hex with no `0x` prefix. Fixed-width fields enforce their lengths: genesis hash and validator identity 32 bytes, uncompressed BLS key 96 bytes, and optional uncompressed PoP 192 bytes. These are **encoding checks only**. A correctly sized string of zero bytes is not thereby a valid group point or PoP; cryptographic validation is performed by the separate C3 API.

`ContentDigest` is `sha256:` followed by 64 lowercase hex digits. It hashes the exact preserved source bytes, including whitespace, order, and trailing newline. There is no JSON canonicalization step. Reformatting a checkpoint changes its identity and requires a new explicit selection/pin. A document can serialize and reparse with the same evidence bytes and values while acquiring a different outer file digest.

A digest identifies content; it does not authenticate the producer, prove stake ownership, or validate a certificate. Certificate mutation digests hash the decoded raw certificate bytes. The bundle's digest is returned in the parsed object/report rather than stored inside the bundle itself, avoiding a self-referential digest requirement.

The TypeScript helper accepts raw UTF-8 bytes and puts their decoded text into `VerifyRequest.bundle_json`. It does **not** parse and re-stringify the evidence object. This preserves duplicate keys for Rust to reject and prevents rounding of imported numeric values before validation. Invalid UTF-8 is rejected; a BOM is retained so the Rust JSON parser can reject it rather than silently rewrite the original. UTF-8 encoding the retained text reproduces the original valid bytes. `decimalU64` accepts a string or bigint for locally constructed values and refuses JavaScript Number.

## Concrete input bounds

These are Monocle MVP resource limits, not asserted Solana consensus limits. A larger legitimate input needs an explicit supported-contract change, not silent truncation.

| Limit | Maximum |
| --- | --- |
| Raw bundle | 1,048,576 bytes (1 MiB) |
| Raw checkpoint | 4,194,304 bytes (4 MiB) |
| Raw request | 6 × bundle limit + 4,096 bytes, allowing JSON string escaping |
| JSON nesting | Depth 32, with the root at depth zero |
| JSON values | 100,000 |
| Members per object / key length | 64 / 128 UTF-8 bytes |
| Elements per generic array | 4,096 |
| Individual decoded string | 1 MiB |
| Certificates / claim selections | 16 each, at least one |
| Decoded bytes per certificate | 65,536, at least one |
| Validator rank entries | 2,048, at least one |
| Mutation records | 64; current replacement policy also permits only one per certificate |
| Bytes on either side of a mutation | 4,096, at least one |
| Identifier length | 96 ASCII bytes, restricted to letters, digits, `_`, `-`, `.`, `:` |
| Provenance description / local trust reason | 1,024 UTF-8 bytes |

A strict first pass bounds JSON structure and rejects repeated object keys, including escaped spellings of the same key. Whole-file limits apply before JSON parsing. Schema-specific limits apply after the bounded first pass. Trailing non-whitespace input and invalid UTF-8/JSON are rejected. Schema-specific errors include a field path; first-pass errors use `$`. Messages and paths are truncated to keep diagnostics bounded.

The implementation retains Serde JSON's own recursion protection as well. It does not enable unbounded parsing.

## Checkpoint selection and trust

```text
checkpoint bytes -> parse_checkpoint -> Parsed<EpochCheckpoint>
                                               |
                            explicit local TrustBasis decision
                                               |
                                          TrustStore.pin
                                               |
request.selected_checkpoint digest ---------- resolve
bundle.checkpoint_reference digest ----------- compare only
```

`TrustStore` is an in-memory native/core API. There is no user-facing checkpoint selector or persistent trust store yet. Neither `TrustStore` nor `TrustBasis` is deserializable from evidence JSON. The browser harness's synthetic pin is an explicit test action, not an import feature.

Two local decisions exist:

- `SyntheticFixture`: for an explicitly selected synthetic checkpoint. It cannot support a bundle labeled as captured network evidence. Labels remain claims; the local decision is what grants this limited test context.
- `OperatorPinned { reason }`: records a nonempty caller-supplied reason for trusting a checkpoint. The caller remains responsible for independently establishing its authenticity. The reason is disclosed in the report.

No pin verifies key possession. Checkpoint entries may supply actual PoP bytes for C3 to validate; there is no imported flag that bypasses qualification. A missing PoP is preserved as missing, not invented. Operator trust is about the checkpoint source; it does not automatically waive the later cryptographic check.

Rank order must already be contiguous from zero in the input. Entries are never silently sorted. Validator identities and BLS keys must be unique; ranked stakes and the denominator must be positive. The sum of listed stakes is accumulated in `u128` and cannot exceed the declared `u64` denominator. Equality is not required: stake outside the listed eligible BLS ranks can remain in the denominator. The parser never reduces the denominator to the convenient sum of included keys. These are structural invariants, not proof that the supplied rank map matches Agave's authenticated epoch context.

The selected checkpoint must match the profile, genesis hash, epoch, and shred version. If the bundle also references a checkpoint, that reference must equal the explicit selection. Looking up or parsing a file by its digest does not pin it. Removing a pin immediately makes later selections unresolved.

## Preflight behavior

The C2 compiled profile is `agave-98535e50-contracts-v1`, tied to Agave `98535e50e9bb38574e6b0c31ff284cd864d3ca88`. It is separate from C1's synthetic signature probe profile. Its capability is explicitly `contracts_only`.

It recognizes the **names** of the `agave-certificate-wincode` format and four certificate kinds: notarization, finalization, fast finalization, and genesis. It recognizes certificate inspection, block-finality, and genesis-migration claim names. Recognizing a name does not decode its bytes or implement its semantics. Unsupported names produce an inconclusive report; they are never guessed from input shape.

Reports have nine ordered stages: input, profile, context, certificate decoding, signers, key qualification, signature, quorum, finality. Examples:

| Situation | Report result | Diagnostic |
| --- | --- | --- |
| Malformed JSON, duplicate fields, invalid integers/hex, invalid structural invariants | `REJECTED` | Specific input diagnostic |
| Unknown schema version | `INCONCLUSIVE` | `UNSUPPORTED_VERSION` |
| Unsupported profile or source/kind name | `INCONCLUSIVE` | `UNSUPPORTED_PROFILE` or `UNSUPPORTED_CERTIFICATE` |
| Missing explicit selection | `INCONCLUSIVE` | `MISSING_CHECKPOINT` |
| Digest not pinned locally | `INCONCLUSIVE` | `CHECKPOINT_NOT_TRUSTED` |
| Reference or chain context mismatch | `INCONCLUSIVE` | `CHECKPOINT_MISMATCH` or `CONTEXT_MISMATCH` |
| All preparation checks pass | `INCONCLUSIVE` | `VERIFIER_NOT_IMPLEMENTED` on dependent stages |

C2's `preflight` never returns `VERIFIED`. Its evidence scope is always `preflight_only`. Input/profile/context checks may pass; decoding, signature, stake quorum, and finality remain `NOT_CHECKED`. A malformed bundle rejection is not a statement that the chain's block cannot finalize.

## Mutation records

The current operation is `replace_bytes`, with one equal-length patch record per affected certificate. Parent bundle digest and mutation list must appear together. The target must exist, the offset must fit without overflow, the after-bytes must match the target slice, and the result certificate digest must match its current raw bytes.

When parsing a standalone bundle, the parent digest and before-bytes remain unverified history claims because this parser does not load the parent artifact. C6 adds a separate [replay validator](tamper-replay.md) that loads the exact original and reconstructs the derived document, checking one replacement across the whole bundle. This proves content consistency, not author or capture authenticity. Chained/multiple edits are outside the C6 archive contract.

## Reproduce and inspect

```sh
npm ci --ignore-scripts --no-audit --no-fund
bash scripts/check-contracts.sh
```

The browser runner waits for an explicit completion report over its temporary loopback server, with a 60-second deadline; it does not infer success from a page-load event.

This type-checks the TypeScript boundary, checks actual Rust/fixture JSON against those types, runs all native tests and Clippy, regenerates the original C1 signature fixture, builds the workspace WASM probes, and executes the C1 and C2 probes in actual Chrome/Chromium. Use `check-certificates.sh` for the additional C3 regeneration and browser gate. `CHROME_BIN` can select the executable. It requires the C1 Rust/Clang prerequisites and Node.js; C2 adds only a pinned TypeScript development dependency, not a UI framework.

To inspect the native preflight corpus:

```sh
cargo run --locked --quiet -p monocle-contracts-compat
```

The C2 corpus compares 13 complete reports between native and browser WASM. Browser checks independently compare Rust SHA-256 with WebCrypto and exercise the compiled TypeScript helpers. This is not arbitrary-file import through a product worker: the compatibility probe has a fixed embedded corpus and bounded output-reading exports. The production browser API and worker lifecycle are implemented separately in [C5](browser-inspector.md), with [C6](tamper-replay.md) adding mutation/replay operations. The C2 preflight still does not verify certificates.

## Sources informing implementation

- [Serde JSON 1.0.145 deserializer](https://docs.rs/serde_json/1.0.145/serde_json/struct.Deserializer.html): trailing-input checking and recursion constraints.
- [serde_path_to_error 0.1.20](https://docs.rs/serde_path_to_error/0.1.20/serde_path_to_error/): typed field paths in diagnostics.
- [RustCrypto SHA-2 0.10.9](https://docs.rs/sha2/0.10.9/sha2/): established SHA-256 implementation.
- [TypeScript 5.9 release documentation](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-9.html): compiler configuration; exact compiler pin is 5.9.3 in the lockfile.

These support implementation choices. The package's JSON schema, resource caps, trust API, and mutation constraints are Monocle design decisions, not claims of upstream certificate interoperability.
