# Architecture

Status: proposed product architecture with C1 leaf cryptography, C2 input/context preparation, C3 individual certificate verification, C4 direct evidence composition, C5 browser inspection, and C6 mutation/comparison/replay implemented. Capture adaptation remains proposed. The [architecture PDF](architecture.pdf) provides five diagram sheets; this document defines the responsibilities and important details behind them.

## System boundary

Monocle's primary workflow is local verification. A static web host serves application assets. The browser imports evidence and a separately selected checkpoint, invokes a Rust/WASM worker, and displays the resulting report. Native tools currently generate synthetic fixtures; the capture adapter remains planned. The MVP requires neither a Solana program nor a wallet connection.

```mermaid
flowchart LR
    RPC[Genesis RPC capture] --> A[Capture adapter]
    NODE[Optional node footer export] --> A
    FIX[Signed synthetic fixtures] --> B[Bundle assembler]
    A --> B
    B --> UI[Browser controller]
    PIN[Separately trusted checkpoint] --> S[Local session store]
    UI --> S
    S --> W[Rust verifier in WASM worker]
    W --> R[Report inspector]
    R --> E[Portable evidence export]
    E --> UI
```

Transport sources are untrusted inputs to the verifier. TLS, a familiar endpoint name, or a file digest does not establish the authenticity of a validator set. The selected trust context is a distinct input.

## Component responsibilities

| Component | Owns | Must not do |
| --- | --- | --- |
| Capture adapter | Decode a supported source envelope and preserve captured bytes/context | Label a fetch as a successful verification |
| Bundle assembler | Versioned packaging, provenance, checkpoint reference | Silently rewrite signed bytes or manufacture absent context |
| Browser controller | User actions, run IDs, file limits, worker lifecycle | Make its own conflicting cryptographic verdict |
| Session store | Original bytes, derived copies, selected checkpoint, reports | Mutate the baseline or import a trusted flag from a bundle |
| Rust core | Parsing, profile/context checks, crypto checks, quorum, finality policy | Fetch replacement context from the network during a run |
| Report inspector | Trace, stake participation, evidence scope, assumptions | Hide unperformed checks behind a success badge |
| Exporter | Evidence, mutation log, profile/checkpoint identities, prior reports | Treat a stored result as proof when re-imported |

The native tool and worker should call the same core for the same profile. C3/C4 demonstrate the shared individual-certificate and direct evidence-composition paths in native Rust and browser WASM. C5 implements the arbitrary-input worker interface and inspector; C6 adds experiment/replay validation in the Rust core and fresh two-report comparison in the worker. See [tamper-replay.md](tamper-replay.md); see [browser-inspector.md](browser-inspector.md). See [certificate-verification.md](certificate-verification.md).

## Proposed source layout

```text
crates/
  monocle-core/       # deterministic verification and report types
  monocle-wasm/       # minimal browser boundary around the core
  monocle-cli/        # fixture generation, capture packaging, local inspection
apps/
  web/               # React/TypeScript controller and worker client
fixtures/            # synthetic and captured public evidence, with provenance
scripts/             # repeatable build, fixture, and verification utilities
docs/                # public design, research, usage, and status
commits/             # local ignored checkpoint handoffs
```

C2 implements `crates/monocle-core/` and a TypeScript transport package at `packages/contracts/`; C5 implements `crates/monocle-wasm/` and `apps/web/`; a general CLI and capture adapter remain planned. C1 lives separately in `experiments/bls-compat/` and `experiments/browser/`, with reproducible scripts and a pinned Rust workspace. C2 adds `experiments/contracts-compat/` and `experiments/contracts-browser/` for cross-target contract checks. C3 adds `tools/certificate-fixtures/`, `experiments/certificate-compat/`, and `experiments/certificate-browser/`. C4 adds a `finality` module and a native/browser finality corpus harness. See [finality.md](finality.md).

## Core interfaces

`VerifyRequest` contains a run identity, bundle bytes, an explicit checkpoint selection, and a supported profile identity. The core resolves immutable inputs and returns a report rather than mutating UI state. C2 uses `VerificationReport` for preflight only; C3 adds the separate `CertificateVerificationReport` for individual checks; C4 adds `EvidenceVerificationReport` with typed established scope, target, path, and witness IDs.

