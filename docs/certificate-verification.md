# C3 certificate verification

Implemented: 2026-10-11. Profile: `agave-98535e50-base2-v1`.

`monocle_core::certificate::verify_certificates` accepts a bounded `VerifyRequest` and a separate local `TrustStore`. It checks supported individual certificates against the selected checkpoint. It performs real BLS verification; the verifier never signs, fetches context, or trusts a result imported with a bundle.

`VERIFIED` here means **individual certificate checks passed under the selected checkpoint**. The report has `evidence_scope: individual_certificates`. In this certificate-only API, block-finality and genesis-migration claims remain `INCONCLUSIVE`, even when every individual certificate passes. C4's separate `finality::verify_evidence` API now composes those claims; see [finality.md](finality.md). C2's `preflight` API retains its separate `preflight_only` scope and never returns `VERIFIED`.

## Supported representation

The envelope format is `agave-versioned-consensus-v1`: one complete `VersionedWireConsensusMessage::V1`, serialized with the pinned wincode default configuration. It is not RPC JSON, a block footer, or an unversioned inner certificate. No source format is guessed from similar-looking bytes.

| Certificate kind | Certificate tag | Signed vote tag | Payload after the vote tag | Required stake |
| --- | --- | --- | --- | --- |
| `notarization` | 9 | 1 | slot + block ID + shred version | ≥60% |
| `finalization` | 7 | 2 | slot + shred version | ≥60% |
| `fast_finalization` | 8 | 1 | slot + block ID + shred version | ≥80% |
| `genesis` | 12 | 6 | slot + block ID + shred version | ≥82% |

All tags are u8. Slots are u64 little-endian, block IDs are 32 raw bytes, and shred versions are u16 little-endian. Notarization, fast-finalization, and genesis payloads are 43 bytes; finalization payloads are 11 bytes. Fast-finalization certificates aggregate notarization votes. A finalize vote does not sign a block ID.

The containing message is:

```text
version 1 (u8)
certificate tag (u8)
slot (u64 LE)
block ID (32 bytes, absent for finalization)
aggregate signature (192 uncompressed bytes)
bitmap byte length (u64 LE)
bitmap bytes
shred version (u16 LE)
```

The decoder requires complete consumption and rejects truncation, trailing data, and excessive vector lengths before allocation. The bitmap allocation cap is 413 bytes, enough to classify a bounded Base3 input as unsupported; Base2's maximum encoded size is 259 bytes for the existing 2,048-validator container cap. The C2 bundle, request, certificate-count, and checkpoint limits still apply.

The declared envelope kind must match the wire tag. Decoded slot/block values must agree with the claim. An inspection claim cannot attach a block ID to a slot-only finalization certificate. Wire shred version must match the checkpoint; the checkpoint and bundle must also agree on declared epoch and genesis hash. **Epoch and genesis hash are context assumptions, not fields signed by these vote payloads.** No slot-to-epoch or chain-membership proof is implemented.

## Signers, keys, signatures, and stake

1. Decode the Base2 bitmap with upstream `solana-signer-store 0.1.0`: version byte 0, u16 little-endian bit count, then least-significant-bit-first packed bits. Unknown ranks, wrong lengths, and empty selections fail. Base3 and unsupported certificate variants remain inconclusive.
2. Preserve checkpoint rank order. Each set bit selects one distinct rank; no sorting or denominator reduction occurs. The input parser already rejects duplicate identities/keys and noncontiguous ranks.
3. Decode every selected G1 public key using the checked Solana BLS API. Malformed, off-curve, identity, and non-prime-order keys cannot qualify.
4. Require and verify that key's actual G2 proof of possession. The policy is the SDK's standard compressed-public-key-bound PoP with **no extra application payload**. A missing proof is inconclusive; an invalid proof fails. No Boolean label or unsafe qualification shortcut replaces verification. Qualification results are cached only within this request and checkpoint.
5. Decode the aggregate G2 signature with checked point/subgroup validation, explicitly reject identity, aggregate qualified keys, and verify the exact reconstructed source-vote bytes using `solana-bls-signatures 4.0.0`. The upstream verification path also rejects an identity aggregate public key.
6. Sum selected stakes with checked u64 addition, retain the checkpoint's original denominator, then compare `u128(selected) * 100 >= u128(total) * threshold_percent`. With u64 inputs and thresholds below 100, those products cannot overflow u128. No floating-point arithmetic decides quorum.

Only participating keys need PoP qualification for this certificate. An unselected key without a proof does not remove its weight from the denominator. `selected_stake` reports the bitmap-selected weight; it is not evidence of a valid signature until the signature check passes.

One intentional stricter rule: Monocle rejects nonzero unused Base2 padding bits. Upstream truncates those bits. The native tests demonstrate this difference explicitly; this is a narrower profile, not a claim to accept every input that Agave accepts.

