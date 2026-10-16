# Monocle interface

C11 implements the Dusk direction chosen by the owner on 2026-10-16 from the C10 design study (`apps/design-study/`). Product context is in [PRODUCT.md](PRODUCT.md); navigation and scope are documented in [docs/interface.md](docs/interface.md).

## Direction

Dusk takes its light from Alpenglow itself: an indigo night canvas, snow-coloured text, and a rose-to-amber glow used only for light sources, primary actions, and selected emphasis. It replaces the C9 cream/navy/burgundy product palette. The owner-supplied feather-and-lens mark is unchanged in shape; in the product it is drawn with a rose feather and a snow lens so it reads on the dark canvas. The exported brand files in `apps/web/public/brand/` and the favicon keep the C9 reference colours on cream. The architecture PDF remains black and white.

The landing hero is data, not decoration. Its ridge is the fast-finality fixture's cumulative signer stake (ranks 0, 1 and 3 reaching 40, 65 and 80 of 100), lit where it meets the 80% fast-path line, with the 60% slow-path line behind it. A caption labels it as synthetic fixture data.

## Typography and assets

Schibsted Grotesk variable (`400–900`) carries the wordmark, headings, and interface text. Red Hat Mono variable (`300–700`) carries digests, byte views, diagnostic codes, and exact integers. Their Latin-subset WOFF2 files total about 69 kB and are local Vite assets. Both SIL Open Font Licenses and source URLs live in `apps/web/src/fonts/`. No external image or font service is contacted at runtime.

Headings use sentence case. Status is always a word plus a symbol (`✓` verified/pass, `×` rejected/fail, `—` inconclusive/not run), never colour alone.

## Tokens

`apps/web/src/styles.css` owns the font faces and tokens:

| Token | Value | Use |
| --- | --- | --- |
| `--night` | `#12172B` | Canvas |
| `--night-2`, `--panel`, `--panel-2` | `#181F3A`, `#1D2546`, `#232C52` | Rails, cards, selected surfaces |
| `--snow` | `#EEF1F8` | Primary text (15.7:1 on canvas) |
| `--mist`, `--dim` | `#A7AFCC`, `#8C95B9` | Secondary and tertiary text (8.2:1 and 6.0:1 on canvas) |
| `--glacier` | `#9ED6E0` | Verified / pass |
| `--rose` | `#F29484` | Rejected / fail |
| `--amber` | `#F6C177` | Focus, selection, trust highlight |
| `--glow` | amber → rose → orchid | Primary actions and stake bars |

Text on the glow uses `--on-glow` (`#1A1430`; 10.8:1 on amber, 7.9:1 on rose, 5.8:1 at the orchid end). Every text pair passes WCAG AA in the automated scans.

## Inspector layout

The inspector is a three-column workspace below a sticky app bar:

- **App bar:** brand, navigation, the loaded evidence name with its provenance label, the original/experiment report switch (after a comparison), and verifier readiness.
- **Left rail (the only `complementary` landmark):** numbered steps for evidence, checkpoint trust, and verification, followed by recorded RPC evidence and portable replay. The steps replace the C8 page heading, workflow rail, and step headers, which told the same story three times. A selected checkpoint is an amber-outlined card that states its basis and that Monocle does not authenticate it.
- **Centre (`Evidence report`):** status line, then the verdict first. That means the outcome badge, title, scope sentence, and a stake meter per certificate. A meter compares stake with its threshold only when the quorum check ran; otherwise it is dimmed and says quorum was not evaluated. Below come the facts, the trace as a chain of status nodes with codes visible inline, certificates, signers, signed bytes, and trust and scope.
- **Right column (`Experiments`):** tamper lab and the fresh comparison, where each changed check shows original and experiment status side by side.

At 1181px and wider, the rail and experiment column stay in view and scroll independently. Between 761 and 1180px the experiment column moves below. At 760px and narrower everything stacks, and once a report or capture exists the result moves above the controls.

## Routing and state

`Root.tsx` routes with URL hashes, compatible with static hosting. `/` and `#/` show the landing page, `#/inspect` shows the inspector, and `#inspector` is the inspector's skip target. `#outcomes` is a landing anchor. The inspector loads lazily and stays mounted, hidden from layout and accessibility while on the landing page. Nothing is persisted across refreshes.

## Motion

The landing summit glow fades in once. Loading placeholders shimmer while the worker runs. All animation and transitions are removed under `prefers-reduced-motion`.

## Validation boundaries

Chrome coverage includes landing at desktop, 375px and 768px, navigation/history, explicit checkpoint trust, hostile input, replay, captures, reduced motion, and automated WCAG A/AA scans of empty, verified, narrow, capture and comparison states. Screenshots at 1440, 1024 and 390px were inspected. Safari, Firefox, real devices and manual assistive-technology use remain untested.
