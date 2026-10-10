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
