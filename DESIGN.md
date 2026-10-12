# Monocle interface

C8 design system and C9 logo refinement. Product context is in [PRODUCT.md](PRODUCT.md); navigation and scope are documented in [docs/interface.md](docs/interface.md).

## Direction

The reference supplied by the owner on 2026-10-13 defines this revision: a feather curling around a monocle lens, and burgundy `#AA2D3C`, navy `#28324F`, cream `#F2E6D8`. It replaces the previous palette and rejected exploratory marks. The architecture PDF remains black and white.

Cream is the page canvas; navy carries text, the lens, and dark structural sections; burgundy carries primary actions, the feather, and deliberate landing highlights. Existing product layout and interactions are preserved. The shared mark is derived from the owner-supplied reference and retains its short handle and inner reflection arc. The surrounding reference text/badge is excluded.

## Typography and assets

Sora variable (`100–800`) is the wordmark/display family; Public Sans variable (`100–900`) is the body/control family. Their Latin-subset WOFF2 files total about 60 kB and are local Vite assets. Both SIL Open Font Licenses and source URLs live in `apps/web/src/fonts/`. No external image/font services are contacted at runtime. Missing glyphs use system fallbacks; bytes and exact amounts use system monospace.

Landing headings use fluid sizing capped below 6rem, balanced wrapping, and letter spacing no tighter than -0.04em. The inspector uses a restrained rem scale and tabular numeric rendering. Readable explanation text and exact diagnostic metadata remain distinct.

## Tokens and components

`apps/web/src/styles.css` owns shared font faces and tokens. The three supplied sRGB colours are encoded as precise OKLCH primitive tokens (`--brand-navy`, `--brand-burgundy`, `--brand-cream`). Surface, muted, border, hover and report tokens derive from these colours with OKLab mixing. Navy on cream measures 10.30:1 and burgundy on cream 5.42:1. White is used only to lift local product surfaces. Verified checks use navy with an explicit checkmark; failed checks use burgundy with an explicit failure symbol; missing context retains its label and a cream surface. Focus uses a visible palette outline.

The product has an overview/inspector navigation shell, an ordered workflow rail, a three-step evidence/checkpoint/verify column, a report surface, and a separate recorded-capture/replay area. Traces, signer tables, native disclosures, comparison reports, source labels, file inputs, status banners, loading placeholders, and errors share consistent visual treatment. No automatic trust selection or crypto logic is added by this design system.

`landing.css` is scoped to the landing surface. It defines the split hero, geometric optical illustration, keyboard-operable workflow preview, byte-change illustration, trust explanation, and closing action. Its preview is declared illustrative and synthetic; it is not a substitute for the working inspector.

## Routing and state

`Root.tsx` routes with URL hashes, compatible with static hosting. `/` and `#/` show the landing page, `#/inspect` shows the inspector, and the inspector's `#inspector` skip target remains an inspector route. `#workflow` is a landing anchor. The inspector loads lazily and stays mounted, hidden from layout/accessibility while on the landing page. Returning restores the same evidence, trust decision, and reports. Nothing is persisted across refreshes.

## Responsive and motion behavior

The hero and product columns stack on narrow screens. Long digests wrap, wide data tables scroll within labeled regions, and signer pagination stays available. Native controls retain keyboard behavior. Skip links are revealed on focus. All animations/transitions have reduced-motion alternatives. The landing lens settles once on entry; workflow preview transitions are brief. Product motion conveys loading/state rather than introducing a page entrance sequence.

## Validation boundaries

Chrome product coverage includes desktop, 375px and 768px landing layouts, narrow verified/capture/comparison states, navigation/history, explicit checkpoint trust, hostile input, replay, reduced motion, and automated accessibility scans. Screenshots are visually inspected in addition to automated checks. This is not a claim of complete assistive-technology or cross-browser conformance; Safari/Firefox and real devices remain untested.

## Reference feather and lens mark

C9 follows the supplied feather/lens reference rather than drawing another original mark. The cleaned/recoloured raster was converted into native SVG outline paths, so the live mark uses exact CSS palette colours and requires no image request. Feather is burgundy; rim, short handle and inner reflection are navy. Transparent counters show the cream page. `Brand.tsx` is decorative (`aria-hidden`, `focusable=false`) beside the existing accessible brand/link text.

The favicon uses the same outlines on cream. Transparent PNG, standalone native SVG, outlined horizontal wordmark, and cream inverse SVG exports live in `apps/web/public/brand/`. The existing Sora wordmark is retained; its SIL Open Font License remains in `apps/web/src/fonts/Sora-OFL.txt`. Asset notes identify the owner-supplied online reference rather than claiming the original symbol was authored by Monocle.

The logo and palette are a revision of the same pending C9 checkpoint. No cryptography, additional remote dependency, or deployment is included. Validate keyboard states, contrast, and complete report/capture/comparison layouts at desktop and narrow widths.