A report includes:

- Bundle, profile, and checkpoint identities.
- Per-certificate check outcomes and diagnostics.
- Signer ranks and exact stake totals for each applicable certificate.
- Certified slot and block ID where that certificate kind binds them.
- Overall conclusion, evidence scope, and explicit assumptions.

Network transport, timestamps, human-readable labels, and source claims are provenance. They are not signed merely because the bundle contains them. Contradictory parsed/display fields must fail consistency checks or be replaced with values decoded from authoritative bytes.

## Input and result model

| Object | Important fields and rules |
| --- | --- |
| `ProofBundle` | Schema version, profile ID, cluster context, certificate envelopes, capture metadata, checkpoint reference, optional parent digest and mutations |
| `CertificateEnvelope` | Kind, source format, raw bytes, signature and signer encoding as defined by the profile; source and normalized representations must agree |
| `EpochCheckpoint` | Epoch/context, canonical ordered ranks, BLS keys, stake, protocol denominator, qualification provenance; separately trusted |
| `ProtocolProfile` | Immutable upstream revision, allowed formats, payload rules, cryptographic encodings/ciphersuite, supported certificate rules |
| `MutationRecord` | Certificate ID, exact byte offset, equal-length before/after bytes, parent/result certificate digests |
| `ReplayArchive` | Exact baseline/derived JSON strings, profile/policy IDs, optional checkpoint bytes and digest, untrusted saved reports |
| `VerificationReport` | Per-check outcomes, per-certificate results, evidence scope, conclusion, assumptions |

The JSON-facing representation must not lose `u64` precision. C2 uses canonical decimal strings for slots, epochs, and stake, validated as u64. SHA-256 digests identify exact preserved bytes rather than an unspecified JSON key order. Semantic canonicalization, if introduced, needs a separate documented rule.

## Verification lifecycle

1. Enforce size/count limits and recognize schema/profile.
2. Resolve the independently selected checkpoint and compare context.
3. Decode signer selections under the pinned format and qualify keys.
4. Reconstruct the exact signed payload(s), validate encodings, and verify signatures.
5. Compute stake thresholds with checked integer operations.
6. Compose supported certificate evidence into a scoped conclusion.
7. Return all completed checks and reasons for skipped checks.

Invalid input can fail early. Missing prerequisites leave dependent checks unperformed. The evaluator must not assume that every certificate present in a bundle is relevant to the requested claim; C2 requires the claim to select every supplied certificate exactly once. C3 interprets individual certificates; C4 composes direct supported paths only after every selected certificate passes.

## Browser execution and persistence

Use a worker so cryptographic computation does not block input and rendering. Give runs stable identifiers and ignore stale replies. Replacing an input invalidates the corresponding displayed report until a fresh run completes. A worker crash should produce a clear execution error, not a cached verification success.

C6 keeps evidence, experiments, selected trust, and verifier assets in page memory. Explicit replay export preserves original source strings; import checks lineage and discards saved reports before presenting results. A fresh comparison reruns both inputs in one worker with separate run IDs and the same checkpoint selection. No server-side database or user account is required.

The production worker is inlined into the application script. WASM bytes are fetched once and copied into each new worker, so local workflows need no network after the readiness indicator appears. Closing or refreshing the page loses this session; there is no service worker or offline cold-start guarantee.

## Trust and operational boundaries

Checkpoint authenticity is a user/operator assumption in the fixed-checkpoint MVP. Key possession, public-key encoding validity, correct stake data, and the authenticity of the epoch set are separate requirements. Captured source metadata cannot satisfy them on its own.

Application delivery is also part of practical trust: a maliciously replaced verifier can lie about its checks. Reproducible builds, dependency pinning, and local execution help inspection, but the MVP should not claim to solve browser supply-chain trust.

Ordinary certificate capture may require node access and a custom exporter. That integration is outside the deterministic verification core and must not become a hidden dependency of the offline demonstration.
