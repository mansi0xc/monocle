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
import { Brand } from "./Brand";
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
  // One step is open at a time: load evidence, choose trust, then verify.
  const current = !bundle ? 0 : !selection ? 1 : 2;
  const done = [Boolean(bundle), Boolean(selection), Boolean(report)];
  const stepState = (index: number) =>
    done[index] ? "done" : current === index ? "current" : "pending";
  const stepMarker = (index: number) => (
    <span
      className="step"
      data-done={done[index]}
      aria-label={done[index] ? "Done" : `Step ${index + 1}`}
    >
      {done[index] ? "✓" : index + 1}
    </span>
  );
  const showLab = Boolean(selection || derived);
  const goToTrust = () => {
    const target =
      document.getElementById("basis") ?? document.getElementById("trust-step");
    target?.scrollIntoView({ block: "center", behavior: "smooth" });
    target?.focus();
  };
  return (
    <div className="inspector-app">
      <a className="skip-link" href="#inspector">
        Skip to inspector
      </a>
      <header className="app-header">
        <a className="app-brand" href="#/" aria-label="Monocle home">
          <Brand />
        </a>
        <nav className="app-nav" aria-label="Main navigation">
          <a href="#/">Overview</a>
          <a href="#/inspect" aria-current="page">
            Inspector
          </a>
        </nav>
        {bundle && (
          <span className="app-crumb">
            <span className="file-name">{bundle.name}</span>
            <span
              className="source-label"
              data-kind={bundle.parsed.document.provenance.kind}
            >
              {bundle.parsed.document.provenance.kind === "synthetic"
                ? "Synthetic evidence"
                : "Capture · declared provenance"}
            </span>
          </span>
        )}
        {report && derivedReport && (
          <div
            className="report-tabs"
            role="group"
            aria-label="Choose full report"
          >
            <button
              aria-pressed={activeReport === "baseline"}
              data-status={report.conclusion}
              onClick={() => setActiveReport("baseline")}
            >
              Original report
            </button>
            <button
              aria-pressed={activeReport === "derived"}
              data-status={derivedReport.conclusion}
              onClick={() => setActiveReport("derived")}
            >
              Experiment report
            </button>
          </div>
        )}
        <a
          className="app-source"
          href="https://github.com/mansi0xc/monocle"
          target="_blank"
          rel="noopener noreferrer"
        >
          Source <span aria-hidden="true">↗</span>
        </a>
        <span className="local-label" data-ready={assetsReady}>
          <span className="connection-dot" aria-hidden="true" />{" "}
          {assetsReady
            ? import.meta.env.PROD
              ? "Ready for offline verification"
              : "Local verifier ready (dev)"
            : "Loading local verifier…"}
        </span>
      </header>
      <main id="inspector" tabIndex={-1}>
        <h1 className="visually-hidden">Monocle evidence inspector</h1>
        <div
          className="workspace"
          data-has-result={Boolean(report || capture)}
          data-lab={showLab}
        >
          <aside
            className="inputs"
            aria-label="Evidence and checkpoint controls"
          >
            <section className="input-section" data-step={stepState(0)}>
              <h2>{stepMarker(0)} Evidence</h2>
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
              {!bundle && (
                <p className="small muted example-description">
                  {selectedExample.description}
                </p>
              )}
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
                  <p className="small muted">
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
              <details className="more">
                <summary>Import your own bundle</summary>
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
                <p className="small muted">Monocle JSON, up to 1 MiB</p>
              </details>
            </section>
            <section
              className="input-section trust-section"
              id="trust-step"
              tabIndex={-1}
              data-step={stepState(1)}
            >
              <h2>{stepMarker(1)} Checkpoint trust</h2>
              {checkpoint && cp ? (
                <div className="loaded-checkpoint">
                  <strong className="file-name">{checkpoint.name}</strong>
                  <dl className="compact-facts">
                    <dt>Source</dt>
                    <dd>
                      {cp.provenance.kind === "synthetic"
                        ? "Synthetic"
                        : "Capture"}
                    </dd>
                    <dt>Epoch</dt>
                    <dd className="mono">{cp.context.epoch}</dd>
                    <dt>Validators</dt>
                    <dd className="mono">{cp.validators.length}</dd>
                    <dt>Total stake</dt>
                    <dd className="mono">{cp.total_stake}</dd>
                  </dl>
                  {!selection ? (
                    <div className="trust-controls">
                      <p className="small muted">
                        Loading a checkpoint does not make it trusted. Choose
                        its basis deliberately.
                      </p>
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
                      <button
                        className="primary full-width"
                        onClick={selectCheckpoint}
                      >
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
                      <p className="small muted">
                        Applies to this session. Monocle does not authenticate
                        the checkpoint.
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
                </div>
              ) : (
                <p className="empty-note">
                  {capture
                    ? "Capture inspection cannot use an unrelated checkpoint. Load a Monocle proof bundle to select verification context."
                    : bundle
                      ? "Import the validator checkpoint for this evidence."
                      : "Signed examples bring their own checkpoint. Load evidence first."}
                </p>
              )}
              <details className="more" open={Boolean(bundle && !checkpoint)}>
                <summary>
                  {checkpoint
                    ? "Import a different checkpoint"
                    : "Import a checkpoint file"}
                </summary>
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
                <p className="small muted">
                  Monocle JSON, up to 4 MiB. Importing grants no trust.
                </p>
              </details>
            </section>
            <section className="input-section sources">
              <h2>Recorded evidence and replay</h2>
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
                A saved getAgGenesisCert response supplied by the project owner;
                its origin has not been independently established.
              </p>
              <div className="export-row">
                <button
                  disabled={!bundle || Boolean(busy)}
                  onClick={exportReplay}
                >
                  Export replay
                </button>
                <label className="inline-label">
                  <input
                    type="checkbox"
                    checked={includeCheckpoint}
                    disabled={!checkpoint}
                    onChange={(e) => setIncludeCheckpoint(e.target.checked)}
                  />{" "}
                  Include checkpoint bytes
                </label>
              </div>
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
              <details className="more">
                <summary>Import a recorded response or replay</summary>
                <label htmlFor="capture-file">
                  Import genesis RPC response
                </label>
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
                  Raw JSON-RPC response, up to 64 KiB. Historical context is
                  required before it can be verified.
                </p>
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
                <p className="small muted">
                  Monocle replay v1, up to 16 MiB. Trust is selected again and
                  every result is recomputed.
                </p>
              </details>
            </section>
          </aside>
          <section className="evidence-panel" aria-label="Evidence report">
            <div className="run-bar" data-step={stepState(2)}>
              <div className="run-copy">
                {stepMarker(2)}
                <div>
                  <strong>Verify</strong>
                  <p className="small muted">
                    {capture
                      ? "This response lacks historical context; verification has not run."
                      : !bundle
                        ? "Load evidence to begin."
                        : !selection
                          ? "Without a selected checkpoint, verification stays inconclusive."
                          : "Uses the selected checkpoint and the pinned protocol profile."}
                  </p>
                </div>
              </div>
              <div className="run-actions">
                {busy && (
                  <button
                    onClick={() => {
                      invalidate();
                      setNotice("Operation cancelled. No result retained.");
                    }}
                  >
                    Cancel operation
                  </button>
                )}
                <button
                  className={current === 2 ? "primary" : undefined}
                  disabled={!bundle || Boolean(busy)}
                  onClick={verify}
                >
                  {derived ? "Verify comparison" : "Verify evidence"}
                </button>
              </div>
            </div>
            <div className="panel-top">
              <div className="announcement" role="status" aria-live="polite">
                {notice}
              </div>
              <span className="small muted panel-state">
                {busy
                  ? "Working…"
                  : capture
                    ? "Capture inspected"
                    : report
                      ? "Run complete"
                      : "Awaiting verification"}
              </span>
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
            {busy && (
              <div className="working">
                <div className="loading-lines" aria-hidden="true">
                  <span />
                  <span />
                  <span />
                </div>
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
                experiment={
                  activeReport === "derived" && Boolean(derivedReport)
                }
              />
            ) : (
              !busy &&
              !capture && (
                <div className="empty-report" data-next={current}>
                  {!bundle ? (
                    <>
                      <p className="eyebrow">Your local workspace</p>
                      <h2>Every conclusion needs a trace.</h2>
                      <p>
                        Start with a signed example. Monocle shows each check,
                        from the original bytes to the finality claim, and why
                        it holds or fails.
                      </p>
                      <button className="primary" onClick={loadExample}>
                        Start with {selectedExample.title.toLowerCase()}
                      </button>
                      <p className="empty-footnote">
                        Built-in examples are synthetic: public test keys and
                        invented stake. No network connection is needed.
                      </p>
                    </>
                  ) : !selection ? (
                    <>
                      <p className="eyebrow">Next step</p>
                      <h2>Choose the checkpoint you trust.</h2>
                      <p>
                        The evidence has been structurally inspected. Signatures
                        and stake can only be checked against a validator
                        checkpoint, and Monocle never picks one for you.
                      </p>
                      <button onClick={goToTrust}>
                        Go to checkpoint trust
                      </button>
                    </>
                  ) : (
                    <>
                      <p className="eyebrow">Ready</p>
                      <h2>Ready to verify.</h2>
                      <p>
                        Verification checks each certificate’s signature, signer
                        keys and exact stake, then the finality claim. You can
                        also create an experiment first and verify both
                        together.
                      </p>
                    </>
                  )}
                </div>
              )
            )}
          </section>
          {showLab && (
            <section className="lab-panel" aria-label="Experiments">
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
            </section>
          )}
        </div>
      </main>
      <footer className="app-footer">
        <span>Monocle, a fixed-checkpoint research tool for Alpenglow</span>
        <span>Authenticity of the checkpoint is an external assumption.</span>
      </footer>
    </div>
  );
}
