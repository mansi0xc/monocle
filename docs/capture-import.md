# Recorded genesis RPC evidence

C7 adds a bounded, read-only adapter for a recorded `getAgGenesisCert` JSON-RPC response. It extracts the certificate fields while preserving the original file and explains what is missing before cryptographic verification can run. This adapter always reports `INCONCLUSIVE` with scope `capture_inspection`; it does not turn an RPC response into a trusted proof bundle.

## Browser workflow

Choose **Load recorded testnet response** under **Recorded RPC evidence**, or use **Import genesis RPC response** for a local file. The bundled recording is the response supplied by the project owner in the conversation, transcribed into [the capture fixture](../fixtures/captures/testnet-genesis-rpc.json). Its reported slot is `444625255`. Retrieval time, original transport bytes, endpoint identity, and signature validity have not been independently established.

Capture import clears previous bundle/experiment reports and checkpoint selection before reading the file. The inspector displays the exact-source digest, extracted slot and block ID, missing prerequisites, and the distinction between structural extraction and unperformed verification. Signature and bitmap bytes are available in a disclosure. They are byte arrays, not qualified signatures or authenticated signer identities.

**Download original response** returns the exact imported bytes. Import that file again to reproduce the inspection. This is a raw RPC recording, separate from C6's `monocle-replay` archive and Monocle proof-bundle schemas. It carries no saved verdict or trust flag. A loaded production page supports this workflow offline, including the built-in recording; first visits and refreshes still require assets to be available.

The capture adapter does not consume a checkpoint or offer a context-guessing form. The checkpoint input is disabled while viewing a capture. Load a supported Monocle bundle to return to the separately implemented full verification workflow. This prevents an unrelated synthetic or current checkpoint from being presented as the missing historical evidence.

## Native workflow

```sh
npm run inspect:capture -- fixtures/captures/testnet-genesis-rpc.json
```

The direct command, suitable for structured output without npm's script banner, is:

```sh
cargo run --locked --quiet -p monocle-core --bin monocle-capture -- fixtures/captures/testnet-genesis-rpc.json
```

Use `-` as the path to read standard input. The command reads at most 64 KiB + one byte, writes its report to standard output, and never fetches network data, selects trust, modifies input files, or signs anything. Exit 0 means the response was structurally inspected; the report remains `INCONCLUSIVE`. Exit 2 means usage, input, parsing, or output failed. Check the report scope and conclusion rather than treating command success as proof of finality.

## Supported envelope and limits

The adapter accepts one UTF-8 JSON object with `jsonrpc: "2.0"`, an `id`, and exactly one of `result` or `error`. An ID can be null, an integer, or a string of at most 128 UTF-8 bytes. Unknown fields and duplicate keys fail. JSON-RPC does not echo a method name; choosing this adapter is a local declaration that the response came from `getAgGenesisCert`, not cryptographic evidence of that fact.

| Observation | Required shape | Meaning in Monocle |
| --- | --- | --- |
| Certificate present | `result.block` and `result.signature` | Fields extracted; historical context still missing |
| No certificate | `result: null` | This endpoint recording contains no certificate; no independent cluster-state conclusion |
| Method unavailable | `error.code: -32601` | Method error, distinct from a successful null response |
| Other RPC error | Integer error code and message | Recorded endpoint failure, not certificate evidence |

A present certificate requires:

- `block.slot`: a JSON unsigned integer within u64. Rust parses it without JavaScript number conversion and returns a decimal string.
- `block.blockId`: exactly 32 integer bytes in `0..255`.
- `signature.signature`: exactly 192 integer bytes. Length checking does not establish curve-point or signature validity.
- `signature.bitmap`: 1–413 integer bytes. This resource bound matches the existing profile's maximum bounded signer representation; the capture adapter does not decode it into authenticated ranks or stake.

The full file limit is **64 KiB**. Normal strict parser depth/node/array/object/key limits also apply. RPC error messages are limited to 1024 UTF-8 bytes; optional error `data` is retained only in the original source, not interpreted. Bare results, batched responses, block footers, Monocle bundles, saved reports, unknown field extensions, and changed array encodings require a different deliberately supported adapter.

The slot is taken only from `result.block.slot`. A transport or containing-footer slot is never substituted; footer-like envelopes are rejected. No epoch schedule or shred version is guessed from the number.

## What remains necessary for verification

The RPC response does not supply these prerequisites:

1. An independently authenticated historical checkpoint with the canonical rank ordering, matching BLS keys, exact stake values, and original denominator.
2. The actual historical signing context: shred version, epoch, and genesis hash, with evidence of their relationship to the certificate. A current bank or current account list does not establish the historical context.
3. Qualification evidence compatible with the implemented PoP policy. A vote-account registration proof that signs a different application payload cannot silently replace the verifier's standard public-key-bound PoP.
4. Evidence that the producer's message serialization and signer encoding match the chosen verification profile. The response does not carry the full versioned wire framing or its shred-version suffix.

C19 supplies these for a cluster you capture yourself, with stated assumptions: a rebuilt rank map, the recorded context, registered-key trust, and the re-encoded wire message. See [mainnet-genesis.md](mainnet-genesis.md). The bundled testnet recording is too old to rebuild from RPC and remains unverified. C7 does not invent a wire message or placeholder context. The supplied recording remains unverified; the genuine synthetic BLS/quorum/finality demos continue to exercise the existing verifier independently.

## Implementation and test evidence

`monocle_core::capture::inspect_genesis_rpc` is the shared native/WASM parser and report producer. Additive operation **9** in the existing ABI **v2** exposes it to a fresh browser worker. It takes the original file bytes, has no trust-store argument, and does not modify the bridge's existing trust state. The product controller independently clears its prior trust/input state when a capture is imported. The adapter ID is `agave-genesis-rpc-v1`; it is not a verified protocol-profile attestation.

Eight native tests cover the recording, null/errors, exact u64 handling, strict byte arrays, unknown/contradictory fields, footer confusion, parser limits, and CLI behavior. Eight complete native/WASM comparisons use the actual CLI and bridge, including malformed input and maximum u64. Six browser tests cover replacement of prior success, original-byte download/re-import, hostile endpoint text, bounds, exact digest/slot transport, offline zero-request operation, responsive accessibility, and late file reads. Run the complete gate with `bash scripts/check-web.sh`.

## Sources and attribution

The pinned [Agave RPC implementation](https://github.com/anza-xyz/agave/blob/98535e50e9bb38574e6b0c31ff284cd864d3ca88/rpc/src/rpc.rs) reads the finalized bank's genesis certificate and returns block plus aggregate-signature/bitmap fields; its RPC test fixes that output shape. [Wire definitions](https://github.com/anza-xyz/agave/blob/98535e50e9bb38574e6b0c31ff284cd864d3ca88/votor-messages/src/wire.rs) and [migration semantics](https://github.com/anza-xyz/agave/blob/98535e50e9bb38574e6b0c31ff284cd864d3ca88/votor-messages/src/migration.rs) explain why omitted signing context cannot be recovered from the response alone. Source hashes are in [the adapter record](../fixtures/captures/adapter-profile.json).

The [Solana integration guide](https://solana.com/upgrades/alpenglow) documents the method, response fields, and distinction between a null response and an unavailable method. Monocle retains the precise genesis/migration-anchor label from its pinned source rather than claiming that a recorded response proves activation or ordinary block finality. No current endpoint availability or rollout state is inferred from the stored file.
