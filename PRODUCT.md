# Monocle

register: product

## Purpose and audience

Monocle is a local, explainable debugger for Solana Alpenglow finality evidence. Developers, security researchers, and students import evidence, explicitly select a validator checkpoint, and inspect the exact checks behind a scoped conclusion. The MVP uses a fixed, externally trusted checkpoint; it is not an authenticated light client.

## Product boundaries

Evidence provenance, structural validation, checkpoint trust, certificate validity, and finality composition are separate. Synthetic examples use public test keys and invented context. A successful example never establishes a network claim. Missing context and unsupported formats remain inconclusive. A rejected input does not establish that a block is non-final.

## Voice

Precise, calm, and direct. Explain trust assumptions beside the result. Use readable labels with exact diagnostic codes available for debugging. Avoid invented confidence scores, unverifiable network claims, and unexplained cryptography terminology. Landing copy introduces the workflow; the product reports exact evidence and limitations.

## Visual direction

On 2026-10-12, the owner requested a distinctive landing page, a redesign of the whole product, and complementary colors. This supersedes the earlier monochrome product preference. The architecture PDF remains unchanged.

A developer studies dense verification evidence during a daytime workshop, often beside source code: use bright, cool work surfaces, dark ocean-blue text, and restrained persimmon accents. The brand evokes an optical instrument: precise, lucid, and inquisitive. A custom geometric lens gives the landing page its visual identity; the product uses that vocabulary sparingly so controls and evidence remain primary.

Sora carries the brand and selected headings; Public Sans carries readable interface text. Both are bundled locally with their licenses. Typography, purposeful whitespace, a complementary blue/orange palette, and clear state labels establish hierarchy. Color never carries a conclusion by itself.

## Interaction and accessibility

The landing page at `/` or `#/` leads to the inspector at `#/inspect`. Its Verify/Tamper/Replay preview is explicitly illustrative and never runs or grants trust. The real inspector is loaded on first entry and stays mounted when visiting the landing page within the same session, preserving inputs and explicit decisions. Refreshing still starts a new in-memory session.

Inputs and trust selection are deliberate actions. Display the original-byte digest and session trust basis. Verification runs in an interruptible worker; changes invalidate old results. Support keyboard use, visible focus, explicit loading/error/empty states, reduced motion, readable contrast, and narrow screens. Untrusted strings are rendered as text. Preserve large integers as decimal strings.

## Delivery

C8 delivers the landing page, shared identity, responsive product redesign, and in-session navigation on a separate `codex/monocle-ui` branch. C5–C7 verification, tampering, replay, and recorded-capture boundaries remain unchanged. Offline use is supported only in an already-loaded production inspector session. Captures cannot inherit verified results or checkpoint trust. The Colosseum feature exploration is paused. Follow the owner-controlled commit checkpoints in AGENTS.md.
