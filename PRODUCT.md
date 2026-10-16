# Monocle

register: product

## Purpose and audience

Monocle is a local, explainable debugger for Solana Alpenglow finality evidence. Developers, security researchers, and students import evidence, explicitly select a validator checkpoint, and inspect the exact checks behind a scoped conclusion. The MVP uses a fixed, externally trusted checkpoint; it is not an authenticated light client.

## Product boundaries

Evidence provenance, structural validation, checkpoint trust, certificate validity, and finality composition are separate. Synthetic examples use public test keys and invented context. A successful example never establishes a network claim. Missing context and unsupported formats remain inconclusive. A rejected input does not establish that a block is non-final.

## Voice

Precise, calm, and direct. Explain trust assumptions beside the result. Use readable labels with exact diagnostic codes available for debugging. Avoid invented confidence scores, unverifiable network claims, and unexplained cryptography terminology. Landing copy introduces the workflow; the product reports exact evidence and limitations.

## Visual direction

The owner chose the Dusk direction on 2026-10-16 (C10 design study, implemented in C11). It supersedes the C9 cream/navy/burgundy product palette. Dusk uses an indigo night canvas, snow text, and a rose-to-amber alpenglow light reserved for primary actions, selected emphasis and the landing ridge. The ridge is drawn from the fast fixture's real cumulative signer stake and is labelled as synthetic. The owner-supplied feather-and-lens symbol is kept; in the product the feather is rose and the lens snow. Exported brand files keep the reference colours.

Newsreader, a light serif, carries headlines, section titles and verdicts. Schibsted Grotesk carries interface and body text, and Red Hat Mono carries digests, codes and exact integers. All three are bundled locally. All input, trust, result, and offline boundaries remain unchanged. Status words and check symbols retain meaning independently of their colour.

## Interaction and accessibility

The landing page at `/` or `#/` leads to the inspector at `#/inspect`. Its outcome list restates real fixture results with their diagnostic codes, labelled synthetic; the landing page never loads evidence, runs the verifier, or grants trust. The real inspector is loaded on first entry and stays mounted when visiting the landing page within the same session, preserving inputs and explicit decisions. Refreshing still starts a new in-memory session.

Inputs and trust selection are deliberate actions. Display the original-byte digest and session trust basis. Verification runs in an interruptible worker; changes invalidate old results. Support keyboard use, visible focus, explicit loading/error/empty states, reduced motion, readable contrast, and narrow screens. Untrusted strings are rendered as text. Preserve large integers as decimal strings.

## Delivery

C8 delivers the landing page, shared identity, responsive product redesign, and in-session navigation on a separate `codex/monocle-ui` branch. C5–C7 verification, tampering, replay, and recorded-capture boundaries remain unchanged. Offline use is supported only in an already-loaded production inspector session. Captures cannot inherit verified results or checkpoint trust. The Colosseum feature exploration is paused. Follow the owner-controlled commit checkpoints in AGENTS.md.
