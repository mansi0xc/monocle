# Monocle

An open-source finality debugger for Solana's Alpenglow consensus.

Monocle is designed to inspect certificate evidence, check signatures and stake thresholds against an explicitly trusted validator checkpoint, and explain the result. Its tamper lab preserves the original evidence, applies one controlled byte replacement, and compares freshly computed checks. Portable replay files preserve exact evidence and optional checkpoint bytes for verification in a loaded production app, including after disconnecting from the network.

**Project status:** C7 adds recorded genesis RPC inspection and a reproducible graduation demo to the Rust/WASM verifier, tamper lab, and replay workflow. Signed examples remain synthetic; the separate recorded testnet response is displayed as inconclusive because historical context is missing. Real-network verification remains unestablished.

## What the MVP does

1. Import a supported certificate bundle or load a clearly identified synthetic fixture.
2. Select a separately trusted epoch checkpoint and a supported protocol profile.
3. Check encoding, context, signer membership, BLS signatures, stake thresholds, and the evidence needed for a finality conclusion.
4. Explain a result of `VERIFIED`, `REJECTED`, or `INCONCLUSIVE`, including the trust assumptions and individual check results.
5. Compare original and altered evidence and export it for another local verification run.
6. Inspect a recorded genesis RPC response without inventing omitted historical context.

The MVP uses a fixed checkpoint. Authenticating validator-set changes across epochs, transaction execution verification, and production bridge operation are outside its scope. A saved certificate or RPC response is not automatically verified evidence.

## Documentation

| Document | Purpose |
| --- | --- |
| [Idea](docs/idea.md) | Problem, users, distinctive features, scope, and demonstration |
| [Plan](docs/plan.md) | Ordered implementation checkpoints and acceptance criteria |
| [Architecture](docs/architecture.md) | Components, interfaces, trust boundaries, and deployment |
| [Architecture diagrams](docs/architecture.pdf) | Five black-and-white design sheets |
| [Cryptography](docs/cryptography.md) | BLS aggregation, stake arithmetic, payloads, and checkpoint trust |
| [Research](docs/research.md) | Primary sources, observations, and unresolved implementation questions |
| [Testing](docs/testing.md) | Required positive, negative, interoperability, and browser checks |
| [Graduation demo](docs/demo.md) | Setup, a five-minute sequence, explanations, and recovery |
| [Fixture catalog](docs/fixture-catalog.md) | Synthetic cases, recorded evidence, and expected scopes |
| [C7 capture import](docs/capture-import.md) | Raw RPC inspection, local-file command, and missing verification prerequisites |
| [C6 tamper lab and replay](docs/tamper-replay.md) | Controlled mutations, reproducible comparisons, archive format, and offline limits |
| [C5 browser inspector](docs/browser-inspector.md) | Run the app, import evidence, select trust, and understand the worker boundary |
| [C4 finality composition](docs/finality.md) | Fast/slow paths, migration-anchor scope, diagnostics, and assumptions |
| [C3 certificate verification](docs/certificate-verification.md) | Supported wire profile, BLS/PoP checks, exact thresholds, and limits |
| [C2 evidence contracts](docs/contracts.md) | Input schema, bounds, digests, trust selection, and preflight semantics |
| [C1 compatibility experiment](docs/compatibility.md) | Pinned sources, real native/browser checks, and reproducible commands |
| [Development workflow](docs/development-workflow.md) | Owner-controlled commit gates and environment setup |
| [Decisions](docs/decisions.md) | Architectural choices and their tradeoffs |
| [Status](docs/status.md) | What exists and what is next |

## Run the inspector

```sh
npm ci --ignore-scripts --no-audit --no-fund
npm run dev
```

Use Node.js 24.10 or later, the pinned Rust toolchain with WASM/LLVM components, and a WASM-capable Clang. Open the local address Vite prints, load an example, select its checkpoint, and verify. `npm run build:web` builds static assets; `npm run preview` serves them locally. No wallet or RPC connection is required.

For the full implemented gate:

```sh
bash scripts/check-web.sh
```

The gate requires Chrome/Chromium (`CHROME_BIN` can select its executable) and includes native crypto, previous browser corpora, arbitrary-input WASM parity, and inspector interactions/accessibility. See [browser-inspector.md](docs/browser-inspector.md) for setup, trust semantics, limits, and unverified environments.

The Rust library owns verification and composition. `crates/monocle-wasm` exposes a bounded byte interface; `apps/web` manages original inputs, explicit trust, worker lifecycle, and report presentation. The native fixture generator stays separate from the verification path. C6 adds mutation application and replay validation in that same core. C7 adds a read-only genesis RPC adapter and local-file command, sharing the native/WASM core.

## Try a reproducible experiment

Load **Fast finality**, select its synthetic checkpoint, and verify. Open **Tamper lab**, create the signature-bit experiment, then verify the comparison. The original remains unchanged. Export a replay, import it again, explicitly select the checkpoint, and recompute. A saved `VERIFIED` label cannot authorize a result.

For offline use, build and preview the production app, wait for **Ready for offline verification**, and keep that page open. Local imports, experiments, and verification then need no network requests. Offline first visits and page refreshes are not supported; see [tamper-replay.md](docs/tamper-replay.md).

## Inspect recorded network evidence

Choose **Load recorded testnet response** or import a saved `getAgGenesisCert` response. The adapter preserves the source and extracts its fields, then explains why a historical checkpoint, signing context, key qualification, and producer compatibility are needed. Its outcome remains **INCONCLUSIVE**; no guessed context is substituted.

For local-file inspection:

```sh
npm run inspect:capture -- fixtures/captures/testnet-genesis-rpc.json
```

The [demo guide](docs/demo.md) combines the genuine synthetic verification workflow with this separately labeled recording. No live endpoint is needed.

## Development ownership

Mansi creates all commits and pushes. Assistants prepare one checkpoint, write a private handoff under the ignored `commits/` directory, and stop until Mansi confirms the commit. See [AGENTS.md](AGENTS.md) for the repository instructions.
