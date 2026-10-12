# Monocle interface

C8 design system and implementation. Product context is in [PRODUCT.md](PRODUCT.md); navigation and scope are documented in [docs/interface.md](docs/interface.md).

## Direction

Precision optics, ocean blue, persimmon, and cool white. The landing page uses a larger optical scene and a warm experiment section; the working inspector keeps saturation restrained. This explicitly follows the owner's 2026-10-12 request for complementary colors, superseding the earlier monochrome product brief. The architecture PDF remains black and white.

## Typography and assets

Sora variable (`100–800`) is the wordmark/display family; Public Sans variable (`100–900`) is the body/control family. Their Latin-subset WOFF2 files total about 60 kB and are local Vite assets. Both SIL Open Font Licenses and source URLs live in `apps/web/src/fonts/`. No external image/font services are contacted at runtime. Missing glyphs use system fallbacks; bytes and exact amounts use system monospace.

Landing headings use fluid sizing capped below 6rem, balanced wrapping, and letter spacing no tighter than -0.04em. The inspector uses a restrained rem scale and tabular numeric rendering. Readable explanation text and exact diagnostic metadata remain distinct.

## Tokens and components

`apps/web/src/styles.css` owns shared font faces and tokens. Principal OKLCH values: ink `26% .038 224`, brand `32% .059 224`, accent `74% .154 43`, muted `46% .028 224`, canvas `98% .004 210`. Semantic success, warning, and rejection colors supplement visible text and symbols. Focus uses a visible outline.

The product has an overview/inspector navigation shell, an ordered workflow rail, a three-step evidence/checkpoint/verify column, a report surface, and a separate recorded-capture/replay area. Traces, signer tables, native disclosures, comparison reports, source labels, file inputs, status banners, loading placeholders, and errors share consistent visual treatment. No automatic trust selection or crypto logic is added by this design system.

`landing.css` is scoped to the landing surface. It defines the split hero, geometric optical illustration, keyboard-operable workflow preview, byte-change illustration, trust explanation, and closing action. Its preview is declared illustrative and synthetic; it is not a substitute for the working inspector.

## Routing and state

`Root.tsx` routes with URL hashes, compatible with static hosting. `/` and `#/` show the landing page, `#/inspect` shows the inspector, and the inspector's `#inspector` skip target remains an inspector route. `#workflow` is a landing anchor. The inspector loads lazily and stays mounted, hidden from layout/accessibility while on the landing page. Returning restores the same evidence, trust decision, and reports. Nothing is persisted across refreshes.

## Responsive and motion behavior

The hero and product columns stack on narrow screens. Long digests wrap, wide data tables scroll within labeled regions, and signer pagination stays available. Native controls retain keyboard behavior. Skip links are revealed on focus. All animations/transitions have reduced-motion alternatives. The landing lens settles once on entry; workflow preview transitions are brief. Product motion conveys loading/state rather than introducing a page entrance sequence.

## Validation boundaries

Chrome product coverage includes desktop, 375px and 768px landing layouts, narrow verified/capture/comparison states, navigation/history, explicit checkpoint trust, hostile input, replay, reduced motion, and automated accessibility scans. Screenshots are visually inspected in addition to automated checks. This is not a claim of complete assistive-technology or cross-browser conformance; Safari/Firefox and real devices remain untested.
