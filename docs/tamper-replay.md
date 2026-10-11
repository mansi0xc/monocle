# C6 tamper lab and portable replay

Monocle can derive one controlled certificate mutation from an immutable original, recompute both reports under the same checkpoint selection, and export the exact inputs for another local run. Replay files are evidence containers, never instructions to trust a checkpoint or accept a saved verdict.

## Walkthrough

1. Build and open the inspector using the commands in [browser-inspector.md](browser-inspector.md).
2. Load **Fast finality**, explicitly choose **Use this checkpoint**, and verify the genuine synthetic baseline.
3. Open **Tamper lab**, select the certificate and **Flip one signature bit**, then create the experiment. The original remains byte-for-byte unchanged.
4. Verify the comparison. Both results come from fresh Rust verification in the same worker/context. **What changed?** displays changed status, code, or message by stage and certificate; disable its filter to inspect every check. Switch between the full original and experiment reports below it.
5. Export a replay. Checkpoint bytes are included by default; they may be omitted while retaining the required digest.
6. Import the downloaded file. The app clears the current session inputs/trust/results, validates the archive and mutation history, and imports the checkpoint only as a candidate. Saved reports are ignored.
7. Explicitly select the checkpoint and verify again. With the same bytes, profile, policy, and trust basis, the substantive results reproduce. Local run IDs are new.

The **Insufficient stake** example is a separate signed control. Its signature passes and quorum fails. Changing the bitmap of an existing signature is not a way to produce that control.

## Mutation contract

`monocle_core::experiment::apply_mutation` accepts schema version 1, the exact baseline JSON string, a certificate ID, and one action:

```json
{"kind":"signature_bit"}
```

or

```json
{"kind":"replace_bytes","offset":"2","after_hex":"2b"}
```

Offsets are canonical decimal u64 strings. The replacement is 1–4096 bytes of lowercase even-length hexadecimal, must fit within the selected certificate, and must change at least one byte. It replaces the same number of original bytes, so certificate length does not change. Overflow, numeric JSON offsets, unknown fields, no-ops, and chained experiments fail.

The guided action requires the pinned full Base2 wire format and uses the decoded certificate kind. It flips the low bit of the first signature byte: offset 10 for Finalize and 42 for the other supported kinds. That edit can produce an invalid point encoding or an invalid aggregate signature. Raw replacement intentionally permits malformed certificate bytes so the real verifier can explain their failure.

Each experiment starts from an unmodified baseline with no parent/mutation history. Creating a replacement experiment starts again from that original. The core clones the original document, changes one certificate's raw bytes, and records:

- The SHA-256 digest of the exact original bundle source.
- Certificate ID, byte offset, and equal-length before/after bytes.
- SHA-256 identities of the original and changed raw certificate.

The derived document has a new serialization and digest; the original input string is preserved exactly. No claim, context, provenance, other certificate, or checkpoint field is silently rewritten. Consequently, editing a signed slot may fail `CLAIM_MISMATCH` before signature verification. The UI shows the actual stage and leaves downstream checks unperformed.

`validate_lineage` reconstructs the replacement from the supplied original and compares the complete derived document, including all history fields. This catches forged before-bytes, wrong parent/certificate digests, unrelated edits, and changed original whitespace. It establishes a content relationship; it does not authenticate an author or a network capture. Standalone C2 bundle parsing alone cannot establish this history without the original.

## Replay v1

A replay is UTF-8 JSON with these fields. Unknown archive fields and duplicate JSON keys are rejected by the strict parser; saved report contents remain bounded untrusted values.

| Field | Meaning |
| --- | --- |
| `schema_version` | Integer `1` |
| `format` | `monocle-replay` |
| `profile_id` | Must match the baseline; experiments use `agave-98535e50-base2-v1` |
| `policy_id` | Must equal `agave-98535e50-direct-finality-v1` |
| `baseline_json` | Exact original bundle source as a JSON string |
| `derived_json` | Exact experiment source string, or null when exporting only the baseline |
| `checkpoint_json` | Exact checkpoint source string, or null when supplied separately |
| `checkpoint_digest` | Expected checkpoint identity, or null if no reference/candidate is available |
| `saved_reports` | Optional baseline/derived values from a previous run; always untrusted |

