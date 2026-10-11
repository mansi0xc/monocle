import { useEffect, useRef, useState } from "react";
import type {
  EpochCheckpoint,
  EvidenceVerificationReport,
  ProofBundle,
} from "../../../packages/contracts/src/index";
import { MAX_BUNDLE_BYTES } from "../../../packages/contracts/src/index";
import examples from "../generated/examples.json";
import {
  MAX_CHECKPOINT_BYTES,
  type Bundle,
  type Checkpoint,
  type Inspected,
  type Selection,
} from "./protocol";
import { Runner } from "./runner";
import { Report, reportTitle, words } from "./Report";

const encoder = new TextEncoder();
export default function App() {
  const runner = useRef<Runner | null>(null);
  if (!runner.current)
    runner.current = new Runner(
      () =>
        new Worker(new URL("./verifier.worker.ts", import.meta.url), {
          type: "module",
        }),
    );
  const revision = useRef(0);
  const [bundle, setBundle] = useState<Bundle | null>(null);
  const [checkpoint, setCheckpoint] = useState<Checkpoint | null>(null);
  const [selection, setSelection] = useState<Selection | null>(null);
  const [basis, setBasis] = useState<"synthetic" | "operator">("synthetic");
  const [reason, setReason] = useState("");
  const [example, setExample] = useState(examples[0].id);
  const [report, setReport] = useState<EvidenceVerificationReport | null>(null);
  const [busy, setBusy] = useState("");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("Ready to inspect local evidence.");
  const selectedExample = examples.find((item) => item.id === example)!;
  useEffect(
    () => () => {
      revision.current++;
      runner.current?.cancel();
    },
    [],
  );
  function invalidate(message = "") {
    revision.current++;
    runner.current!.cancel();
    setReport(null);
    setError("");
    setBusy(message);
    setNotice(message);
    return revision.current;
  }
  function failed(error: unknown, ticket: number) {
    if (ticket !== revision.current) return;
    setBusy("");
    if (error instanceof Error && error.name === "AbortError") return;
    setError(
      error instanceof Error ? error.message : "Unable to inspect this input.",
    );
    setNotice("The operation failed. Review the input message.");
  }
  function acceptCheckpoint(value: Checkpoint) {
    setCheckpoint(value);
    setSelection(null);
    setReason("");
    setBasis(
      value.parsed.document.provenance.kind === "synthetic"
        ? "synthetic"
        : "operator",
    );
  }
  async function loadExample() {
    const ticket = invalidate("Inspecting the synthetic example…");
    setBundle(null);
    setCheckpoint(null);
    setSelection(null);
    const b = encoder.encode(selectedExample.bundle_json);
    const c = encoder.encode(selectedExample.checkpoint_json);
    try {
      const result = await runner.current!.run<{
        bundle: Inspected<ProofBundle>;
        checkpoint: Inspected<EpochCheckpoint>;
      }>({ kind: "example", bundle: b, checkpoint: c });
      if (ticket !== revision.current) return;
      setBundle({
        bytes: b,
        parsed: result.bundle,
        name: selectedExample.title,
      });
      acceptCheckpoint({
        bytes: c,
        parsed: result.checkpoint,
        name: "Synthetic epoch 42",
      });
      setBusy("");
      setNotice("Example loaded. Select its checkpoint before verifying.");
    } catch (error) {
      failed(error, ticket);
    }
  }
  async function importFile(kind: "bundle" | "checkpoint", file: File) {
    const ticket = invalidate(`Inspecting ${kind}…`);
    if (kind === "bundle") setBundle(null);
    else {
      setCheckpoint(null);
      setSelection(null);
    }
    try {
      const limit = kind === "bundle" ? MAX_BUNDLE_BYTES : MAX_CHECKPOINT_BYTES;
      if (file.size > limit)
        throw new Error(
          `${kind === "bundle" ? "Bundle" : "Checkpoint"} exceeds the ${limit / 1024 / 1024} MiB file limit.`,
        );
      const bytes = new Uint8Array(await file.arrayBuffer());
      if (ticket !== revision.current) return;
      if (kind === "bundle") {
        const parsed = await runner.current!.run<Inspected<ProofBundle>>({
          kind,
          bytes,
        });
        if (ticket !== revision.current) return;
        setBundle({ bytes, parsed, name: file.name });
      } else {
        const parsed = await runner.current!.run<Inspected<EpochCheckpoint>>({
          kind,
          bytes,
        });
        if (ticket !== revision.current) return;
        acceptCheckpoint({ bytes, parsed, name: file.name });
      }
      setBusy("");
      setNotice(
        `${kind === "bundle" ? "Bundle" : "Checkpoint"} inspected. Verification has not run.`,
      );
    } catch (error) {
      failed(error, ticket);
    }
  }
  function selectCheckpoint() {
    if (!checkpoint) return;
    invalidate();
    if (
      basis === "operator" &&
      (!reason.trim() || encoder.encode(reason).length > 1024)
    ) {
      setError("Record a trust reason between 1 and 1024 UTF-8 bytes.");
      return;
    }
    setSelection({
      digest: checkpoint.parsed.digest,
      basis,
      reason: basis === "operator" ? reason : "",
    });
    setNotice(
      "Checkpoint selected for this session. Evidence still needs verification.",
    );
  }
  async function verify() {
    if (!bundle) return;
    const ticket = invalidate("Verifying evidence in the local worker…");
    try {
      const result = await runner.current!.run<EvidenceVerificationReport>({
        kind: "verify",
        bundle: bundle.bytes,
        checkpoint: checkpoint?.bytes ?? null,
        selection,
      });
      if (ticket !== revision.current) return;
      setReport(result);
      setBusy("");
      setNotice(reportTitle(result));
    } catch (error) {
      failed(error, ticket);
    }
  }
  const cp = checkpoint?.parsed.document;
  return (
    <>
      <a className="skip-link" href="#inspector">
        Skip to inspector
      </a>
      <header className="app-header">
        <a className="brand" href="./" aria-label="Monocle home">
          <svg width="29" height="29" viewBox="0 0 32 32" aria-hidden="true">
            <circle
              cx="14"
              cy="14"
              r="10"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            />
            <path
              d="M21 21v6h7"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            />
          </svg>
          monocle<span className="brand-period">.</span>
        </a>
        <span className="header-label">Alpenglow evidence inspector</span>
        <span className="local-label">
          <span aria-hidden="true">◉</span> Local verification
        </span>
      </header>
      <main id="inspector">
        <div className="page-heading">
          <div>
            <h1>Follow the evidence.</h1>
            <p>
              Inspect the certificates, signers, and assumptions behind
              finality.
            </p>
          </div>
          <span className="profile-label">
            Base2 profile <span className="mono">98535e50</span>
          </span>
        </div>
        <div className="workspace">
          <aside
            className="inputs"
            aria-label="Evidence and checkpoint controls"
          >
            <section className="input-section">
              <h2>
                <span className="step">1</span> Evidence
              </h2>
              <label htmlFor="example">Signed synthetic example</label>
              <div className="example-control">
                <select
                  id="example"
                  value={example}
                  onChange={(e) => setExample(e.target.value)}
                >
                  {examples.map((e) => (
                    <option key={e.id} value={e.id}>
                      {e.title}
                    </option>
                  ))}
                </select>
                <button onClick={loadExample}>Load</button>
              </div>
              <p className="small muted example-description">
                {selectedExample.description}
              </p>
              <label htmlFor="bundle-file">Or import a proof bundle</label>
              <input
                id="bundle-file"
                type="file"
                accept=".json,application/json"
                onChange={(e) => {
                  const file = e.target.files?.[0];
                  e.target.value = "";
                  if (file) void importFile("bundle", file);
                }}
              />
              <p className="small muted">Monocle JSON · up to 1 MiB</p>
              {bundle && (
                <div className="loaded-input">
                  <div className="section-heading">
                    <strong className="file-name">{bundle.name}</strong>
                    <button
                      className="text-button"
                      onClick={() => {
                        invalidate();
                        setBundle(null);
                        setNotice("Evidence cleared.");
                      }}
                    >
                      Clear evidence
                    </button>
                  </div>
                  <span className="source-label">
                    {bundle.parsed.document.provenance.kind === "synthetic"
                      ? "Synthetic evidence"
                      : "Capture · declared provenance"}
                  </span>
                  <p className="small">
                    {bundle.parsed.document.provenance.description}
                  </p>
                  <dl className="compact-facts">
                    <dt>Claim</dt>
                    <dd>{words(bundle.parsed.document.claim.kind)}</dd>
                    <dt>Slot</dt>
                    <dd className="mono">
                      {bundle.parsed.document.claim.slot}
                    </dd>
                    <dt>Epoch</dt>
                    <dd className="mono">
                      {bundle.parsed.document.context.epoch}
                    </dd>
                  </dl>
                  <details>
                    <summary>Input identity</summary>
                    <p className="mono small break">{bundle.parsed.digest}</p>
                    <p className="small muted">
                      SHA-256 identifies these original bytes; it does not
                      authenticate their source.
                    </p>
                    <dl className="compact-facts">
                      <dt>Profile</dt>
                      <dd className="mono">
                        {bundle.parsed.document.profile_id}
                      </dd>
                      <dt>Block ID</dt>
                      <dd className="mono">
                        {bundle.parsed.document.claim.block_id_hex ??
                          "Not supplied"}
                      </dd>
                      <dt>Genesis hash</dt>
                      <dd className="mono">
                        {bundle.parsed.document.context.genesis_hash_hex}
                      </dd>
                      <dt>Shred version</dt>
                      <dd>{bundle.parsed.document.context.shred_version}</dd>
                      <dt>Certificates</dt>
                      <dd>
                        {bundle.parsed.document.claim.certificate_ids.join(
                          ", ",
                        )}
                      </dd>
                      <dt>Checkpoint reference</dt>
                      <dd className="mono">
                        {bundle.parsed.document.checkpoint_reference ??
                          "Not supplied"}
                      </dd>
                    </dl>
                  </details>
                </div>
              )}
            </section>
            <section className="input-section">
              <h2>
                <span className="step">2</span> Validator checkpoint
              </h2>
              <p className="small muted">
                Importing a checkpoint does not make it trusted. Select its
                basis separately.
              </p>
              <label htmlFor="checkpoint-file">Import checkpoint</label>
              <input
                id="checkpoint-file"
                type="file"
                accept=".json,application/json"
                onChange={(e) => {
                  const file = e.target.files?.[0];
                  e.target.value = "";
                  if (file) void importFile("checkpoint", file);
                }}
              />
              <p className="small muted">Monocle JSON · up to 4 MiB</p>
              {checkpoint && cp ? (
                <div className="loaded-input">
                  <strong className="file-name">{checkpoint.name}</strong>
                  <p className="small">
                    {cp.provenance.kind === "synthetic"
                      ? "Synthetic"
                      : "Capture"}{" "}
                    · epoch <span className="mono">{cp.context.epoch}</span> ·{" "}
                    {cp.validators.length} validators
                  </p>
                  <p className="small">
                    Total stake <span className="mono">{cp.total_stake}</span>
                  </p>
                  <details>
                    <summary>Checkpoint identity</summary>
                    <p className="mono small break">
                      {checkpoint.parsed.digest}
                    </p>
                    <p className="small">{cp.provenance.description}</p>
                    <dl className="compact-facts">
                      <dt>Genesis hash</dt>
                      <dd className="mono break">
                        {cp.context.genesis_hash_hex}
                      </dd>
                      <dt>Shred version</dt>
                      <dd>{cp.context.shred_version}</dd>
                    </dl>
                  </details>
                  {!selection ? (
                    <div className="trust-controls">
                      <label htmlFor="basis">Trust basis</label>
                      <select
                        id="basis"
                        value={basis}
                        onChange={(e) => {
                          invalidate();
                          setBasis(e.target.value as "synthetic" | "operator");
                        }}
                      >
                        {cp.provenance.kind === "synthetic" && (
                          <option value="synthetic">
                            Synthetic fixture only
                          </option>
                        )}
                        <option value="operator">
                          Operator-pinned checkpoint
                        </option>
                      </select>
                      {basis === "operator" ? (
                        <>
                          <label htmlFor="reason">
                            Why do you trust this checkpoint?
                          </label>
                          <textarea
                            id="reason"
                            rows={3}
                            maxLength={1024}
                            value={reason}
                            onChange={(e) => setReason(e.target.value)}
                            placeholder="Record the source and how you checked it"
                          />
                          <p className="small muted">
                            This records your assumption; Monocle does not
                            authenticate it.
                          </p>
                        </>
                      ) : (
                        <p className="small muted">
                          Public test keys and invented context. This selection
                          applies only to synthetic evidence.
                        </p>
                      )}
                      <button className="full-width" onClick={selectCheckpoint}>
                        Use this checkpoint
                      </button>
                    </div>
                  ) : (
                    <div className="trust-selected">
                      <strong>✓ Checkpoint selected</strong>
                      <p className="small">
                        {selection.basis === "synthetic"
                          ? "Synthetic fixture only"
                          : `Operator-pinned: ${selection.reason}`}
                      </p>
                      <button
                        className="text-button"
                        onClick={() => {
                          invalidate();
                          setSelection(null);
                          setNotice("Checkpoint trust removed.");
                        }}
                      >
                        Remove selection
                      </button>
                    </div>
                  )}
                </div>
              ) : (
                <p className="empty-note">
                  Load an example or import the validator set for your evidence.
                </p>
              )}
            </section>
            <section className="input-section run-section">
              <h2>
                <span className="step">3</span> Verify
              </h2>
              <button
                className="primary full-width"
                disabled={!bundle || Boolean(busy)}
                onClick={verify}
              >
                Verify evidence <span aria-hidden="true">↗</span>
              </button>
              {busy && (
                <button
                  className="full-width"
                  onClick={() => {
                    invalidate();
                    setNotice("Operation cancelled. No result retained.");
                  }}
                >
                  Cancel operation
                </button>
              )}
              <p className="small muted">
                {!selection
                  ? "Without a selected checkpoint, verification remains inconclusive."
                  : "Verification uses the selected checkpoint and the pinned protocol profile."}
              </p>
            </section>
          </aside>
          <div className="evidence-panel" aria-label="Evidence report">
            <div className="panel-top">
              <span>Inspection</span>
              <span className="small muted">
                {busy
                  ? "Working…"
                  : report
                    ? "Run complete"
                    : "Awaiting verification"}
              </span>
            </div>
            <div className="announcement" role="status" aria-live="polite">
              {notice}
            </div>
            {error && (
              <div className="error" role="alert">
                <strong>Could not complete the operation</strong>
                <p>{error}</p>
                <p className="small">
                  Correct the input or retry the operation. No verification
                  result was retained.
                </p>
              </div>
            )}
            {busy && (
              <div className="working">
                <span className="spinner" aria-hidden="true" />
                <p>{busy}</p>
                <span className="small muted">
                  You can cancel or load different evidence at any time.
                </span>
              </div>
            )}
            {report && bundle ? (
              <Report
                key={report.run_id}
                report={report}
                bundle={bundle.parsed.document}
                checkpoint={selection ? (cp ?? null) : null}
              />
            ) : (
              !busy && (
                <div className="empty-report">
                  <div className="empty-symbol" aria-hidden="true">
                    ⊙
                  </div>
                  <h2>
                    {bundle
                      ? "Ready for inspection."
                      : "Every conclusion needs a trace."}
                  </h2>
                  <p>
                    {bundle
                      ? "Your input has been structurally inspected. Run verification to check signatures, signer weight, and the requested finality claim."
                      : "Load a signed example or import your evidence. Monocle makes each verification step visible, from the original bytes to the finality claim."}
                  </p>
                  <ol className="pipeline">
                    <li>
                      <strong>Decode</strong>
                      <span>Certificate bytes & signer ranks</span>
                    </li>
                    <li>
                      <strong>Authenticate</strong>
                      <span>BLS signatures & key qualification</span>
                    </li>
                    <li>
                      <strong>Evaluate</strong>
                      <span>Stake thresholds & finality scope</span>
                    </li>
                  </ol>
                  <p className="empty-footnote">
                    The built-in examples are synthetic. No live network
                    connection is required to verify local evidence.
                  </p>
                </div>
              )
            )}
          </div>
        </div>
      </main>
      <footer>
        <span>Monocle · Fixed-checkpoint research tool</span>
        <span>Authenticity of the checkpoint is an external assumption.</span>
      </footer>
    </>
  );
}
