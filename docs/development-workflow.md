# Development workflow

## Ownership and commit boundary

Mansi owns every Git commit and push. The assistant prepares changes and verification evidence, then stops at the checkpoint. The assistant does not commit, amend, push, or otherwise create/rewrite commits and does not stage files unless explicitly requested.

The local `commits/` folder contains the detailed handoff narrative and is excluded through `.gitignore` and `.git/info/exclude`. A tracked template lives in [commit-handoff-template.md](commit-handoff-template.md); the completed private reports do not belong in tracked documentation.

## A complete work cycle

1. Read project status, the next checkpoint, repository instructions, and relevant design notes.
2. Inspect Git status and preserve unrelated work.
3. Implement only the current coherent checkpoint.
4. Run checks that substantiate the actual changes and record any missing validation.
5. Update public documents to reflect actual behavior and limitations.
6. Write the numbered local handoff, ending with a suggested one-line commit message.
7. Tell Mansi which handoff to review and that the checkpoint is ready for a commit.
8. Stop implementation. Answer review questions or revise this checkpoint if requested.
9. Resume remaining tasks only when Mansi explicitly confirms the commit is made. Inspect status and the latest commit before starting again.

A status question, permission to fix the current draft, or elapsed time does not count as confirmation of a commit. If confirmation is ambiguous, clarify rather than silently proceeding.

## Suggested owner review for the foundation

Review `README.md`, `AGENTS.md`, and `docs/`, then read the local handoff. Inspect the actual diff or new files. Use an explicit staging list so the intended scope is clear; for the foundation, `.gitignore`, `AGENTS.md`, `README.md`, and `docs/` are the planned public files.

Before committing, check that `commits/` is absent from the staged file list. Git ignore rules protect normal staging, but an explicit force-add can bypass them. Do not force-add the private reports.

The assistant supplies a suggested message, not an executed commit command. After committing, tell the assistant that the commit is made so the next checkpoint can start.

## Local environment observation

At the foundation inspection on 2026-10-09:

| Tool | Observed state |
| --- | --- |
| Rust | `rustc 1.91.1` |
| Cargo | `cargo 1.91.1` |
| Rust target installed | `aarch64-apple-darwin` |
| Node.js | `v24.10.0` |
| npm | `11.6.1` |
| pnpm | Command available; version not checked |
| wasm-pack | Not found on the inspected PATH |
| Browser WASM target | Not present in the installed Rust target list |

These are observations, not the selected or minimum supported toolchain. No dependency installation or toolchain change was performed in C0. C1 pins the existing Rust 1.91.1 toolchain, adds its `wasm32-unknown-unknown` standard library and `llvm-tools` component, and resolves dependencies in `Cargo.lock`. No global default toolchain switch or npm installation was needed. Run instructions are in [compatibility.md](compatibility.md).

## Public documentation versus private handoffs

Public docs explain the project, design, setup, test commands, assumptions, and current feature support. Private reports explain the particular change set in detail: investigation, obstacles, solutions, validation, unresolved work, and review guidance. Do not make public docs depend on private reports to understand the project.

## Handling discoveries and scope changes

If source research invalidates a design assumption, update the relevant decision and plan with evidence. Do not force code to match an obsolete diagram. If a change would remove the Rust/WASM architecture or materially change the trust model, document the alternatives and resolve the product choice before implementing the substitution.

Do not publish, deploy, or change repository settings merely because a checkpoint is complete. Those are separate actions from preparing the owner's next commit.

## C2 boundary tooling

C2 adds the `monocle-core` workspace crate and a private npm package used only to pin TypeScript 5.9.3 for contract checks. Use `npm ci --ignore-scripts --no-audit --no-fund`, then `bash scripts/check-contracts.sh`. Compiled TypeScript, shape-check fixtures, and WASM remain under ignored `target/`; `node_modules/` is ignored. No global npm installation or product UI framework is required.

## C3 verification tooling

Use `bash scripts/check-certificates.sh` for the C3 gate. It includes earlier checkpoint checks, the signed fixture regeneration comparison, and the C3 native/browser comparison. `cargo run --locked --quiet -p monocle-certificate-compat` prints the fixed corpus reports. Arbitrary-input native consumers can call `monocle_core::certificate::verify_certificates`; the C5 inspector now adds browser input; a general-purpose CLI remains unimplemented. Fixture generation is a separate development tool and has no dependency on the verification core.

## C4 evidence tooling

Use `bash scripts/check-finality.sh` for the C4 gate. `cargo run --locked --quiet -p monocle-finality-compat` prints the fixed end-to-end corpus reports. The core entry point is `monocle_core::finality::verify_evidence`; C5 wraps this entry point with browser import and worker control. `generate_finality` is a second binary in the existing fixture package; its default run remains the original C3 generator for compatibility.

## C5 product tooling

Use `npm run dev` to build WASM and launch the local inspector. Use `bash scripts/check-web.sh` for the complete implemented gate. Node.js 24.10 or later is required by the current local test tooling; React, Vite, browser tests, formatting, and accessibility tools are pinned in the npm lockfile. Generated WASM/example inputs, production output, dependencies, screenshots, and test traces are ignored. See [browser-inspector.md](browser-inspector.md). No deploy or publication is part of this checkpoint.

## C6 experiment/replay tooling

The full gate remains `bash scripts/check-web.sh`; no dependency or global-tool change is needed for C6. `crates/monocle-wasm/examples/replay_probe.rs` supplies native operation results to the actual WASM parity check. Production offline testing uses `npm run build:web` and `npm run preview`; wait for **Ready for offline verification** before disconnecting, and keep the page open. Development-server operation is not covered by that offline promise.

The byte ABI is now version 2, with operations 7 (mutation) and 8 (replay validation). Rebuild the web assets after Rust changes; old JavaScript/WASM ABI combinations fail explicitly. Bundle/checkpoint/core request caps remain unchanged even though the outer replay/bridge cap is 16 MiB. C6 changes remain one owner-controlled checkpoint; real capture work must wait for the owner's commit confirmation.