## Trust and qualification limits

`TrustStore.pin` is a separate caller action. This proves neither checkpoint authenticity nor the correctness of its rank ordering, stake values, denominator, epoch, or relationship to a real cluster. A valid standard PoP proves possession of a BLS secret key; it does not prove vote-account registration, validator identity ownership, or stake entitlement. Existing on-chain registration proofs may bind additional application data and are not automatically usable under this PoP policy.

Synthetic fixtures use invented context and public test keys. The supplied testnet genesis RPC response at slot 444625255 remains unverified. A future capture adapter needs the matching historical checkpoint, precise source framing, and compatible qualification evidence. C3 does not convert that JSON response into a verified certificate.

## Result details

`CertificateVerificationReport` adds per-certificate raw SHA-256 identity, decoded kind/slot/block, reconstructed signed bytes, signer ranks, exact selected/total stake, threshold, and five ordered checks: decoding, signers, key qualification, signature, quorum. Global checks cover input, profile, context, and the explicitly unperformed finality stage. `CertificateReport` fields that cannot yet be computed are null or empty; later checks remain `NOT_CHECKED` after a prerequisite fails.

- Invalid supported bytes, contradictory declarations, malformed points, false PoPs, bad signatures, or insufficient stake: `REJECTED`.
- Missing context or PoP, unsupported profile/version/encoding, or a deferred finality/migration claim: `INCONCLUSIVE`.
- Inspection with every selected certificate passing: `VERIFIED`, scoped to individual certificates.

One failed certificate prevents a mixed bundle from receiving a successful overall inspection. A rejected bundle is not a chain-level proof that a block can never finalize. Reports are outputs, not importable trust tokens.

## Compatibility evidence and reproducibility

```sh
npm ci --ignore-scripts --no-audit --no-fund
bash scripts/check-certificates.sh
```

The script runs the prior checkpoints' checks, 15 new native integration tests, deterministic C3 fixture regeneration, TypeScript shape checks, the WASM build/import gate, and a real Chrome/Chromium comparison of 32 full C3 reports with native Rust. Use `CHROME_BIN` if necessary. The product inspector and worker remain C5 work; these browser pages are development harnesses.

The [fixture generator](../tools/certificate-fixtures/src/main.rs) does not depend on `monocle-core`. It uses a reduced [wincode reference schema](../tools/certificate-fixtures/src/reference.rs) adapted from pinned Agave source, while the runtime decoder is a separate bounded byte adapter. Tests compare all four wire layouts and signed payloads against that schema and explicit fixed-byte expectations at slots 0, 42, and u64::MAX. Signer decoding uses the actual upstream crate, with tests comparing canonical bitmaps and the documented padding difference. The BLS backend is the actual Solana SDK leaf crate.

This is source-reviewed adapter compatibility with independently encoded fixtures, **not execution of the full Agave certificate verifier**. Both paths share the reviewed protocol assumptions and crypto backend. Full upstream differential execution, real captures, fallback/multi-message certificates, cross-epoch authentication, broader composition paths, arbitrary-input browser APIs, and production security auditing remain outside this checkpoint.

## Pinned sources

The [profile record](../experiments/certificate-compat/protocol-profile.json) records exact revisions, versions, and inspected-source hashes.

- [Agave wire schema and vote serialization](https://github.com/anza-xyz/agave/blob/98535e50e9bb38574e6b0c31ff284cd864d3ca88/votor-messages/src/wire.rs).
- [Certificate thresholds and source votes](https://github.com/anza-xyz/agave/blob/98535e50e9bb38574e6b0c31ff284cd864d3ca88/votor-messages/src/certificate.rs), [genesis threshold](https://github.com/anza-xyz/agave/blob/98535e50e9bb38574e6b0c31ff284cd864d3ca88/votor-messages/src/migration.rs).
- [Upstream verification order and qualified-key inputs](https://github.com/anza-xyz/agave/blob/98535e50e9bb38574e6b0c31ff284cd864d3ca88/bls-cert-verify/src/cert_verify.rs), [epoch rank/stake construction](https://github.com/anza-xyz/agave/blob/98535e50e9bb38574e6b0c31ff284cd864d3ca88/runtime/src/epoch_stakes.rs).
- [Signer-store 0.1.0 source](https://docs.rs/crate/solana-signer-store/0.1.0/source/src/lib.rs), [Solana BLS 4.0.0 source](https://docs.rs/crate/solana-bls-signatures/4.0.0/source/src/lib.rs), [blstrs 0.7.1 checked G1 decoding](https://docs.rs/crate/blstrs/0.7.1/source/src/g1.rs) and [G2 decoding](https://docs.rs/crate/blstrs/0.7.1/source/src/g2.rs).
