# Monocle interface

Implemented C5/C6/C7 product surface: `apps/web/src/styles.css`. Product direction is in [PRODUCT.md](PRODUCT.md).

## Visual system

Black and white with achromatic OKLCH values: ink L19%, muted text L44%, rules L86%, input surface L97%, white L100%. Form borders use L65%; focus outlines use ink. No color carries status meaning. A single system sans-serif family handles interface text, with system monospace for byte strings and exact numeric values. No external font or image requests.

A quiet wordmark/header leads into the inspection workspace. The left column holds the actual three-step workflow; the right column holds the report. Thin rules group sections without nested cards. The input column is 320px (340px on wide screens, 290px at intermediate widths), with a flexible report. Below 740px they stack. Body measures stay within 72 characters where practical; long hashes wrap within their containers.

Main heading 1.75rem, result heading 2rem, section heading 1.125rem, interface body .875–1rem, dense metadata .8125rem. Small status codes supplement readable text. Buttons/fields use 3–4px radii. Check/cross/dash symbols and explicit PASS/FAIL/NOT CHECKED text expose check state.

## Interaction

Changing evidence, the active experiment, or checkpoint context clears the previous results. Editing an unsubmitted experiment form does not alter the active evidence; export preserves current reports. Loading never implies verification or checkpoint authenticity. Native selects, file inputs, checkboxes, details, and table semantics provide familiar keyboard behavior. Focus outlines are 2px with an offset; a skip link and live status region support navigation. The spinner indicates active work and becomes static with reduced motion. Hover transitions are limited to 160ms; there are no decorative entrance animations.

Errors include a recoverable next action. The result exposes scope before the detailed trace, then certificates, exact weights, signed bytes, and assumptions. Signer tables paginate at 50 entries and retain an internal horizontal scroll region at narrow widths.

## Validation and boundaries

Chrome screenshots were inspected at 1440px and 375px. Automated accessibility checks exercise empty, verified, and narrow layouts. These checks do not establish conformance across all assistive technologies or browsers. C6 adds a native disclosure for mutation controls, a before/after trace table, original/experiment report selection, and local replay controls. The table shows changed checks by default and supports inspecting all checks; on narrow screens it scrolls inside its own labeled region. Profile/checkpoint requirements and the difference between loaded assets and offline cold start remain explicit.

## Recorded evidence

C7 keeps recorded RPC inputs in a separate labeled section beside synthetic examples. Capture output leads with inconclusive scope and extracted identity, followed by missing prerequisites, an explicit unperformed-check trace, preserved bytes, and exact download. It uses the existing monochrome typography, borders, native controls, and responsive facts layout. No network status or trust is inferred from the sample's filename.
