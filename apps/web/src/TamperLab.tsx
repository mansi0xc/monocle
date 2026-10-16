import { useState } from "react";
import {
  BASE2_PROFILE_ID,
  type CheckResult,
  type MutationAction,
} from "../../../packages/contracts/src/index";
import type { Bundle, Comparison } from "./protocol";
import { words } from "./Report";
export function TamperLab({
  baseline,
  derived,
  busy,
  create,
  reset,
}: {
  baseline: Bundle;
  derived: Bundle | null;
  busy: boolean;
  create: (id: string, action: MutationAction) => void;
  reset: () => void;
}) {
  const [certificateId, setCertificateId] = useState(
    baseline.parsed.document.certificates[0].id,
  );
  const [kind, setKind] = useState("signature_bit");
  const [offset, setOffset] = useState("0");
  const [after, setAfter] = useState("");
  const mutation = derived?.parsed.document.mutations[0];
  const supported =
    baseline.parsed.document.profile_id === BASE2_PROFILE_ID &&
    !baseline.parsed.document.parent_digest;
  return (
    <details className="tamper-lab" open={Boolean(derived)}>
      <summary>
        Tamper lab{" "}
        <span className="muted">
          {derived ? "One recorded change" : "Create a separate experiment"}
        </span>
      </summary>
      <div className="lab-body">
        <p className="small muted">
          The original stays byte-for-byte unchanged. Each experiment starts
          from that original; changes are never stacked.
        </p>
        {mutation && (
          <div className="mutation-summary">
            <strong>Recorded replacement</strong>
            <p className="small">
              <span className="mono">{mutation.certificate_id}</span> · byte
              offset <span className="mono">{mutation.offset}</span> ·{" "}
              {mutation.after_hex.length / 2} byte(s)
            </p>
            <dl className="patch-values">
              <dt>Before</dt>
              <dd className="mono">{mutation.before_hex}</dd>
              <dt>After</dt>
              <dd className="mono">{mutation.after_hex}</dd>
            </dl>
            <button className="text-button" onClick={reset}>
              Discard experiment
            </button>
          </div>
        )}
        {supported ? (
          <form
            onSubmit={(e) => {
              e.preventDefault();
              create(
                certificateId,
                kind === "signature_bit"
                  ? { kind: "signature_bit" }
                  : { kind: "replace_bytes", offset, after_hex: after },
              );
            }}
          >
            <div className="lab-fields">
              <label>
                Certificate
                <select
                  aria-label="Certificate"
                  value={certificateId}
                  onChange={(e) => setCertificateId(e.target.value)}
                >
                  {baseline.parsed.document.certificates.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.id} · {words(c.kind)}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                Experiment
                <select
                  aria-label="Experiment"
                  value={kind}
                  onChange={(e) => setKind(e.target.value)}
                >
                  <option value="signature_bit">Flip one signature bit</option>
                  <option value="replace_bytes">
                    Replace raw certificate bytes
                  </option>
                </select>
              </label>
            </div>
            {kind === "replace_bytes" && (
              <div className="lab-fields">
                <label>
                  Byte offset
                  <input
                    inputMode="numeric"
                    value={offset}
                    onChange={(e) => setOffset(e.target.value)}
                    maxLength={20}
                    required
                  />
                </label>
                <label>
                  Replacement hex
                  <input
                    className="mono"
                    value={after}
                    onChange={(e) => setAfter(e.target.value)}
                    maxLength={8192}
                    placeholder="Lowercase hex, e.g. ff"
                    required
                  />
                </label>
              </div>
            )}
            <p className="small muted">
              {kind === "signature_bit"
                ? "Changes the signature while preserving the payload and signer selection. The result may fail point decoding or signature verification."
                : "An equal-length replacement, up to 4096 bytes. Changing a signed field can fail claim binding before the signature check; changing a bitmap can alter both keys and stake."}
            </p>
            <button type="submit" disabled={busy}>
              {derived ? "Replace experiment" : "Create experiment"}
            </button>
          </form>
        ) : (
          <p className="small">
            The lab requires an unmodified baseline using the supported Base2
            profile. Import the original or its complete replay file.
          </p>
        )}
        <details className="lab-identity">
          <summary>Original and experiment identities</summary>
          <dl className="facts">
            <dt>Original SHA-256</dt>
            <dd className="mono">{baseline.parsed.digest}</dd>
            <dt>Experiment SHA-256</dt>
            <dd className="mono">
              {derived?.parsed.digest ?? "No experiment created"}
            </dd>
          </dl>
        </details>
        <p className="small muted">
          For a valid signature with insufficient stake, use the separately
          signed <strong>Insufficient stake</strong> example. Editing a signer
          bitmap does not preserve its signature.
        </p>
      </div>
    </details>
  );
}
function checks(report: Comparison["baseline"]) {
  const rows = new Map<string, { label: string; check: CheckResult }>();
  for (const check of report.checks)
    rows.set(JSON.stringify(["bundle", check.stage]), {
      label: `Bundle · ${words(check.stage)}`,
      check,
    });
  for (const certificate of report.certificates)
    for (const check of certificate.checks)
      rows.set(
        JSON.stringify([
          "certificate",
          certificate.certificate_id,
          check.stage,
        ]),
        {
          label: `${certificate.certificate_id} · ${words(check.stage)}`,
          check,
        },
      );
  return rows;
}
export function ComparisonView({ reports }: { reports: Comparison }) {
  const [changedOnly, setChangedOnly] = useState(true);
  const before = checks(reports.baseline),
    after = checks(reports.derived);
  const rows = [...new Set([...before.keys(), ...after.keys()])]
    .map((key) => ({ key, before: before.get(key), after: after.get(key) }))
    .filter(
      (row) =>
        !changedOnly ||
        JSON.stringify(row.before?.check) !== JSON.stringify(row.after?.check),
    );
  const cell = (check: CheckResult | undefined) => (
    <>
      {check ? (
        <>
          <strong className="comparison-status" data-status={check.status}>
            {words(check.status)}
          </strong>
          <code>{check.diagnostic.code}</code>
          <span>{check.diagnostic.message}</span>
        </>
      ) : (
        "Not emitted; an earlier prerequisite stopped this trace."
      )}
    </>
  );
  return (
    <section className="comparison" aria-label="Fresh comparison">
      <div className="section-heading">
        <h2>What changed?</h2>
        <label className="inline-label">
          <input
            type="checkbox"
            checked={changedOnly}
            onChange={(e) => setChangedOnly(e.target.checked)}
          />{" "}
          Only changed checks
        </label>
      </div>
      <p className="small muted">
        Both reports were recomputed from their exact input bytes using the same
        checkpoint selection.
      </p>
      <div className="comparison-outcomes">
        <div>
          <span>Original</span>
          <strong
            className="outcome-badge"
            data-status={reports.baseline.conclusion}
          >
            {reports.baseline.conclusion}
          </strong>
          <span>{words(reports.baseline.evidence_scope)}</span>
        </div>
        <span aria-hidden="true">→</span>
        <div>
          <span>Experiment</span>
          <strong
            className="outcome-badge"
            data-status={reports.derived.conclusion}
          >
            {reports.derived.conclusion}
          </strong>
          <span>{words(reports.derived.evidence_scope)}</span>
        </div>
      </div>
      <div
        className="table-scroll"
        role="region"
        tabIndex={0}
        aria-label="Before and after verification checks"
      >
        <table className="comparison-table">
          <thead>
            <tr>
              <th>Check</th>
              <th>Original</th>
              <th>Experiment</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={row.key}>
                <th scope="row">{row.before?.label ?? row.after?.label}</th>
                <td>{cell(row.before?.check)}</td>
                <td>{cell(row.after?.check)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {rows.length === 0 && (
        <p className="small">
          No check diagnostics changed. Compare the evidence identities and full
          reports for other differences.
        </p>
      )}
    </section>
  );
}
