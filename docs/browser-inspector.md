# Browser inspector (C5, extended in C6/C7)

Monocle now has a React/TypeScript inspector at `apps/web`, backed by the same Rust core used by the native tests. It accepts arbitrary Monocle v1 proof bundles and checkpoint files within the existing limits. All built-in examples are signed synthetic evidence. C5 adds no new cryptographic primitive or consensus rule. C6 adds [tamper experiments and replay](tamper-replay.md); the shared runtime details below reflect that extension.

## Run it

Prerequisites: the pinned Rust toolchain and its WASM/LLVM components, WASM-capable Clang, and Node.js 24.10 or later. The package lock pins React/React DOM 19.3.0, Vite 8.3.4, TypeScript 5.9.3, Playwright 1.64.0, axe Playwright 4.13.0, and Prettier 3.9.10. There is no global frontend tool installation.

```sh
npm ci --ignore-scripts --no-audit --no-fund
npm run dev
```

Open the loopback address printed by Vite. `npm run dev` builds the Rust WASM and prepares the synthetic examples before starting the UI. Changes to Rust require restarting this command; TypeScript/CSS changes are handled by Vite. For a production build:

```sh
npm run build:web
npm run preview
```

Build output is `apps/web/dist/`. Serve it over HTTP(S); opening its HTML with `file://` does not provide a supported worker environment. Assets use relative URLs for static hosting, including a nested base path. Refresh clears session inputs, selected trust, and results. C6 adds explicit file export and offline replay within a loaded production session. It adds no deployment, persistent storage, service worker, or offline first-visit/refresh support.

## First inspection

1. Choose **Fast finality** and **Load**. This structurally parses a signed bundle and its synthetic checkpoint without granting trust.
2. Choose **Use this checkpoint**. The default synthetic basis is restricted to synthetic evidence. This is a local session decision, not authentication of a real validator set.
3. Choose **Verify evidence**. The worker reruns the actual parser, key/PoP checks, aggregate signature verification, stake thresholds, and direct finality composition.
4. Read the established scope and trace. Expand certificate bytes and validator identities; the signer table shows exact checkpoint stake and PoP presence. Presence alone is not validity.

The other examples cover slow finality, genesis-anchor evidence, missing finalization, a changed signature, and a genuinely signed under-quorum certificate. `scripts/prepare-web.mjs` selects original strings from the existing finality corpus and deliberately omits saved expectations/reports from product assets.

## Import and trust

A proof bundle is limited to 1 MiB and a checkpoint to 4 MiB. File size is checked before reading; Rust repeats authoritative bounds, duplicate-key rejection, schema/hex validation, and structural limits. A raw `getAgGenesisCert` response is not the Monocle bundle schema. Use the separate C7 **Recorded RPC evidence** controls for [capture inspection](capture-import.md), bounded to 64 KiB. This mode clears prior verification/trust and remains inconclusive without historical context.

Original byte arrays stay in memory. The worker passes their original UTF-8 text through the request envelope without parsing and reserializing the evidence. Rust computes SHA-256 over the original source. Slots, epochs, and stakes remain decimal strings; the UI makes no threshold decisions with floating-point numbers.

Checkpoint import only creates an inspected candidate. **Use this checkpoint** records a separate synthetic or operator decision. An operator selection requires a nonblank reason of at most 1024 UTF-8 bytes. The reason documents an assumption and does not authenticate the checkpoint. Each verification reparses the selected bytes, pins the chosen basis in a fresh Rust trust store, and compares the resulting digest with the session selection before running the core.

Replacing the checkpoint revokes its selection immediately, including when replacement parsing fails. Replacing evidence retains a deliberately selected checkpoint but invalidates the report; the core checks whether the new evidence matches that context. Loading a built-in example replaces both inputs and requires selecting its checkpoint again. Imported flags, checkpoint references, saved reports, and display strings cannot grant trust or produce a verified result.

## Worker and byte boundary

`crates/monocle-wasm` contains a small ABI v2 around `monocle-core`. A Rust-owned input vector accepts at most 16 MiB for replay packaging. Each operation retains its own stricter core bounds, including `MAX_REQUEST_BYTES` (6 MiB + 4096 bytes) for verification. `monocle_input_alloc`, `monocle_execute`, and output pointer/length exports transfer bounded data. Rust accepts no caller-supplied pointer to dereference. JavaScript reads fresh memory views after execution because memory growth can detach earlier views. The JavaScript host is trusted application code and has access to linear memory; this ABI is not a sandbox against malicious application scripts.

