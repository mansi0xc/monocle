# Monocle

An open-source finality debugger for Solana's Alpenglow consensus.

Monocle is designed to inspect certificate evidence, check signatures and stake thresholds against an explicitly trusted validator checkpoint, and explain the result. Its planned tamper lab preserves the original evidence, applies a controlled change, and shows which verification checks change. Portable bundles make experiments reproducible offline.

**Project status:** C4 verifies direct fast/slow block-finality evidence and separately scoped genesis migration anchors under an explicitly trusted checkpoint. The Rust core checks real BLS signatures, proofs of possession, individual stake thresholds, and evidence composition. Synthetic end-to-end reports agree between native Rust and browser WASM. The product inspector, tamper/replay workflow, and real-network verification remain unimplemented.

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
| [C4 finality composition](docs/finality.md) | Fast/slow paths, migration-anchor scope, diagnostics, and assumptions |
| [C3 certificate verification](docs/certificate-verification.md) | Supported wire profile, BLS/PoP checks, exact thresholds, and limits |
| [C2 evidence contracts](docs/contracts.md) | Input schema, bounds, digests, trust selection, and preflight semantics |
| [C1 compatibility experiment](docs/compatibility.md) | Pinned sources, real native/browser checks, and reproducible commands |
| [Development workflow](docs/development-workflow.md) | Owner-controlled commit gates and environment setup |
| [Decisions](docs/decisions.md) | Architectural choices and their tradeoffs |
| [Status](docs/status.md) | What exists and what is next |

## Planned implementation

The Rust library owns individual certificate verification and direct evidence composition. A small native tool will prepare fixtures and capture files; a WASM build will run the same verification logic in a browser worker. A TypeScript interface will handle inspection, mutations, comparison, and local import/export. C1 establishes a working native/WASM path using the Solana BLS leaf crate; C2 adds `crates/monocle-core` for input and context preparation and `packages/contracts` for TypeScript transport. C3 adds the certificate verifier, an independent signed-fixture generator, and a browser certificate harness. C4 adds the scoped evidence evaluator and composition corpus. The product interface remains planned.

Run the current checks from the repository root:

```sh
npm ci --ignore-scripts --no-audit --no-fund
bash scripts/check-finality.sh
```

It requires Rustup, a WASM-capable Clang, Node.js, and Chrome/Chromium (`CHROME_BIN` can select its executable). The pinned toolchain includes the WASM target and LLVM tools. See [finality.md](docs/finality.md) and [compatibility.md](docs/compatibility.md) for commands and limits. There is no product application start command yet.

## Development ownership

Mansi creates all commits and pushes. Assistants prepare one checkpoint, write a private handoff under the ignored `commits/` directory, and stop until Mansi confirms the commit. See [AGENTS.md](AGENTS.md) for the repository instructions.
