# Monocle

An open-source finality debugger for Solana's Alpenglow consensus.

Monocle is designed to inspect certificate evidence, check signatures and stake thresholds against an explicitly trusted validator checkpoint, and explain the result. Its planned tamper lab preserves the original evidence, applies a controlled change, and shows which verification checks change. Portable bundles make experiments reproducible offline.

**Project status:** design foundation prepared. The Rust verifier, browser application, capture tooling, and automated tests have not been implemented yet. This repository does not currently verify certificates.

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
| [Development workflow](docs/development-workflow.md) | Owner-controlled commit gates and environment setup |
| [Decisions](docs/decisions.md) | Architectural choices and their tradeoffs |
| [Status](docs/status.md) | What exists and what is next |

## Planned implementation

A Rust library will own verification and finality evaluation. A small native tool will prepare fixtures and capture files; a WASM build will run the same verification logic in a browser worker. A TypeScript interface will handle inspection, mutations, comparison, and local import/export. Library selection and native/WASM compatibility must be established in the first implementation checkpoint before this structure is committed to in code.

There is no installation or application start command yet. Reproducible build and test instructions will be added with the corresponding implementation.

## Development ownership

Mansi creates all commits and pushes. Assistants prepare one checkpoint, write a private handoff under the ignored `commits/` directory, and stop until Mansi confirms the commit. See [AGENTS.md](AGENTS.md) for the repository instructions.