| Operation | Input | Result |
| --- | --- | --- |
| 1 | Original bundle bytes | Parsed document and exact-source digest; no signature verdict |
| 2 | Original checkpoint bytes | Parsed candidate and digest; revokes previous trust |
| 3 | Empty bytes | Explicit synthetic pin of the candidate |
| 4 | UTF-8 local reason | Explicit operator pin of the candidate |
| 5 | C2/C4 request bytes | Fresh C4 evidence report |
| 6 | Empty bytes | Clear candidate and trust |
| 7 | Mutation request with original source | Derived bundle with exact patch metadata |
| 8 | Replay archive bytes | Validated archive; no trust selection or current verdict |
| 9 | Raw genesis JSON-RPC response bytes | Capture inspection and missing prerequisites; never a verification success |
| 10 | UTF-8 local reason | Explicit operator pin that also accepts the candidate's keys as registered on chain (C19) |

These numbers are the Rust `Operation` enum in `crates/monocle-wasm`; the browser uses the generated `Operation` names from the contracts package rather than literals. Unknown numbers fail with `INVALID_SCHEMA` at path `operation`.

Responses are `{ok:true,value:...}` or `{ok:false,error:Diagnostic}`. An execution failure has no usable old output. Request run IDs must match the worker envelope and returned report; C6 comparison reports use distinct baseline/derived IDs checked together. The app never accepts imported reports as results.

In production, the worker is bundled inline and WASM bytes are loaded once into page memory. Fresh workers receive a copy of those bytes; their startup does not require another fetch. Development mode may still request Vite modules. See [offline behavior](tamper-replay.md#offline-behavior).

Every job creates a dedicated worker. Completion, error, cancellation, and the 30-second deadline terminate it and release its verifier state. Starting another job cancels the previous one. The controller also tracks input revisions so a late file read cannot overwrite newer input. A cancelled worker cannot publish a report into a newer run. File reads are size-bounded; cancellation invalidates their completion even if the browser finishes the read internally.

## Result semantics and presentation

- **VERIFIED** establishes only the displayed scope under the selected checkpoint and pinned profile. Synthetic success remains synthetic.
- **REJECTED** describes a failed supplied claim, not proof that the block is non-final.
- **INCONCLUSIVE** covers missing context/counterparts and unsupported evidence. Unperformed checks stay visible.
- **Genesis anchor verified** is migration evidence, not ordinary block finality or completed network migration.

The C8 complementary-color interface uses text, check/cross symbols, visible keyboard focus, native controls/disclosures, a polite result announcement, and error alerts. Controls stack on narrow screens; wide signer tables scroll within their region. Validator tables paginate at 50 rows. Untrusted names, descriptions, and diagnostics use React text rendering. No imported markup is injected. CSS motion respects reduced-motion preferences.

## Validation

```sh
bash scripts/check-web.sh
```

This runs the C1–C4 regression gate, C5/C6/C7 Rust tests, the production web build/typecheck, frontend formatting, 35 full native-versus-arbitrary-input-WASM report comparisons plus six experiment/replay and eight capture comparisons, eight worker lifecycle unit tests, and 26 Chrome product tests. Chrome/Chromium is required; `CHROME_BIN` overrides the default macOS/Linux executable.

Browser coverage includes empty/keyboard entry, explicit trust, fast/slow/genesis results, invalid signatures, under-quorum evidence, missing counterparts, malformed/oversized/non-UTF-8/saved-report inputs, exact u64/digest transport, hostile text, trust revocation/operator reasons, cancellation, missing WASM, delayed file reads, incompatible context/profile, narrow screens, reduced motion, and automated WCAG A/AA scans. Screenshots and failure traces are generated only under ignored `test-results/`. Automated accessibility checks do not replace assistive-technology testing.

Firefox, Safari, real mobile devices, production hosting, authenticated real checkpoints, and historical captures have not been verified by this checkpoint. The fixed Base2 profile, external checkpoint-authenticity assumption, and C3/C4 scope restrictions remain unchanged.

## Sources informing the boundary

- [React app setup](https://react.dev/learn/creating-a-react-app) and [Vite guide](https://vite.dev/guide/): a static local tool with explicit build and worker assets.
- [Worker termination](https://developer.mozilla.org/en-US/docs/Web/API/Worker/terminate): cancellation stops the worker instead of waiting for synchronous cryptography to cooperate.
- [WebAssembly memory growth](https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/JavaScript_interface/Memory/grow): recreate views after execution.
- Existing [contracts](contracts.md), [certificate verification](certificate-verification.md), and [finality policy](finality.md) remain authoritative for cryptographic behavior and trust limits.


## C8 presentation and navigation

The default page is now the landing page. Use **Open inspector** or `/#/inspect` for the evidence workspace. The redesigned reports, controls, traces and capture/replay tools preserve the verifier interface described above. The inspector stays mounted during same-session visits home, preserving explicit decisions; refreshes reset it. See [interface.md](interface.md) and [../DESIGN.md](../DESIGN.md).
