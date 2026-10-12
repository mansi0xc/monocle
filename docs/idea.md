# Monocle: project idea

Status: MVP and C8 interface workflows committed. C9 adopts the supplied feather/lens reference and burgundy/navy/cream palette, awaiting owner review/commit. Real-network verification remains unestablished.

## Description

Monocle is an open-source finality debugger for Solana's Alpenglow consensus, designed to make cryptographic verification understandable and reproducible. It checks certificate signatures and validator stake thresholds against a trusted checkpoint, explaining why verification succeeds, fails, or remains inconclusive. Its tamper lab and portable evidence bundles let developers experiment with altered certificates, compare results, and replay verification offline.

These sentences describe the intended product. The repository now contains native/browser BLS compatibility checks plus bounded evidence contracts and explicit local checkpoint selection; C3/C4 now add individual certificate checks and direct evidence composition. C5 adds the working browser inspector, file import, explicit checkpoint selection, and worker-backed reports. C6 implements the tamper application and portable replay, limited to one equal-length byte replacement from an unmodified baseline. Both comparison reports are freshly verified; imported reports are ignored. C7 adds separately labeled recorded RPC inspection and the recorded evidence workflow. A capture missing historical context remains inconclusive.

## The problem

A developer inspecting consensus evidence needs more than a binary success flag. An aggregate signature can be incorrectly serialized, paired with the wrong signer map, interpreted under the wrong protocol version, or associated with incomplete finality evidence. Several of these failures look similar from outside the verifier. A useful debugging tool should expose which assumption or check failed and retain enough evidence to reproduce the result.

The distinction between data access and verification is central. Receiving an RPC response establishes that an endpoint returned some bytes. A cryptographic conclusion additionally requires the correct message interpretation, qualified public keys, signer weights, and trusted context. Fetching all those values from the same untrusted bundle does not remove that trust problem.

Monocle's contribution is the inspection and experiment workflow around protocol-compatible verification. It is not a claim to have invented BLS verification or to be the first implementation of Alpenglow's verification rules.

## Intended users

| User | Task Monocle should make easier |
| --- | --- |
| Solana infrastructure developer | Explain why a captured certificate cannot be verified under a particular context |
| Security researcher | Reproduce malformed-input, wrong-context, and signer-selection cases |
| Student or protocol learner | Observe the difference between signature validity, sufficient stake, and complete finality evidence |
| Tooling contributor | Share a minimal portable example with its assumptions and expected diagnostics |

The graduation demonstration focuses on developers and learners. It does not offer custody, authorize transfers, or run a bridge.

## Distinctive features

### 1. Explainable checks

Present a structured verification trace: parse, profile, context, signer resolution, key qualification, signature, quorum, and finality evaluation. Each check has a stable code, a status, and a specific explanation. A check that could not run is visibly different from a check that failed.

Example: a report should explain that the selected checkpoint is for a different epoch, or that a valid notarization is missing its finalization counterpart. It should not collapse both situations into an unexplained red badge.

### 2. Controlled tamper lab

Keep the original bundle immutable and create a separate derived copy for each experiment. Record exactly what changed. The C6 lab supports one equal-length byte replacement within a certificate, including a signature, signed field, or bitmap edit. Context selection can also be removed to observe an inconclusive run; changing bundle metadata or stacking mutations is not part of the lab contract.

Changing a bitmap usually also changes the public-key aggregate and breaks the signature check. A demonstration of insufficient stake with a valid signature therefore needs its own correctly signed fixture. The interface must distinguish these experiments.

### 3. Portable, reproducible evidence

Export the source evidence, profile identity, checkpoint reference, mutation history, and diagnostic reports. On import, recompute verification. A saved report is evidence about a previous run; it is not authorization to skip checks.

C6 offline replay means keeping the production page open after its readiness indicator appears, with evidence/context loaded or available as local files. The loaded page retains the inline worker and WASM bytes in memory. A first visit or refresh while disconnected is not supported; the development server is not an offline deployment.

### 4. Explicit trust and evidence scope

Show whether the result concerns ordinary block finality, migration/genesis evidence, or a certificate that is individually valid but insufficient for a block-finality conclusion. Display the selected checkpoint and the origin of the evidence. Synthetic examples remain clearly labeled even when their signatures verify correctly.

## Principal user journey

1. Open Monocle and choose a prepared example or import a local bundle.
2. Inspect its origin, certificate type, slot, and claimed protocol profile.
3. Select a compatible checkpoint from an explicitly trusted local source.
4. Run verification in the browser worker.
5. Inspect check results, participating signer ranks, and exact stake totals.
6. Make a controlled mutation and rerun under the same context.
7. Compare the reports and export the experiment.
8. Re-import it and reproduce the result with the required checkpoint.

The UI must explain missing prerequisites before presenting a finality conclusion. No wallet connection is needed for this workflow.

## MVP scope

- One explicitly pinned implementation profile initially.
- A fixed epoch checkpoint per verification run.
- Genuine BLS verification using established primitives and compatible message encodings.
- A native fixture/reference path and browser verification using the same core.
- Structured diagnostics with three overall conclusions.
- Fast-path evidence, paired slow-path evidence, and separately classified genesis evidence, introduced in tested increments.
- Original/derived bundle comparison and local export/import.
- A deterministic synthetic demonstration that works independently of public endpoint availability.
- Real captured evidence support when its exact format and trusted context are available; otherwise an honest inconclusive result.

## Non-goals for the MVP

- Authenticating arbitrary future validator sets from a genesis trust root.
- Tracking the entire chain or validating every transaction's execution.
- Running an on-chain verifier, bridge, wallet, token, or staking protocol.
- A production live certificate feed or hosted verification service.
- Supporting every historical Agave wire format.
- Treating reward, skip, or unknown certificate types as ordinary block-finality evidence.
- Inferring ancestor finality without the required authenticated ancestry data.

## Demonstration and success criteria

The demonstration should start with a real cryptographic baseline, change one signed input, explain the failing check, then replay the exported experiment. A second example should contrast a valid signature with insufficient stake or incomplete evidence. The presenter should be able to explain every displayed verdict and its trust assumptions.

Success means that another developer can reproduce the fixture results, inspect the actual signed inputs and signer context, and understand why a conclusion follows. It does not require a live-network success badge. A real capture without its historical checkpoint should be shown as incomplete, not hidden or promoted to verified.

## Dependencies and unresolved feasibility

C1 has demonstrated the Solana BLS primitive path in native Rust and browser WASM using fixed synthetic evidence. C3/C4 establish a narrow four-kind Base2 profile and direct composition policy. Full upstream equivalence, broader format support, and authenticated historical validator context remain open. Neither follows merely from the architecture diagram, a successful primitive test, or the RPC request already observed.

See [research.md](research.md), [cryptography.md](cryptography.md), and [plan.md](plan.md) for the evidence and implementation gates.
