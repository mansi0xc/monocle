# Monocle

An open-source finality debugger for Solana's Alpenglow consensus.

Monocle is designed to inspect certificate evidence, check signatures and stake thresholds against an explicitly trusted validator checkpoint, and explain the result. Its tamper lab preserves the original evidence, applies one controlled byte replacement, and compares freshly computed checks. Portable replay files preserve exact evidence and optional checkpoint bytes for verification in a loaded production app, including after disconnecting from the network.

**Project status:** C6 adds the tamper lab, diagnostic comparison, and portable replay to the black-and-white Rust/WASM inspector. Importing a replay never selects trust or restores saved verdicts. Fast/slow finality and genesis-anchor evidence retain their distinct scopes. Built-in examples are synthetic; real-network verification remains unestablished.

## What the MVP will do

1. Import a supported certificate bundle or load a clearly identified synthetic fixture.
2. Select a separately trusted epoch checkpoint and a supported protocol profile.
3. Check encoding, context, signer membership, BLS signatures, stake thresholds, and the evidence needed for a finality conclusion.
4. Explain a result of `VERIFIED`, `REJECTED`, or `INCONCLUSIVE`, including the trust assumptions and individual check results.
5. Compare original and altered evidence and export it for another local verification run.

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

The Rust library owns verification and composition. `crates/monocle-wasm` exposes a bounded byte interface; `apps/web` manages original inputs, explicit trust, worker lifecycle, and report presentation. The native fixture generator stays separate from the verification path. C6 adds mutation application and replay validation in that same core. The real-capture adapter remains C7 work.

## Try a reproducible experiment

Load **Fast finality**, select its synthetic checkpoint, and verify. Open **Tamper lab**, create the signature-bit experiment, then verify the comparison. The original remains unchanged. Export a replay, import it again, explicitly select the checkpoint, and recompute. A saved `VERIFIED` label cannot authorize a result.

For offline use, build and preview the production app, wait for **Ready for offline verification**, and keep that page open. Local imports, experiments, and verification then need no network requests. Offline first visits and page refreshes are not supported; see [tamper-replay.md](docs/tamper-replay.md).

## Development ownership

Mansi creates all commits and pushes. Assistants prepare one checkpoint, write a private handoff under the ignored `commits/` directory, and stop until Mansi confirms the commit. See [AGENTS.md](AGENTS.md) for the repository instructions.
