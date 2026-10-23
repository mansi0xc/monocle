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
| Changed signer set | Rejected supplied evidence: the bitmap names different signers, so the unchanged aggregate fails `INVALID_SIGNATURE`; dependent checks remain unperformed. Fixture ID `changed_signature` is kept for compatibility. |
| Insufficient stake | Signature passes, quorum fails; independently signed below-threshold aggregate |

`fixtures/genesis-capture/synthetic/` holds a synthetic RPC capture: `validators/` and `genesis-cert/`, each with a digest manifest. `built/` holds the checkpoint, bundle and summary the builder produces from it. The cluster exercises every admission and rank-map exclusion and a stake tie, and its genesis certificate is genuinely signed by public test keys. It verifies only under registered-key trust. `fixtures/genesis-capture/mainnet-vote-account.json` is one real mainnet vote account, kept only as a parser test vector.

`fixtures/genesis-capture/mainnet/` is real (C21). It holds `validators/`, recorded from `api.mainnet-beta.solana.com` early in epoch 1063, and `genesis-cert/`, recorded after migration. Both are kept exactly as recorded, with their digest manifests. `built/` holds the epoch-1064 checkpoint (673 validators, digest `sha256:a83a1b6f…520194`), the genesis bundle for slot 459,652,999 and the summary. The certificate is **VERIFIED** only under registered-key trust; see [mainnet-genesis.md](mainnet-genesis.md#result-2026-10-21) for its assumptions. The endpoint was not authenticated. `validators/003-cluster-nodes.json` lists the public network addresses of 3,912 nodes, as `getClusterNodes` returned them.

These are expectations for the presenter, not imported authorization. `scripts/prepare-web.mjs` copies only original evidence/context strings into product assets. Browser verification computes its own results. The recorded RPC sample has a separate control and asset so its provenance is not mixed with synthetic examples.

The C6 lab creates derived experiments in memory; it does not alter the checked-in originals. Full C3/C4 fixture regeneration is part of `scripts/check-web.sh`. Replay files and source captures can be shared locally but their saved labels and reports remain untrusted.
