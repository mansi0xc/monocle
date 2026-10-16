# Monocle interface

C11 implements the Dusk direction, C12 adds its serif display face, and C13 adds explanatory motion and interaction, chosen by the owner on 2026-10-16 from the C10 design study (`apps/design-study/`). Product context is in [PRODUCT.md](PRODUCT.md); navigation and scope are documented in [docs/interface.md](docs/interface.md).

## Direction

Dusk takes its light from Alpenglow itself: an indigo night canvas, snow-coloured text, and a rose-to-amber glow used only for light sources, primary actions, and selected emphasis. It replaces the C9 cream/navy/burgundy product palette. The owner-supplied feather-and-lens mark is unchanged in shape; in the product it is drawn with a rose feather and a snow lens so it reads on the dark canvas. The exported brand files in `apps/web/public/brand/` and the favicon keep the C9 reference colours on cream. The architecture PDF remains black and white.

The landing hero is data, not decoration. Its ridge is the fast-finality fixture's cumulative signer stake (ranks 0, 1 and 3 reaching 40, 65 and 80 of 100), lit where it meets the 80% fast-path line, with the 60% slow-path line behind it. A caption labels it as synthetic fixture data.

## Typography and assets

Three faces, each with one job (C12, after owner feedback that a single grotesk made the site monotonous):

- **Newsreader** (variable, instanced to weights `300–500` and optical sizes `24–72`) is the display face. It sets the landing headline and section titles, capability and ledger statements, outcome names, report verdict titles, report section titles, and the experiments column headings. Large sizes use the light weight.
- **Schibsted Grotesk** (variable `400–900`) carries the wordmark, navigation, controls, rail headings, labels, and running text.
- **Red Hat Mono** (variable `300–700`) carries digests, byte views, diagnostic codes, verdict badges, and exact integers.

All three Latin-subset WOFF2 files (about 155 kB together) are local Vite assets. Their SIL Open Font Licenses and sources live in `apps/web/src/fonts/`. No external image or font service is contacted at runtime.

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

The inspector shows one step at a time (C14). Owner feedback was that showing every operation at once felt messy and cramped.

- **App bar:** brand, navigation, the loaded evidence name with its provenance label, the original/experiment report switch (after a comparison), and verifier readiness.
- **Left rail (the only `complementary` landmark):**
  - **Evidence:** the example picker stays visible for quick switching; importing your own bundle sits in a disclosure.
  - **Checkpoint trust:** importing a checkpoint file sits in a disclosure, opened automatically when imported evidence still needs one.
  - **Recorded evidence and replay:** the recorded testnet response and replay export stay one click away; file imports sit in a disclosure.

  The open step is marked with an amber edge and a filled step number. Finished steps show a check and stay quiet. The trust choice is a framed control group, and a selected checkpoint becomes an amber-outlined card stating its basis and that Monocle does not authenticate it.
- **Centre (`Evidence report`):**
  - A **run bar** holds step 3, Verify, beside the result it produces. Its button only takes the primary glow when verification is the next step. It stays usable earlier, because running without a checkpoint is allowed and ends as inconclusive.
  - Before a result, the centre names the next action: start with an example, then "Choose the checkpoint you trust" with a "Go to checkpoint trust" button, then "Ready to verify".
  - After a run, the verdict comes first, then the facts, the trace, the certificates, and the trust and scope.
- **Right column (`Experiments`):** appears only once a checkpoint is selected or a replay brings an experiment. Before that the workspace has two columns.

At 1181px and wider, the rail and experiment column stay in view and scroll independently. Between 761 and 1180px the experiment column moves below. At 760px and narrower everything stacks, and once a report or capture exists the result moves above the controls.

## Routing and state

`Root.tsx` routes with URL hashes, compatible with static hosting. `/` and `#/` show the landing page, `#/inspect` shows the inspector, and `#inspector` is the inspector's skip target. `#outcomes` is a landing anchor. The inspector loads lazily and stays mounted, hidden from layout and accessibility while on the landing page. Nothing is persisted across refreshes.

## Motion and interaction

C12 added the display face; C13 adds motion that explains something, not decoration:

- **Landing entrance.** The one orchestrated moment: the ridge draws itself signer by signer (40 → 65 → 80), each signer's point appears in turn, and the summit lights when the climb meets the 80% line.
- **Ridge scenarios.** A four-button switch (fast path, too little stake, slow path, genesis anchor) morphs the ridge between recorded fixture results. The relevant threshold line highlights, and the summit chip shows the real result code. "Too little stake" visibly stops below its line and the glow dims. The caption updates in a polite live region and always says "Synthetic keys, invented stake."
- **Break it on purpose.** The fast certificate's real bytes 34–55. Byte 42 is a toggle button; flipping it from 01 to 00 slides the new value in, and the recorded trace cascades: signature fails, quorum and finality become "not run", and the verdict reads REJECTED with the non-finality caveat. The section states that it replays a recorded result and verifies nothing.
- **Inspector.** After each run the trace reveals in check order, stake bars fill from zero to their exact share, the verdict title wipes in, a newly selected checkpoint card settles in, and each changed comparison row gets a fading amber edge.

Reveals clip or move text rather than fading it, so text keeps full contrast at every frame and the automated scans pass mid-animation. Ridge morphing uses CSS `d` transitions (Chromium/Firefox; Safari snaps to the new shape). All motion is removed under `prefers-reduced-motion`.

## Validation boundaries

Chrome coverage includes landing at desktop, 375px and 768px, navigation/history, explicit checkpoint trust, hostile input, replay, captures, reduced motion, and automated WCAG A/AA scans of empty, verified, narrow, capture and comparison states. Screenshots at 1440, 1024 and 390px were inspected. Safari, Firefox, real devices and manual assistive-technology use remain untested.
