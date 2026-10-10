# Third-party sources and licenses

Monocle consumes established cryptographic implementations as Cargo dependencies; it does not vendor or replace their cryptographic primitives. Exact package checksums and transitive versions are in `Cargo.lock`.

| Component | Pinned version | Declared license | Role |
| --- | --- | --- | --- |
| solana-bls-signatures | 4.0.0 | Apache-2.0 | Solana signature and proof-of-possession API |
| blst | 0.3.17 | Apache-2.0 | BLS12-381 native/WASM cryptographic backend |
| blstrs | 0.7.1 | MIT OR Apache-2.0 | Rust curve and pairing wrappers |
| solana-signer-store | 0.1.0 | Apache-2.0 | Runtime Base2 decoding and development fixture encoding |
| bitvec | 1.1.1 | MIT | Packed-bit reference fixtures; transitive signer-store dependency |
| wincode | 0.6.1 | Apache-2.0 | Test-only independent schema serialization |
| serde | 1.0.228 | MIT OR Apache-2.0 | Fixed fixture decoding and reporting |
| serde_json | 1.0.145 | MIT OR Apache-2.0 | JSON fixture/report transport for the experiment |
| hex | 0.4.3 | MIT OR Apache-2.0 | Human-readable fixture bytes |
| sha2 | 0.10.9 | MIT OR Apache-2.0 | Exact-source SHA-256 identities in C2 |
| serde_path_to_error | 0.1.20 | MIT OR Apache-2.0 | Bounded input diagnostic field paths |
| TypeScript | 5.9.3 | Apache-2.0 | Development-only boundary type checking; pinned in package-lock.json |
| getrandom | 0.2.16 | MIT OR Apache-2.0 | Target-specific fail-closed entropy hook; no randomness is needed for verification |

These are license declarations inspected in the package manifests, not a security audit or complete legal analysis of every transitive dependency.

The Notar payload layout and the small reference schema in `experiments/bls-compat/tests/compat.rs` follow the Apache-2.0 licensed Agave source at commit `98535e50e9bb38574e6b0c31ff284cd864d3ca88`:

- [Vote payload definition](https://github.com/anza-xyz/agave/blob/98535e50e9bb38574e6b0c31ff284cd864d3ca88/votor-messages/src/wire.rs)
- [Block field layout](https://github.com/anza-xyz/agave/blob/98535e50e9bb38574e6b0c31ff284cd864d3ca88/votor-messages/src/consensus_message.rs)
- [Upstream license copy](third_party/agave/LICENSE)

The adapted reference uses a reduced Notar-only enum and local field types. The runtime serializer is an explicit 43-byte adapter, not an imported full Agave vote/certificate decoder. The source revisions, inspected-file hashes, domain strings, and formats are recorded in [the experiment profile](experiments/bls-compat/protocol-profile.json).

Project publication licensing remains an owner decision; this notice does not assign a license to all Monocle-authored code.

C3 adapts the four-kind complete wire layout and source-vote layout in `crates/monocle-core/src/certificate/wire.rs` and `tools/certificate-fixtures/src/reference.rs` from the same Apache-2.0 Agave revision. The reference is reduced to four certificate kinds and three vote payloads, with local field types. The runtime adapter adds allocation/consumption limits; Base2 decoding directly uses the upstream signer-store crate and adds a documented stricter padding check. The existing upstream license copy applies to this attribution as well. See [C3 profile and inspected-source hashes](experiments/certificate-compat/protocol-profile.json).
