# Project status

Updated: 2026-10-16.

## Current state

Mansi committed C10, the design study, as `9d2566e` and `a6d562b` and chose the Dusk direction. C11 implements Dusk in the product: an indigo/alpenglow palette, locally bundled Schibsted Grotesk and Red Hat Mono, a landing page built from real fixture outcomes, and a three-column inspector (evidence and trust, verdict-first report, experiments). Mansi committed it as `048c3a7`. C12 (committed as `8bc8708`) added Newsreader as a serif display face for headlines, section titles and verdicts, so headings no longer share the interface font. C13 (committed as `f93b6fe`) added explanatory motion and interaction. C14 (committed as `fff1251`) reorganised the inspector to show one step at a time. Finished steps collapse, Verify moves into a run bar beside the report, experiments appear once trust is selected, and less-used imports sit behind disclosures. The landing ridge climbs signer by signer and switches between four recorded fixture scenarios, a "Break it on purpose" demo replays the recorded one-byte experiment, and inspector runs reveal their trace in order. The verification model, trust rules, tamper/replay and capture behaviour are unchanged. Colosseum feature exploration is paused.

The landing page restates synthetic fixture results with their diagnostic codes and labels them as such. It cannot load evidence or grant trust. The actual inspector preserves its in-memory evidence and checkpoint choices when navigating home and back; refreshing still discards the session.

The recorded response at slot `444625255` remains **INCONCLUSIVE**. Its historical checkpoint, signing context, producer compatibility, and qualifying proofs have not been established. Capture inspection does not run signature, quorum, or finality checks and never substitutes an unrelated checkpoint.

The genuine Rust/WASM verification paths, immutable tamper experiments, fresh diagnostic comparison, and portable replay remain available for supported Monocle bundles. Fast/slow block finality and genesis-anchor evidence retain their distinct scopes. Successful demonstrations use synthetic public test keys and invented context. No real testnet certificate is cryptographically verified.

## Checkpoint progress

| Checkpoint | State |
| --- | --- |
| C0 - Foundation | Committed by Mansi: `ad561d4` |
| C1 - Protocol/native/WASM feasibility | Committed by Mansi: `08274f5` |
| C2 - Evidence contracts and trust selection | Committed by Mansi: `39399a6` |
| C3 - Native verification and signed fixtures | Committed by Mansi: `06b6a8a` |
| C4 - Finality composition | Committed by Mansi: `8fe23fa` |
| C5 - Browser inspector | Committed by Mansi: `2ab8e72` |
| C6 - Tamper lab and replay | Committed by Mansi: `95a990a` |
| C7 - Capture integration | Committed by Mansi: `e396ade` |
| C8 - Landing page and product UI | Committed by Mansi: `75cbd85`; merge confirmed |
| C9 - Reference logo and palette | Committed by Mansi: `8c10872`; merged |
| C10 - Design directions | Committed by Mansi: `9d2566e`, `a6d562b`; Dusk chosen |
| C11 - Dusk implementation | Committed by Mansi: `048c3a7` |
| C12 - Display typeface | Committed by Mansi: `8bc8708` |
| C13 - Motion and interaction | Committed by Mansi: `f93b6fe` |
| C14 - Step-by-step inspector | Committed by Mansi: `fff1251` |
| C15 - Checkpoint defects stay inconclusive | Implemented; awaiting owner review/commit |

C10 added three static design-direction previews under `apps/design-study/`; the owner chose Dusk, which C11 implements.

## Run and validate

Use `npm run dev` for development, or `npm run build:web` and `npm run preview` for production-session offline use. Open the inspector and wait for **Ready for offline verification**. Local verification, replay, and recorded capture inspection need no further network requests while that session stays open. Visiting only the landing page does not load the verifier. Offline first visits/refreshes and offline development-server use are unsupported.

Use `npm run inspect:capture -- fixtures/captures/testnet-genesis-rpc.json` for native capture inspection. Exit 0 means parsed input, not cryptographic success. See [capture-import.md](capture-import.md), [interface.md](interface.md), and [fixture-catalog.md](fixture-catalog.md).

C8 validation passes TypeScript checking, production Vite compilation, formatting, eight worker lifecycle tests, and 33 Chrome browser tests. Browser coverage includes landing navigation/history, checkpoint selection, real WASM verification, tampering, replay, captures, offline zero-HTTP behavior, and automated accessibility checks. Desktop/mobile landing and product screenshots were visually inspected.

C11 validation passes TypeScript, formatting, production compilation, contract checks, eight worker tests and all 33 Chrome browser tests (landing tests rewritten for the new page; inspector, capture and replay tests unchanged). Native code remains unchanged.

C9 reference/palette validation passed TypeScript, formatting, production Vite compilation, eight worker tests and all 33 Chrome browser regressions. Automated accessibility, offline zero-HTTP behaviour, explicit trust, captures, tampering, and replay pass with the new palette. Desktop/mobile landing, verification, capture, and comparison screenshots were inspected. Exact navy/cream and burgundy/cream contrast ratios are 10.30:1 and 5.42:1. Native code remains unchanged.

The full C7 gate previously passed 74 native Rust tests and the native/browser comparison corpora described in [testing.md](testing.md). Native code, contracts, fixtures, and cryptographic inputs are unchanged in C8; those native suites were not rerun for this presentation checkpoint.

## Boundaries and remaining work

Historical checkpoint authentication/qualification and context-backed conversion of raw RPC captures remain unresolved. Base3/fallback, full Agave differential execution, ancestry/execution proofs, authenticated epoch transitions, node footer export, and production hosting remain outside scope. One equal-length mutation per experiment is supported. Firefox/Safari, real mobile devices, and manual assistive-technology use remain untested.

C9 is one reviewable checkpoint. The detailed handoff and validation evidence remain local under ignored `commits/`. No assistant staging, commit, push, merge, publication, or deployment occurred. Further implementation waits for owner review and commit confirmation.
