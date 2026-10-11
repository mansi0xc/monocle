# Monocle

register: product

## Purpose and audience

Monocle is a local, explainable debugger for Solana Alpenglow finality evidence. Developers, security researchers, and students import evidence, explicitly select a validator checkpoint, and inspect the exact checks behind a scoped conclusion. The MVP uses a fixed, externally trusted checkpoint; it is not an authenticated light client.

## Product boundaries

Evidence provenance, structural validation, checkpoint trust, certificate validity, and finality composition are separate. Synthetic examples use public test keys and invented context. A successful example never establishes a network claim. Missing context and unsupported formats remain inconclusive. A rejected input does not establish that a block is non-final.

## Voice

Precise, calm, and direct. Explain trust assumptions beside the result. Use readable labels with exact diagnostic codes available for debugging. Avoid promotional language, invented confidence scores, and unexplained cryptography terminology.

## Visual direction

The owner requested black and white. A developer reads a certificate and compares signer weights during a daytime workshop on a laptop: a white work surface with dark text keeps dense evidence legible beside the reference material. Use a restrained monochrome palette, system typography, thin rules, and a clear distinction between input controls and the evidence report. No gradients, colored status badges, decorative dashboards, or repeated metric cards.

## Interaction and accessibility

Inputs and trust selection are deliberate actions. Display the original-byte digest and session trust basis. Verification runs in an interruptible worker; changes invalidate old results. Support keyboard use, visible focus, explicit loading/error/empty states, reduced motion, readable contrast, and narrow screens. Status must be expressed in text, independently of color. Untrusted strings are rendered as text. Preserve large integers as decimal strings.

## Delivery

C5 provides the browser inspector backed by the existing Rust verifier. Tamper controls and portable replay belong to C6; real capture integration belongs to C7. Follow the owner-controlled commit checkpoints in AGENTS.md.
