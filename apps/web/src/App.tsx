import { useEffect, useRef, useState } from "react";
import type {
  EpochCheckpoint,
  EvidenceVerificationReport,
  ProofBundle,
  ContentDigest,
  MutationAction,
  CaptureReport,
} from "../../../packages/contracts/src/index";
import {
  MAX_BUNDLE_BYTES,
  MAX_REPLAY_BYTES,
  MAX_CAPTURE_BYTES,
} from "../../../packages/contracts/src/index";
import examples from "../generated/examples.json";
import recordedCapture from "../generated/capture.json";
import { CaptureView } from "./CaptureView";
import {
  MAX_CHECKPOINT_BYTES,
  type Bundle,
  type Checkpoint,
  type Inspected,
  type Selection,
  type ReplayImport,
  type Comparison,
  type ExportedReplay,
} from "./protocol";
import VerifierWorker from "./verifier.worker.ts?worker&inline";
import { loadVerifierAssets } from "./assets";
import { TamperLab, ComparisonView } from "./TamperLab";
import { Runner } from "./runner";
import { Report, reportTitle, words } from "./Report";

const encoder = new TextEncoder();
export default function App() {
  const [assetsReady, setAssetsReady] = useState(false);
  const runner = useRef<Runner | null>(null);
  if (!runner.current)
    runner.current = new Runner(
      () => new VerifierWorker(),
      30_000,
      async () => {
        const bytes = await loadVerifierAssets();
        setAssetsReady(true);
        return bytes;
      },
    );
  const revision = useRef(0);
  const [bundle, setBundle] = useState<Bundle | null>(null);
  const [capture, setCapture] = useState<{
    bytes: Uint8Array;
    name: string;
    report: CaptureReport;
  } | null>(null);
  const [checkpoint, setCheckpoint] = useState<Checkpoint | null>(null);
  const [selection, setSelection] = useState<Selection | null>(null);
  const [basis, setBasis] = useState<"synthetic" | "operator">("synthetic");
  const [reason, setReason] = useState("");
  const [example, setExample] = useState(examples[0].id);
  const [report, setReport] = useState<EvidenceVerificationReport | null>(null);
  const [derived, setDerived] = useState<Bundle | null>(null);
  const [derivedReport, setDerivedReport] =
    useState<EvidenceVerificationReport | null>(null);
  const [activeReport, setActiveReport] = useState<"baseline" | "derived">(
    "baseline",
  );
  const [requiredCheckpoint, setRequiredCheckpoint] =
    useState<ContentDigest | null>(null);
  const [replayInfo, setReplayInfo] = useState("");
  const [includeCheckpoint, setIncludeCheckpoint] = useState(true);
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
  useEffect(() => {
    let mounted = true;
    loadVerifierAssets().then(
      () => {
        if (mounted) setAssetsReady(true);
      },
      () => {},
    );
    return () => {
      mounted = false;
    };
  }, []);
  function invalidate(message = "", preserveReports = false) {
    revision.current++;
    runner.current!.cancel();
    if (!preserveReports) {
      setReport(null);
      setDerivedReport(null);
    }
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
    setCapture(null);
    setBundle(null);
    setDerived(null);
    setRequiredCheckpoint(null);
    setReplayInfo("");
    setActiveReport("baseline");
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
    if (kind === "bundle") {
      setCapture(null);
      setBundle(null);
      setDerived(null);
      setRequiredCheckpoint(null);
      setReplayInfo("");
      setActiveReport("baseline");
    } else {
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
  async function importCapture(
    input: File | { bytes: Uint8Array; name: string },
  ) {
    const ticket = invalidate("Inspecting recorded genesis RPC response…");
    setCapture(null);
    setBundle(null);
    setDerived(null);
    setCheckpoint(null);
    setSelection(null);
    setRequiredCheckpoint(null);
    setReplayInfo("");
    setActiveReport("baseline");
    try {
      const size = input instanceof File ? input.size : input.bytes.byteLength;
      if (size > MAX_CAPTURE_BYTES)
        throw new Error("RPC response exceeds the 64 KiB file limit.");
      const bytes =
        input instanceof File
          ? new Uint8Array(await input.arrayBuffer())
          : input.bytes;
      if (ticket !== revision.current) return;
      const result = await runner.current!.run<CaptureReport>({
        kind: "capture",
        bytes,
      });
      if (ticket !== revision.current) return;
      setCapture({ bytes, name: input.name, report: result });
      setBusy("");
      setNotice(
        "RPC response inspected. No cryptographic verification or trust selection was performed.",
      );
    } catch (error) {
      failed(error, ticket);
    }
  }
  function downloadCapture() {
    if (!capture) return;
    const url = URL.createObjectURL(
      new Blob([new Uint8Array(capture.bytes)], { type: "application/json" }),
    );
    const a = document.createElement("a");
    a.href = url;
    a.download = "genesis-rpc-response.json";
    document.body.append(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
    setNotice("Original RPC response downloaded without modification.");
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
      if (derived) {
        const result = await runner.current!.run<Comparison>({
          kind: "compare",
          baseline: bundle.bytes,
          derived: derived.bytes,
          checkpoint: checkpoint?.bytes ?? null,
          selection,
          requiredCheckpoint,
        });
        if (ticket !== revision.current) return;
        setReport(result.baseline);
        setDerivedReport(result.derived);
        setActiveReport("derived");
        setBusy("");
        setNotice("Comparison complete. Both results were freshly verified.");
        return;
      }
      const result = await runner.current!.run<EvidenceVerificationReport>({
        kind: "verify",
        bundle: bundle.bytes,
        requiredCheckpoint,
        checkpoint: checkpoint?.bytes ?? null,
        selection,
      });
      if (ticket !== revision.current) return;
      setReport(result);
      setActiveReport("baseline");
      setBusy("");
      setNotice(reportTitle(result));
    } catch (error) {
      failed(error, ticket);
    }
  }
  async function createExperiment(
    certificateId: string,
    action: MutationAction,
  ) {
    if (!bundle) return;
    const ticket = invalidate(
      "Creating experiment from the immutable original…",
    );
    // A failed replacement must not leave a different older experiment visible.
    setDerived(null);
    try {
      const result = await runner.current!.run<Bundle>({
        kind: "mutate",
        baseline: bundle.bytes,
        certificateId,
        action,
      });
      if (ticket !== revision.current) return;
      setDerived(result);
      setBusy("");
      setNotice(
        "Experiment created. Verify the comparison to see which checks change.",
      );
    } catch (error) {
      failed(error, ticket);
    }
  }
  async function importReplay(file: File) {
    const ticket = invalidate(
      "Inspecting replay and checking its mutation history…",
    );
    setCapture(null);
    setBundle(null);
    setDerived(null);
    setCheckpoint(null);
    setSelection(null);
    setRequiredCheckpoint(null);
    setReplayInfo("");
    try {
      if (file.size > MAX_REPLAY_BYTES)
        throw new Error("Replay exceeds the 16 MiB file limit.");
      const bytes = new Uint8Array(await file.arrayBuffer());
      if (ticket !== revision.current) return;
      const result = await runner.current!.run<ReplayImport>({
        kind: "replay",
        bytes,
      });
      if (ticket !== revision.current) return;
      setBundle(result.baseline);
      setDerived(result.derived);
      if (result.checkpoint) acceptCheckpoint(result.checkpoint);
      setRequiredCheckpoint(result.requiredCheckpoint);
      setActiveReport("baseline");
      setBusy("");
      setReplayInfo(
        `Requires ${result.profileId} · ${result.policyId}. ${result.derived ? "Mutation history checked against the original. " : ""}${result.hasSavedReports ? "Saved reports were ignored. " : ""}Select a checkpoint and recompute.`,
      );
      setNotice(
        "Replay imported without trust or saved verdicts. Select a checkpoint and recompute.",
      );
    } catch (error) {
      failed(error, ticket);
    }
  }
  async function exportReplay() {
    if (!bundle) return;
    const ticket = invalidate("Preparing replay download…", true);
    try {
      const result = await runner.current!.run<ExportedReplay>({
        kind: "export",
        baseline: bundle.bytes,
        derived: derived?.bytes ?? null,
        checkpoint: includeCheckpoint ? (checkpoint?.bytes ?? null) : null,
        checkpointDigest:
          requiredCheckpoint ?? checkpoint?.parsed.digest ?? null,
        reports: { baseline: report, derived: derivedReport },
      });
      if (ticket !== revision.current) return;
      const url = URL.createObjectURL(
        new Blob([new Uint8Array(result.bytes)], { type: "application/json" }),
      );
      const a = document.createElement("a");
      a.href = url;
      a.download = result.filename;
      document.body.append(a);
      a.click();
      a.remove();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
      setBusy("");
      setNotice(
        "Replay downloaded. The recipient must select trust and recompute all results.",
      );
    } catch (error) {
      failed(error, ticket);
    }
  }
  const cp = checkpoint?.parsed.document;
  const shownReport =
    activeReport === "derived" && derivedReport ? derivedReport : report;
  const shownBundle =
    activeReport === "derived" && derivedReport && derived ? derived : bundle;
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
          <span aria-hidden="true">◉</span>{" "}
          {assetsReady
            ? import.meta.env.PROD
              ? "Ready for offline verification"
              : "Local verifier ready (dev)"
            : "Loading local verifier…"}
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
                        setCapture(null);
                        setBundle(null);
                        setDerived(null);
                        setRequiredCheckpoint(null);
                        setReplayInfo("");
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
            <section className="input-section capture-controls">
              <h2>Recorded RPC evidence</h2>
              <p className="small muted">
                Inspect a saved getAgGenesisCert response. Historical context is
                required before it can become verification input.
              </p>
              <button
                className="full-width"
                onClick={() =>
                  void importCapture({
                    bytes: encoder.encode(recordedCapture.source_json),
                    name: "Recorded testnet response",
                  })
                }
              >
                Load recorded testnet response
              </button>
              <p className="small muted">
                Supplied by the project owner; retrieval time and endpoint
                origin have not been independently established.
              </p>
              <label htmlFor="capture-file">Import genesis RPC response</label>
              <input
                id="capture-file"
                type="file"
                accept=".json,application/json"
                onChange={(e) => {
                  const file = e.target.files?.[0];
                  e.target.value = "";
                  if (file) void importCapture(file);
                }}
              />
              <p className="small muted">
                Raw JSON-RPC response · up to 64 KiB
              </p>
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
                disabled={Boolean(capture)}
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
                  {capture
                    ? "Capture inspection cannot use an unrelated checkpoint. Load a Monocle proof bundle to select verification context."
                    : "Load an example or import the validator set for your evidence."}
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
                {derived ? "Verify comparison" : "Verify evidence"}{" "}
                <span aria-hidden="true">↗</span>
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
                {capture
                  ? "This response lacks historical context. The inspection lists what must be supplied; verification has not run."
                  : !selection
                    ? "Without a selected checkpoint, verification remains inconclusive."
                    : "Verification uses the selected checkpoint and the pinned protocol profile."}
              </p>
            </section>
            <section className="input-section replay-controls">
              <h2>Portable replay</h2>
              <p className="small muted">
                Share original bytes, the experiment, and recorded reports.
                Trust must be selected again on import.
              </p>
              <label className="inline-label">
                <input
                  type="checkbox"
                  checked={includeCheckpoint}
                  disabled={!checkpoint}
                  onChange={(e) => setIncludeCheckpoint(e.target.checked)}
                />{" "}
                Include checkpoint bytes
              </label>
              <button
                className="full-width"
                disabled={!bundle || Boolean(busy)}
                onClick={exportReplay}
              >
                Export replay
              </button>
              <label htmlFor="replay-file">Import replay file</label>
              <input
                id="replay-file"
                type="file"
                accept=".json,application/json"
                onChange={(e) => {
                  const file = e.target.files?.[0];
                  e.target.value = "";
                  if (file) void importReplay(file);
                }}
              />
              <p className="small muted">Monocle replay v1 · up to 16 MiB</p>
              {replayInfo && <p className="small break">{replayInfo}</p>}
              {requiredCheckpoint && (
                <details>
                  <summary>Required checkpoint digest</summary>
                  <p className="mono small break">{requiredCheckpoint}</p>
                  <p className="small muted">
                    {checkpoint
                      ? "Select this exact checkpoint to reproduce the recorded context."
                      : "Checkpoint bytes are not included. Import the referenced checkpoint separately."}
                  </p>
                </details>
              )}
            </section>
          </aside>
          <div className="evidence-panel" aria-label="Evidence report">
            <div className="panel-top">
              <span>Inspection</span>
              <span className="small muted">
                {busy
                  ? "Working…"
                  : capture
                    ? "Capture inspected"
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
                  Correct the input or retry the operation.
                </p>
              </div>
            )}
            {capture && (
              <CaptureView
                report={capture.report}
                name={capture.name}
                download={downloadCapture}
                clear={() => {
                  invalidate();
                  setCapture(null);
                  setNotice("Capture cleared.");
                }}
              />
            )}
            {bundle && (
              <TamperLab
                key={bundle.parsed.digest}
                baseline={bundle}
                derived={derived}
                busy={Boolean(busy)}
                create={createExperiment}
                reset={() => {
                  invalidate();
                  setDerived(null);
                  setActiveReport("baseline");
                  setNotice(
                    "Experiment discarded. Original bytes are unchanged.",
                  );
                }}
              />
            )}
            {report && derivedReport && (
              <ComparisonView
                reports={{ baseline: report, derived: derivedReport }}
              />
            )}
            {report && derivedReport && (
              <div
                className="report-tabs"
                role="group"
                aria-label="Choose full report"
              >
                <button
                  aria-pressed={activeReport === "baseline"}
                  onClick={() => setActiveReport("baseline")}
                >
                  Original report
                </button>
                <button
                  aria-pressed={activeReport === "derived"}
                  onClick={() => setActiveReport("derived")}
                >
                  Experiment report
                </button>
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
            {shownReport && shownBundle ? (
              <Report
                key={shownReport.run_id}
                report={shownReport}
                bundle={shownBundle.parsed.document}
                checkpoint={selection ? (cp ?? null) : null}
              />
            ) : (
              !busy &&
              !capture && (
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
