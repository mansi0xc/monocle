# Fixture and example catalog

Fixtures serve different purposes. A synthetic signature can be cryptographically valid while making no network claim. A capture can be real recorded data while remaining unverified. Keep those labels visible during a demonstration.

| Collection | Origin and purpose | Executed through |
| --- | --- | --- |
| [BLS compatibility](../experiments/bls-compat/fixtures/README.md) | Deterministic public test keys; primitive and byte-binding proof | Native tests and six browser/native cases |
| [Contracts](../fixtures/contracts/README.md) | Synthetic bounded schemas, exact integers, explicit trust | 13 preflight report comparisons; no crypto verdict |
| [Certificates](../fixtures/certificates/README.md) | 32 independently encoded synthetic Base2 scenarios; real signatures, PoPs, negative cases | Native/C3 browser corpus and core integration tests |
| [Finality](../fixtures/finality/README.md) | 35 synthetic composition scenarios; direct fast/slow paths and separate genesis scope | Native/C4 browser corpus and arbitrary-input bridge |
| [Recorded RPC](../fixtures/captures/README.md) | User-supplied testnet response at slot 444625255; historical authenticity/context unavailable | Read-only native/WASM/browser capture inspection, always inconclusive |

## Built-in signed examples

| Choice | Expected result after explicit synthetic checkpoint selection |
| --- | --- |
| Fast finality | Verified synthetic direct fast path |
| Slow finality | Verified synthetic paired Notar/Finalize path |
| Genesis anchor | Verified synthetic migration-anchor evidence, separate from block finality |
| Missing finalization | Inconclusive block claim despite valid notarization |
| Changed signature | Rejected supplied evidence; dependent checks remain unperformed |
| Insufficient stake | Signature passes, quorum fails; independently signed below-threshold aggregate |

These are expectations for the presenter, not imported authorization. `scripts/prepare-web.mjs` copies only original evidence/context strings into product assets. Browser verification computes its own results. The recorded RPC sample has a separate control and asset so its provenance is not mixed with synthetic examples.

The C6 lab creates derived experiments in memory; it does not alter the checked-in originals. Full C3/C4 fixture regeneration is part of `scripts/check-web.sh`. Replay files and source captures can be shared locally but their saved labels and reports remain untrusted.
