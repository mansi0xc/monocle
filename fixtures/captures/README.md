# Recorded genesis RPC evidence

`testnet-genesis-rpc.json` transcribes the JSON body supplied by Mansi in the project conversation after requesting `getAgGenesisCert` from `https://api.testnet.solana.com`. The certificate's declared slot is 444625255. It is a user-supplied network capture, not a synthetic signed fixture and not independently authenticated evidence.

The body values/order are preserved from the supplied text; this repository file adds a trailing LF. The file digest identifies this transcription, not the original HTTP transfer. Request time, HTTP headers, node version, historical checkpoint, signing context, and compatible key qualification were not supplied. No capture date is inferred from the day the file was added to the repository.

The expected adapter observation is `certificate_present` and the conclusion is `INCONCLUSIVE`, scoped to `capture_inspection`. Signature, quorum, and finality checks do not run. Expected results live in tests, not in the browser's bundled evidence asset. No test result or trusted flag in this directory grants trust.

`adapter-profile.json` records this fixture's identity, source/provenance limits, and hashes of inspected pinned source files. There is no fixture generator: regenerating the recording from today's RPC would create a different observation. Keep this record stable and add a separately attributed recording if needed.

```sh
cargo run --locked --quiet -p monocle-core --bin monocle-capture -- fixtures/captures/testnet-genesis-rpc.json
```

See [capture import](../../docs/capture-import.md) and the [demo guide](../../docs/demo.md).
