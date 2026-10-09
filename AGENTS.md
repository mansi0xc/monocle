# Monocle repository instructions

## Owner-controlled commits

These rules are explicit instructions from Mansi and apply to every assistant working in this repository.

- Never create a Git commit, amend a commit, push, or perform another Git operation that creates or rewrites commits. Mansi performs all commits and pushes.
- Do not stage changes unless Mansi specifically requests staging. Leave each checkpoint reviewable in the working tree.
- Work on one agreed checkpoint from `docs/plan.md` at a time. Do not combine later implementation checkpoints merely to keep working.
- At a checkpoint, create a detailed, numbered Markdown handoff in the local `commits/` directory. Use `docs/commit-handoff-template.md` as the structure.
- Cover achievements, changed files, research and sources, problems and solutions, validation results, unresolved issues, cryptographic concepts, workflow progress, and what Mansi should review. Clearly distinguish implemented, proposed, and unverified behavior.
- The final line of each handoff must be a suggested one-line commit message.
- Tell Mansi the checkpoint is ready and link the handoff. Then stop. Do not start remaining tasks until Mansi explicitly confirms the commit is made.
- Once confirmed, inspect the repository status and latest commit before resuming. Never discard unrelated changes.
- `commits/` is private local working material. It is ignored by both `.gitignore` and this checkout's `.git/info/exclude`. Never stage, publish, or copy its reports into tracked documentation.
- Requests to explain or revise a pending checkpoint do not automatically authorize the next checkpoint.

## Project context

Read `docs/status.md`, `docs/idea.md`, `docs/plan.md`, and the relevant architecture/cryptography documents before implementation. Keep public documentation aligned with implemented behavior. The C1 experiment proves fixed-fixture native/browser BLS execution; it is not a complete certificate or finality verifier. Consult current status before making claims.

Monocle is a finality debugger with a fixed trusted validator checkpoint for the MVP. It is not yet an authenticated light client. Synthetic fixtures, raw RPC captures, validated certificates, and block-finality conclusions are distinct objects and must be presented distinctly.

## Engineering constraints

- Prefer existing, reviewed cryptographic primitives and protocol-compatible upstream code. Do not invent a signature scheme or claim Agave compatibility without evidence.
- Pin the protocol source revision and dependency versions when selecting an implementation profile; validate native and WASM compatibility early.
- Never substitute mocked signature validation for real cryptographic verification in the product's verified path.
- Trust must be supplied separately from an imported evidence bundle. An imported `trusted: true` or `pop_verified: true` flag grants nothing.
- Preserve exact signed bytes, signer rank order, integer stake arithmetic, and distinctions between certificate types.
- Missing context or unsupported formats produce an inconclusive result, not a fabricated success or a claim that a block is non-final.
- Test behavior relevant to each change. Record commands and results honestly in the local handoff.
- Use restrained interface design. The architecture documentation uses black and white; prefer a similarly clear, minimal product presentation.

## CodeGraph

If the repository contains a `.codegraph/` directory, use CodeGraph before text search or file reads for code discovery. Otherwise skip it; do not index the repository without the owner's decision.
