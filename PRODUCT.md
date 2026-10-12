# Monocle

register: product

## Purpose and audience

Monocle is a local, explainable debugger for Solana Alpenglow finality evidence. Developers, security researchers, and students import evidence, explicitly select a validator checkpoint, and inspect the exact checks behind a scoped conclusion. The MVP uses a fixed, externally trusted checkpoint; it is not an authenticated light client.

## Product boundaries

Evidence provenance, structural validation, checkpoint trust, certificate validity, and finality composition are separate. Synthetic examples use public test keys and invented context. A successful example never establishes a network claim. Missing context and unsupported formats remain inconclusive. A rejected input does not establish that a block is non-final.

## Voice

Precise, calm, and direct. Explain trust assumptions beside the result. Use readable labels with exact diagnostic codes available for debugging. Avoid invented confidence scores, unverifiable network claims, and unexplained cryptography terminology. Landing copy introduces the workflow; the product reports exact evidence and limitations.

## Visual direction

The owner's 2026-10-13 reference supersedes the earlier ocean-blue/persimmon palette and exploratory logos. The architecture PDF remains unchanged. Use the supplied feather-and-lens symbol, with the reference's surrounding badge and “Feathered Lens” text removed; the product name remains Monocle.

The selected colours are burgundy `#AA2D3C`, deep navy `#28324F`, and cream `#F2E6D8`. Cream carries the main canvas, navy carries readable text and structural areas, and burgundy carries primary actions, selected emphasis, and the feather. The lens and wordmark are navy. The landing has a navy optical scene and a burgundy experiment section; product report surfaces remain restrained and readable.

Sora carries the wordmark/headings; Public Sans carries interface text. Both stay bundled locally. All input, trust, result, and offline boundaries remain unchanged. Status words and check symbols retain meaning independently of their colour.

## Interaction and accessibility

The landing page at `/` or `#/` leads to the inspector at `#/inspect`. Its Verify/Tamper/Replay preview is explicitly illustrative and never runs or grants trust. The real inspector is loaded on first entry and stays mounted when visiting the landing page within the same session, preserving inputs and explicit decisions. Refreshing still starts a new in-memory session.

Inputs and trust selection are deliberate actions. Display the original-byte digest and session trust basis. Verification runs in an interruptible worker; changes invalidate old results. Support keyboard use, visible focus, explicit loading/error/empty states, reduced motion, readable contrast, and narrow screens. Untrusted strings are rendered as text. Preserve large integers as decimal strings.

## Delivery

C8 delivers the landing page, shared identity, responsive product redesign, and in-session navigation on a separate `codex/monocle-ui` branch. C5–C7 verification, tampering, replay, and recorded-capture boundaries remain unchanged. Offline use is supported only in an already-loaded production inspector session. Captures cannot inherit verified results or checkpoint trust. The Colosseum feature exploration is paused. Follow the owner-controlled commit checkpoints in AGENTS.md.
