# Signed synthetic certificate corpus

`corpus.json` contains 32 deterministic C3 cases. Every case contains the exact `bundle_json` and `checkpoint_json` text, expected conclusion/stage/code, original signed payload bytes, and original signer ranks. The checkpoint reference hashes the exact checkpoint string. It is not a certificate of authenticity.

Keys are derived from the publicly known seeds `[11; 32]` through `[15; 32]`. They must never be operational keys. The invented rank stakes are `[40, 25, 18, 15, 2]`, with denominator 100, epoch 42, slot 42, and shred version 1234. Signatures and proofs of possession are real BLS operations over this synthetic context.

Reproduce without touching the checked-in file:

```sh
cargo run --locked --quiet -p monocle-certificate-fixtures > target/regenerated-certificates.json
cmp fixtures/certificates/corpus.json target/regenerated-certificates.json
cargo run --locked --quiet -p monocle-certificate-compat
```

The generator uses a reduced upstream wincode schema and the upstream signer-store encoder, not the runtime byte decoder. Its test-only unchecked curve decoders deliberately construct invalid subgroup vectors; the verifier exclusively uses checked conversions. The fixture tests establish that those negative vectors are on-curve but outside the prime-order subgroup, so they do more than test malformed encodings.

Coverage includes all four supported kinds; exact 80% and 82% positives; separately signed under-quorum examples; changed payload, bitmap, and signed context; malformed/identity/non-subgroup points; missing, malformed, and wrong-key PoPs; unknown versions/kinds; Base3; empty/unknown-rank/padded bitmaps; trailing bytes; envelope/claim contradictions; and deferred block/migration claims. Additional native tests cover exact 60/80/82% arithmetic boundaries, all wire truncations, u64::MAX, denominator preservation, trust separation, and mixed bundles.

`signed_payload_hex` and `original_signer_ranks` identify what was signed before a tamper operation. In negative cases they can deliberately differ from the bytes/ranks decoded from the final bundle. Never reinterpret a changed bitmap under an old signature as a quorum-only test.

These files are synthetic test evidence. They are not exported testnet certificates, authenticated checkpoints, or claims about real blocks. The corpus wrapper is a harness format; its contained bundle/checkpoint strings use the public contracts.
