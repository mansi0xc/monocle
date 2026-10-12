# Landing page and inspector interface

C8 is a presentation and navigation checkpoint. It adds no consensus functionality or cryptographic guarantees.

## Entry points

The default page introduces Monocle and links to `#/inspect`. The inspector route uses a fragment so static hosting does not require a server-side route fallback. The documentation and source links open the public repository in a separate tab.

The landing workflow preview has three keyboard-operable buttons: Verify, Tamper, and Replay. It demonstrates report concepts with labeled synthetic illustrations. It neither loads evidence nor selects a checkpoint. “Open inspector” opens the real application with its existing empty state.

## Working with evidence

The inspector begins with an ordered guide and the actual evidence/checkpoint/verify controls. The result surface presents scope and trust basis before certificate details. A captured RPC response has a separate inconclusive report; its data cannot inherit a synthetic result. The tamper lab, original/experiment reports, raw-byte disclosures, signer tables, and diagnostic codes remain available.

Recorded RPC import and portable replay are grouped below the main workspace. File size limits, source caveats, checkpoint choices, exact digests, and fresh verification requirements remain visible. The UI preserves the existing native file, select, checkbox, and disclosure semantics.

## Navigation and session lifetime

After the first visit, the inspector stays mounted while the landing page is shown. Going home and back, or using browser history, preserves local inputs, an explicitly selected checkpoint, and the current report. This does not create new trust; removing/replacing the checkpoint or importing a replay still follows the original invalidation rules. There is no localStorage/sessionStorage persistence. Refreshing or closing the page discards the session.

The inspector is loaded lazily. Visiting only the landing page does not mean the WASM verifier is loaded or ready offline. Offline support still requires opening the production inspector, waiting for **Ready for offline verification**, and keeping the session open. A cold visit or refresh without network access is unsupported.

## Design and assets

The owner's complementary-color request replaces the earlier black-and-white product brief. The selected cream canvas, navy text/structure, and burgundy actions follow the owner-supplied 2026-10-13 palette. The supplied feather/lens reference defines the shared mark. All three colours are used consistently across the landing and inspector; explicit labels and symbols distinguish report states. Fonts are bundled locally with licenses, as are the logo/favicon; the optical illustration is vector code. No new runtime dependency, remote image, analytics endpoint, or external font request is introduced.

The implementation details and token values live in [../DESIGN.md](../DESIGN.md). Cryptographic scope remains in [cryptography.md](cryptography.md), [finality.md](finality.md), and [capture-import.md](capture-import.md).

## Review

Check the landing page at desktop and mobile widths; switch all three preview modes. Enter the inspector, verify the synthetic fast example after selecting its checkpoint, create a signature mutation, compare both reports, and export/reimport a replay. Visit the landing page and return to confirm the same session remains available. Inspect the recorded response and observe the missing-context outcome. No real-network finality claim is added by this checkpoint.
