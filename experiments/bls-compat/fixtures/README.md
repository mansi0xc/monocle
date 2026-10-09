# C1 synthetic aggregate fixture

`aggregate.json` contains public synthetic evidence, not a network capture. Three deterministic test keys are derived from the public 32-byte seeds `[1; 32]`, `[2; 32]`, and `[3; 32]`. Never use these seeds for operational keys.

All three keys sign the same 43-byte Notar payload: tag 1, slot 42, block ID `[0x42; 32]`, shred version 1234. The individual signatures are aggregated by `solana-bls-signatures 4.0.0` and serialized as a 192-byte uncompressed signature. Each stored 96-byte public key is qualified by verifying its stored 192-byte proof of possession before aggregation.

This fixture carries no stake, validator identity, signer bitmap, or authentic epoch-set claim. The proof of possession uses the upstream compressed-public-key payload with no extra application payload; it is not a claim about production vote-account registration.

From the repository root:

```sh
mkdir -p target
cargo run --locked --quiet --bin generate_fixture > target/regenerated-fixture.json
cmp experiments/bls-compat/fixtures/aggregate.json target/regenerated-fixture.json
```

The generator prints only public fixture artifacts. Private scalars are not exported. The verifier consumes the checked-in bytes rather than generating fresh signatures before accepting them.

| Probe index | Transformation | Expected result |
| --- | --- | --- |
| 0 | Original aggregate and all three qualified keys | Accept |
| 1 | Flip one bit of the signed block ID | Signature rejection |
| 2 | Flip one bit of the signed shred version | Signature rejection |
| 3 | Omit the third key from the verifying aggregate | Signature rejection |
| 4 | Replace the first proof of possession with zero bytes | PoP rejection |
| 5 | Replace the aggregate signature with zero bytes | Signature rejection |

Any other index returns a probe error, distinct from a cryptographic rejection. The missing-signer case is not an insufficient-stake demonstration; there is no stake logic in C1.