The baseline's checkpoint reference, if present, must agree with the archive. Included checkpoint bytes must match the archive digest. An omitted checkpoint is not synthesized or fetched: the recipient supplies the exact referenced bytes and selects them explicitly. A whitespace change changes the checkpoint identity. Selecting a different digest for an imported replay produces an execution error identifying the missing requirement. Without a selected checkpoint, the real core returns an inconclusive result.

Unknown replay versions and finality policies are rejected. A profile label must match the original bundle; an unsupported original profile can be retained as evidence but cannot produce verification success. The reader does not silently translate profiles or rewrite signed data.

The Rust archive parser never converts saved results into verifier reports or updates a trust store. The browser import worker discards those values and exposes only their presence to the page. To obtain current results, the worker reparses the checkpoint, applies the explicit local trust basis, and calls `finality::verify_evidence` on both original inputs. Both comparison run IDs must match the active request. Cancellation, input/context replacement, and deadlines invalidate the pair together.

Export preserves current reports and may include them for inspection outside the app, along with their run IDs, input identities, and assumptions. Such text can be edited by anyone. It has no authority on re-import. Exporting also revalidates the archive before downloading it. A checkpoint/reference mismatch fails export rather than producing a contradictory archive.

## Limits and byte boundary

| Resource | Limit |
| --- | --- |
| Original or derived bundle | 1 MiB each, including newly serialized mutation metadata |
| Checkpoint | 4 MiB |
| One byte replacement | 1–4096 bytes |
| Replay file | 16 MiB, including JSON escaping and optional saved reports |
| One saved report | 1 MiB of serialized JSON |
| Outer replay decoded string | 4 MiB; embedded source then faces its own normal parser limits |
| WASM bridge input allocation | 16 MiB |
| JavaScript WASM output read | 32 MiB |
| One worker job | 30 seconds including preparation; then terminate |

The outer archive cap is a total, not a guarantee that every maximum-sized combination of escaped strings/reports fits. Existing depth, node, array, object, key, and nested input limits still apply. C2/C4 verification requests retain their `6 MiB + 4096` cap. Raising the transport allocation cap does not relax an operation's core parser limits.

The byte ABI is version **2**. Existing operations 1–6 remain available; operation 7 applies a mutation and returns the derived source/document/digest, and operation 8 parses and validates a replay. Allocation/execution success codes remain `1`; the ABI version is a separate export. Old host/WASM version combinations fail explicitly. No new dependency or cryptographic primitive is introduced.

## Offline behavior

Use the **production build**, then keep its page open:

```sh
npm run build:web
npm run preview
```

Wait for **Ready for offline verification**. The application has loaded its inline worker script and cached WASM bytes in page memory. Each fresh worker receives a copy of that buffer. Built-in examples are part of the loaded application, and evidence/checkpoints/replays can be read from local files. Import, mutation, verification, comparison, download, and re-import then require no HTTP requests.

This is not persistent offline installation. Refreshing or closing the page loses assets and session state; an offline first visit or refresh is unsupported. There is no service worker or IndexedDB cache. Development mode can request modules from Vite and therefore displays **Local verifier ready (dev)** instead. Asset loading failures can be retried after reconnecting. Offline capability does not authenticate downloaded application code or checkpoint contents.

## Validation and remaining scope

Run `bash scripts/check-web.sh`. C6 adds ten native tests, six native/WASM operation comparisons, two worker lifecycle tests, and seven Chrome browser tests to the prior gates. Replay tests recompute full reports, normalize only run IDs, alter saved verdicts and history, omit/replace checkpoints, exercise raw-slot and signature mutations, and test invalid ranges and resource caps. The offline test disconnects the browser context after readiness and records zero HTTP requests through the complete experiment/replay workflow. The comparison is also checked at desktop/narrow sizes and with automated accessibility scans.

This remains a fixed-checkpoint debugger using a pinned Base2 profile. No real testnet evidence has been verified, no checkpoint has been authenticated from a network trust root, and no additional finality paths have been added. Multiple patches, chained histories, persistent offline installation, and broad browser/device validation are outside C6. The next owner-approved checkpoint is C7 capture integration and demonstration hardening.
