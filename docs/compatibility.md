# C1: native Rust and browser WASM feasibility

Status: signature-verification experiment implemented; full certificate verification remains unimplemented.

## What is pinned

The reference is Agave `v4.4.0-beta.0`, commit `98535e50e9bb38574e6b0c31ff284cd864d3ca88`. Its lockfile resolves `solana-bls-signatures 4.0.0`, `blst 0.3.17`, `blstrs 0.7.1`, and `wincode 0.6.1`; the experiment uses those versions as well. [The profile](../experiments/bls-compat/protocol-profile.json) records source hashes, the SDK revision for the BLS crate, encodings, and domain strings.

This revision gives the synthetic probe a precise reference. It does **not** establish that the user's historical testnet capture was produced by this exact revision or is compatible with this probe.

## Architecture decision from the experiment

Reuse the Solana BLS leaf crate with default features and parallel verification disabled. It executes in native Rust and in `wasm32-unknown-unknown` using the same `blst`/`blstrs` backend. No JavaScript cryptographic substitute or stubbed signature check is used.

The full `agave-bls-cert-verify` crate is not imported or claimed to have been compiled. Its source dependency graph includes the broader Votor implementation, and the pinned Agave workspace declares Rust 1.98.1. The smaller experiment works with Rust 1.91.1. The result supports a lightweight adapter approach; certificate parsing, signer maps, quorum, and policy still require their own implementation and reference tests.

## How the experiment works

The native fixture generator derives three publicly known test keys, signs one Notar vote payload, aggregates the signatures, and writes uncompressed public keys, proofs of possession, the payload, and the aggregate signature to JSON. Verification decodes the stored bytes and actually checks every included PoP before creating the qualified key aggregate.

A small hand-written payload adapter is checked against both a fixed expected byte string and a separately defined `wincode` schema matching the pinned Notar/Block fields. This is a source-level serialization check, not execution of Agave's complete certificate verifier.

The Rust library has a deliberately narrow, pointer-free WASM ABI: enumerate a fixed set of embedded probe cases and return `1` for accepted, `0` for cryptographic rejection, or `2` for an experiment/input error. It is not the product's general-purpose input API. C2 now defines the evidence contracts and bounded parsing; the product worker/WASM API remains C5 work. See [contracts.md](contracts.md).

## Reproduce the checks

Prerequisites: Rustup, a WASM-capable Clang, Node.js, and Chrome or Chromium. `rust-toolchain.toml` selects Rust 1.91.1, formatting/lint tools, the WASM target, and LLVM tools. The default browser search supports common macOS/Linux paths; use `CHROME_BIN` for another executable location.

```sh
bash scripts/check-compat.sh
```

The script runs formatting checks, the workspace native tests (including C1's eight integration tests), Clippy with warnings denied, deterministic fixture regeneration, a WASM release build, an unresolved-import check, and real headless-browser execution compared with the native executable. Browser testing uses a temporary profile and a loopback server; both are cleaned up when the run finishes. There is no npm dependency installation step.

Individual commands:

```sh
cargo test --locked
cargo run --locked --quiet --bin probe
bash scripts/build-wasm.sh
node scripts/check-browser.mjs
```

Use `scripts/build-wasm.sh`, not an unqualified direct cross-build, to apply the checked archiver configuration. Set `CC_wasm32_unknown_unknown` if the appropriate Clang is not the one on PATH. Builds are currently demonstrated on macOS arm64, not every host platform.

## The failure that the browser test caught

The first cross-build completed but Chrome could not instantiate its result. The module contained 24 unresolved `env.blst_*` imports. The C backend build log showed Apple's `ranlib` warning that a WASM object was not a Mach-O file; the produced archive lacked the usable symbol index needed by the WASM linker.

The build script now selects the LLVM archiver shipped with the pinned Rust toolchain. The BLS functions then link into the module. A structural check rejects any unresolved imports before the browser test, and the browser independently enforces the same rule before instantiation. No cryptographic implementation was modified to fix the build.

## Entropy and threads

Verification and the fixed input path are deterministic. The upstream dependency graph nevertheless includes `getrandom` through its random-key generation support. For the WASM experiment, an explicit custom backend returns `UNSUPPORTED` if entropy is requested. It never fills a buffer with fake random bytes or claims success. Signing/random-key generation is not a browser feature of this experiment.

This choice avoids unnecessary JavaScript entropy bindings for a verification-only probe. A future feature that actually requires randomness must deliberately supply an appropriate backend. Parallel features remain disabled; no SharedArrayBuffer setup or browser thread pool is required by this tested path.

## Validation observed

- Eight native integration tests passed.
- Genuine three-key aggregate accepted after PoP verification.
- Changed block ID, shred version, signer set, PoP, and signature cases rejected as expected.
- Native and Chrome WASM results match for all six cases.
- Unknown probe index is an error, not a successful verification.
- The module has zero runtime imports.
- Fixture regeneration matches the checked-in bytes exactly.

Per-run environment and detailed outcomes are recorded in the ignored local checkpoint handoff. Timings from this tiny fixture are not presented as a representative certificate-verification benchmark.

## What this does not prove

No rank bitmap is decoded by C1. No historical validator set or stake denominator is authenticated. No stake threshold or finality composition is evaluated. No real testnet certificate is verified. Complete subgroup/identity/encoding edge-case coverage and full upstream differential certificate tests remain for the verifier checkpoint. The experiment page is a developer test harness, not the planned inspector or tamper lab.

C1 was committed as `08274f5`. C2 adds evidence contracts and explicit trusted-context selection; see [status.md](status.md) for the current commit gate.

C2 updates the shared browser runner to wait for an explicit loopback completion report instead of a virtual-time DOM snapshot. Both experiment pages use this test-only mechanism; it sends no evidence to an external service.
