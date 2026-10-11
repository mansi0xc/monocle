# Research and evidence ledger

Reviewed: 2026-10-09. Sources below are primary protocol or implementation references. The original moving-branch links below are research pointers. C1 now pins Agave `98535e50e9bb38574e6b0c31ff284cd864d3ca88` and Solana BLS 4.0.0 for its synthetic probe; see [compatibility.md](compatibility.md) and [the exact profile record](../experiments/bls-compat/protocol-profile.json).

## Existing observation supplied by Mansi

A request to `getAgGenesisCert` at `https://api.testnet.solana.com` returned a certificate object whose block slot is `444625255`, with block ID, signer bitmap, and signature arrays. This was supplied in the project conversation. It demonstrates successful retrieval for that request, not ongoing endpoint availability and not cryptographic validity.

The matching historical validator rank map, keys, stake denominator, key qualification, and signing context have not yet been obtained and verified for that capture. No report should label it cryptographically verified at this stage.

## Primary references and design impact

| Reference | Finding relevant to Monocle | Design impact |
| --- | --- | --- |
| [SIMD-0326](https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0326-alpenglow.md) | Describes certificate types and fast/slow finality | Separate certificate checks from evidence composition |
| [Agave certificate verifier](https://github.com/anza-xyz/agave/blob/master/bls-cert-verify/src/cert_verify.rs) | Accepts externally supplied rank/stake/key context and checks aggregates | The wrapper must make that trust input explicit |
| [Verifier dependencies](https://github.com/anza-xyz/agave/blob/master/bls-cert-verify/Cargo.toml) | Upstream verifier has its own dependency and feature graph | Prove WASM support before choosing direct reuse |
| [Vote wire format](https://github.com/anza-xyz/agave/blob/master/votor-messages/src/wire.rs) | Defines payload serialization and signed context | Pin and test message construction, not only the curve |
| [Certificate types](https://github.com/anza-xyz/agave/blob/master/votor-messages/src/certificate.rs) | Associates types with thresholds and payload interpretation | Keep semantic policy attached to a protocol profile |
| [Migration logic](https://github.com/anza-xyz/agave/blob/master/votor-messages/src/migration.rs) | Defines genesis handover and its dedicated threshold | Show genesis as a separate evidence scope |
| [Epoch stake construction](https://github.com/anza-xyz/agave/blob/master/runtime/src/epoch_stakes.rs) | Defines the ordered validator context used by the client | Do not derive signer ranks from an arbitrary RPC account ordering |
| [Solana integration guide](https://solana.com/upgrades/alpenglow) | Documents genesis RPC and footer notification access | Separate capture adapters from verification |

## Findings requiring careful wording

**Genesis access:** the RPC method is useful for capture and migration observation. It is not a general stream of ordinary finality certificates. Unsupported method, null result, and a returned object are different observations.

**Footer access:** ordinary evidence may be included in a block footer and exposed through a node integration. Certificate fields can be absent, and a certificate can refer to a different slot from the containing block. Do not promise availability from an ordinary public RPC or a standard subscription without confirming that exact integration.

**Genesis terminology:** the high-level integration page describes the genesis slot in activation terms, while the migration implementation derives an anchor during the handover. Monocle will use the precise label "genesis/migration anchor" and follow the pinned code's semantics instead of asserting that it is an ordinary post-migration block.

**Source drift:** current branch contents may differ from the node that produced the user's capture. A current implementation compiling successfully would not alone establish compatibility with that historical response.

## Open questions and experiments

| Question | Evidence needed | Checkpoint |
| --- | --- | --- |
| Can the selected verification path run in browser WASM? | Execute a real signature verification and a negative vector in the browser build | C1 |
| Which revision matches the supported certificate representation? | Immutable upstream SHA, payload/encoding vectors, capture compatibility notes | C1 |
| Which BLS dependency and qualification API are suitable? | Review scheme defaults, serialization, key validation, license, native/WASM behavior | C1 |
| How will checkpoint authenticity and PoP qualification be supplied? | Explicit documented policy and rejection cases | C2-C3 |
| Can the historical context for slot 444625255 be acquired? | Authentic checkpoint artifact and exact context bound to that capture | C7, with early discovery as practical |
| How are failures distinguished across the browser boundary? | Typed report schema and integration cases | C2-C5 |
| Can a report be reproduced without live network access? | Export/re-import and browser offline verification checks | C6 |

C1 has now executed the fixed signature cases in native Rust and Chrome WASM, checked one payload against a wincode reference schema, and selected compatible leaf dependencies. Historical capture compatibility and checkpoint authenticity remain unresolved. These results do not imply that the complete certificate verifier or the remaining experiments have passed.

## Research practice for later checkpoints

Record exact source revisions, dependency versions, commands, decisive results, and rejected alternatives. Public documentation should retain reusable findings; the local commit handoff should retain the detailed work narrative. Do not copy broad rollout dates, marketing performance figures, or the original idea document's "trustless" claim into product claims without the evidence required for that statement.

## C2 implementation findings (2026-10-10)

C2 uses a duplicate-rejecting, bounded JSON visitor before typed decoding; a generic JSON value parse alone would erase repeated object fields. Exact-source hashing avoids inventing a canonical JSON representation. SHA-256 is supplied by RustCrypto, with an independent known-answer test and browser WebCrypto comparison.

The [C2 contract specification](contracts.md) records primary library documentation, version pins, resource limits, and the trust boundary. This checkpoint does not select a new consensus rule or establish historical capture compatibility.

## C3 implementation findings (2026-10-11)

The pinned wire schema distinguishes containing certificate tags from source-vote tags. Fast-finalization uses notarization vote signatures; finalization signs only a slot and shred version. The four supported thresholds are inclusive 60/60/80/82%, and the upstream certificate verifier checks signatures before quorum using externally supplied qualified rank/stake context.

C3 directly reuses signer-store 0.1.0 Base2 decoding. Inspection found that it truncates unused padding bits; Monocle intentionally requires zero padding. Checked SDK point conversions and real PoP checks qualify selected keys. Synthetic on-curve points outside the prime-order subgroups test both G1 and G2 rejection, alongside identity/malformed points and wrong-key PoPs.

The reduced schema/reference and bounded runtime adapter agree on four wire kinds and exact source payloads. This is not a differential run of the full Agave verifier. Exact sources, file hashes, policy differences, and remaining qualification/context limits are in [the C3 specification](certificate-verification.md) and [profile record](../experiments/certificate-compat/protocol-profile.json). The historical genesis capture remains unverified.

## C4 composition findings (2026-10-11)

Pinned `runtime/src/validated_block_finalization.rs` distinguishes fast certificates from paired Notar/Finalize evidence for the same slot and verifies each aggregate independently. `votor/src/consensus_pool.rs` pairs certificates in either arrival order by slot, takes the block identity from notarization, and handles Genesis through migration status. Migration state includes activation/phase/ancestry conditions beyond the presence of a valid genesis certificate.

C4 follows those direct evidence paths under the existing fixed checkpoint. It does not port the full runtime, authenticate migration state, or claim a differential Agave execution. The complete-bundle acceptance rule, explicit target requirement, stricter supported-kind scope, and deterministic witness selection are Monocle policies. Source hashes and distinctions are recorded in [the policy record](../experiments/finality-compat/policy.json) and [C4 specification](finality.md).

## C5 browser boundary findings (2026-10-11)

The product can reuse the existing zero-import WASM build with a small Rust-owned buffer ABI; no new cryptographic dependency or scheme is needed. Memory growth requires fresh JavaScript views, and terminating a worker provides an immediate cancellation boundary even during synchronous BLS verification. Original JSON strings must reach the strict Rust parser unchanged; generic JavaScript parsing would erase duplicate keys and risk integer conversion.

React/Vite setup, worker termination, memory-view behavior, version pins, executable checks, and remaining limits are documented with primary sources in [the C5 inspector specification](browser-inspector.md). Synthetic browser success remains conditional on a separately selected synthetic checkpoint and does not verify a network capture.

## C6 reproducible experiments and offline assets (2026-10-11)

The existing pinned decoder places the signature after version/kind/slot (10 bytes) for Finalize, and after the additional block ID (42 bytes) for other supported kinds. The guided experiment uses that kind-aware layout; native tests exercise both offsets. A signed-field mutation can be caught by claim binding before signature verification, so the comparison displays actual check outcomes without assuming the failing stage.

The replay wrapper preserves source JSON strings. Its parser needs a 4 MiB decoded-string allowance for a legal checkpoint, while nested bundle/checkpoint parsers retain their original limits. It checks derivation against the supplied original, not imported claims about the original. An empty struct enum variant is used for the signature preset because testing showed that a unit variant accepted extra tagged fields despite the intended strict schema.

[Vite's worker documentation](https://vite.dev/guide/features#web-workers) describes the inline-worker build. Keeping the production worker in loaded JavaScript and retaining one WASM buffer lets fresh workers run after disconnection without persistent trust state or a service worker. The [Web Workers guide](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Using_web_workers) informs the dedicated-worker boundary. Replay downloads release their temporary URLs using [URL.revokeObjectURL](https://developer.mozilla.org/en-US/docs/Web/API/URL/revokeObjectURL_static). This is a loaded-session implementation choice, not a promise that an offline page refresh works.

Executable checks confirm mutation/replay parity, imported-verdict isolation, deterministic report reproduction, and zero HTTP requests through the offline workflow. No new BLS primitive, upstream protocol revision, network capture, or authenticated checkpoint was introduced. See [tamper-replay.md](tamper-replay.md).

## C7 genesis RPC inspection findings (2026-10-11)

Pinned `rpc/src/rpc.rs` lines 932–942 returns `WireBlockCertMessage` from the finalized bank's stored genesis certificate. Its test at lines 5322–5357 fixes the JSON fields: `block.slot`, `block.blockId`, `signature.signature`, and `signature.bitmap`. No epoch, shred version, genesis hash, validator ranks/keys/stakes, qualification proofs, or full wire framing accompanies that response. Inspected source hashes and the recording's provenance are in [adapter-profile.json](../fixtures/captures/adapter-profile.json).

The [Solana integration guide](https://solana.com/upgrades/alpenglow) documents null versus certificate output and an unavailable-method error. The adapter distinguishes those observations without treating any recorded response as independent proof of current cluster state. Pinned migration code describes the certified genesis anchor through ancestry/handover logic; Monocle retains that precise scope rather than inferring ordinary finality or the first newly produced block from the RPC label.

The user-supplied slot-444625255 response is now a checked-in recording, with an explicit transcription/trailing-newline note and unknown retrieval time. It is inspectable offline but still lacks authenticated historical context and compatible qualification. No current validator-account ordering or current shred version is substituted. Live endpoint access was not required to implement or demonstrate this read-only path. Native/WASM/browser checks cover exact integers, source identity, strict framing, error distinctions, and context/trust separation.
