# C4 synthetic evidence composition corpus

`corpus.json` contains 35 end-to-end claim scenarios. Each includes original bundle/checkpoint JSON strings, an explicit test-harness checkpoint selection choice, and expected conclusion, established scope, path, witness IDs, and diagnostic. The selection flag is a fixture-runner instruction, not part of the imported public evidence contract.

The generator composes the independently encoded [C3 fixtures](../certificates/README.md), preserving their public seeds and invented context. Two additional Finalize aggregates use the same reduced reference schema: ranks `[0, 2, 3, 4]` carry 75/100 stake for the different-signers slow path, and `[0, 2]` carry 58/100 stake for a genuinely signed under-quorum finalize example. No generator depends on `monocle-core`.

```sh
cargo run --locked --quiet -p monocle-certificate-fixtures --bin generate_finality > target/regenerated-finality.json
cmp fixtures/finality/corpus.json target/regenerated-finality.json
cargo run --locked --quiet -p monocle-finality-compat
```

Cases cover valid fast/slow/genesis claims, reversed order and differing signers, missing counterparts or block targets, contradictory slots/blocks/context, incompatible ordinary/migration scope, insufficient stake on each path, invalid signatures, missing PoPs, unsupported bitmap/kind/format/profile/claim, rejected saved-report injection, missing checkpoint selection, duplicate notarizations, and bad/unsupported extras alongside an otherwise valid fast certificate.

The wrong-slot/block cases deliberately alter bytes and contradict the claim. They fail the declaration check before signature verification; they are not advertised as valid signatures about another target. All successful paths and quorum-only negative cases use genuine signatures. Genesis successes refer only to the synthetic migration anchor, never actual network activation.

The corpus wrapper is a development format. Product consumers use the contained bounded contracts and the separate trust API, not the wrapper's expected results.
