import { useState } from "react";
import type {
  CertificateReport,
  CheckResult,
  EpochCheckpoint,
  EvidenceVerificationReport,
  ProofBundle,
} from "../../../packages/contracts/src/index";

export const words = (text: string) => text.replaceAll("_", " ");
export function Trace({ checks }: { checks: CheckResult[] }) {
  return (
    <ol className="trace">
      {checks.map((check, i) => (
        <li key={i} data-status={check.status}>
          <span className="check-symbol" aria-hidden="true">
            {check.status === "PASS"
              ? "✓"
              : check.status === "FAIL"
                ? "×"
                : "—"}
          </span>
          <div>
            <div className="check-title">
              <strong>{words(check.stage)}</strong>
              <span className="check-status">
                {words(check.status.toLowerCase())}
              </span>
            </div>
            <p>{check.diagnostic.message}</p>
            <details className="diagnostic">
              <summary>Diagnostic</summary>
              <code>{check.diagnostic.code}</code>
              <span className="mono">{check.diagnostic.path}</span>
            </details>
          </div>
        </li>
      ))}
    </ol>
  );
}
function Signers({
  certificate,
  checkpoint,
}: {
  certificate: CertificateReport;
  checkpoint: EpochCheckpoint | null;
}) {
  const [page, setPage] = useState(0);
  const [all, setAll] = useState(false);
  if (!checkpoint) return <p>Signer weights require a selected checkpoint.</p>;
  const ranks = new Set(certificate.signer_ranks);
  const validators = checkpoint.validators.filter(
    (v) => all || ranks.has(v.rank),
  );
  const pages = Math.max(1, Math.ceil(validators.length / 50));
  const current = Math.min(page, pages - 1);
  return (
    <div className="signers">
      <div className="section-heading">
        <h4>Signer weights</h4>
        <label className="inline-label">
          <input
            type="checkbox"
            checked={all}
            onChange={(e) => {
              setAll(e.target.checked);
              setPage(0);
            }}
          />{" "}
          Show all validators
        </label>
      </div>
      <p className="small muted">
        Weights come from the selected checkpoint. Inclusion in this table does
        not itself establish signature or key validity.
      </p>
      <div
        className="table-scroll"
        tabIndex={0}
        role="region"
        aria-label={`Signer weights for ${certificate.certificate_id}`}
      >
        <table>
          <thead>
            <tr>
              <th>Rank</th>
              <th>Validator identity</th>
              <th>Selected</th>
              <th className="numeric">Stake</th>
              <th>PoP present</th>
            </tr>
          </thead>
          <tbody>
            {validators.slice(current * 50, current * 50 + 50).map((v) => (
              <tr key={v.rank}>
                <td>{v.rank}</td>
                <td>
                  <details>
                    <summary className="mono">
                      {v.identity_hex.slice(0, 12)}…
                    </summary>
                    <dl className="key-details">
                      <dt>Identity</dt>
                      <dd className="mono">{v.identity_hex}</dd>
                      <dt>BLS public key</dt>
                      <dd className="mono">{v.bls_pubkey_hex}</dd>
                    </dl>
                  </details>
                </td>
                <td>{ranks.has(v.rank) ? "Yes" : "No"}</td>
                <td className="mono numeric">{v.stake}</td>
                <td>{v.proof_of_possession_hex ? "Yes" : "No"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {validators.length === 0 && <p>No signer ranks were established.</p>}
      {pages > 1 && (
        <div className="pagination">
          <button disabled={current === 0} onClick={() => setPage(current - 1)}>
            Previous
          </button>
          <span>
            Page {current + 1} of {pages}
          </span>
          <button
            disabled={current + 1 === pages}
            onClick={() => setPage(current + 1)}
          >
            Next
          </button>
        </div>
      )}
    </div>
  );
}
function Certificate({
  cert,
  bundle,
  checkpoint,
  first,
}: {
  cert: CertificateReport;
  bundle: ProofBundle;
  checkpoint: EpochCheckpoint | null;
  first: boolean;
}) {
  const envelope = bundle.certificates.find(
    (c) => c.id === cert.certificate_id,
  );
  return (
    <details className="certificate" data-status={cert.conclusion} open={first}>
      <summary>
        <span>
          <strong>{cert.certificate_id}</strong>
          <span className="muted">
            {cert.kind ? words(cert.kind) : "Kind not established"}
          </span>
        </span>
        <span className="certificate-outcome" data-status={cert.conclusion}>
          {cert.conclusion}
        </span>
      </summary>
      <div className="certificate-body">
        <dl className="facts">
          <dt>Certified slot</dt>
          <dd className="mono">{cert.slot ?? "Not established"}</dd>
          <dt>Block binding</dt>
          <dd className="mono">
            {cert.block_id_hex ??
              "No block ID established for this certificate"}
          </dd>
          <dt>Selected stake</dt>
          <dd className="mono">
            {cert.selected_stake ?? "—"} / {cert.total_stake ?? "—"}
            {cert.threshold_percent !== null && (
              <span className="muted">
                {" "}
                · threshold {cert.threshold_percent}%
              </span>
            )}
          </dd>
        </dl>
        <Trace checks={cert.checks} />
        <Signers certificate={cert} checkpoint={checkpoint} />
        <details className="bytes">
          <summary>Signed bytes and certificate identity</summary>
          <dl className="facts">
            <dt>Certificate SHA-256</dt>
            <dd className="mono">{cert.raw_digest}</dd>
            <dt>Signed payload</dt>
            <dd className="mono">
              {cert.signed_payload_hex ?? "Not established"}
            </dd>
          </dl>
          {envelope && (
            <>
              <p className="small">
                Original certificate bytes · {envelope.raw_hex.length / 2} bytes
                · {envelope.format}
              </p>
              <pre>{envelope.raw_hex}</pre>
            </>
          )}
        </details>
      </div>
    </details>
  );
}
export function reportTitle(report: EvidenceVerificationReport) {
  if (report.conclusion === "REJECTED") return "Evidence rejected";
  if (report.conclusion === "INCONCLUSIVE") return "Verification inconclusive";
  if (report.evidence_scope === "genesis_migration")
    return "Genesis anchor verified";
  if (report.evidence_scope === "individual_certificates")
    return "Certificates verified";
  return `${report.finality_path === "fast" ? "Fast" : "Slow"} finality verified`;
}
export function Report({
  report,
  bundle,
  checkpoint,
}: {
  report: EvidenceVerificationReport;
  bundle: ProofBundle;
  checkpoint: EpochCheckpoint | null;
}) {
  return (
    <>
      <div className="result-header" data-status={report.conclusion}>
        <p className="result-label">
          <strong className="outcome-badge" data-status={report.conclusion}>
            {report.conclusion}
          </strong>{" "}
          <span>
            {report.checkpoint_digest
              ? "under the selected checkpoint"
              : "without a selected trusted checkpoint"}
          </span>
        </p>
        <h2>{reportTitle(report)}</h2>
        <p>
          {report.conclusion === "REJECTED"
            ? "The supplied evidence failed a check. This does not prove that the block is non-final."
            : report.conclusion === "INCONCLUSIVE"
              ? "A required check could not establish the requested claim. Read the trace for missing context or unsupported evidence."
              : report.evidence_scope === "genesis_migration"
                ? "This establishes migration-anchor evidence only. It does not prove ordinary block finality or completed network migration."
                : "The selected evidence satisfies the requested scope, subject to the checkpoint and protocol assumptions below."}
        </p>
      </div>
      <dl className="facts result-facts">
        <dt>Established scope</dt>
        <dd>{words(report.evidence_scope)}</dd>
        <dt>Requested slot</dt>
        <dd className="mono">{bundle.claim.slot}</dd>
        <dt>Path</dt>
        <dd>
          {report.finality_path
            ? words(report.finality_path)
            : "Not established"}
        </dd>
        <dt>Witness certificates</dt>
        <dd>
          {report.supporting_certificate_ids.join(", ") || "None established"}
        </dd>
      </dl>
      <section className="report-section">
        <h3>Verification trace</h3>
        <Trace checks={report.checks} />
      </section>
      {report.certificates.length > 0 && (
        <section className="report-section">
          <div className="section-heading">
            <h3>Certificates</h3>
            <span className="muted small">
              {report.certificates.length} evaluated
            </span>
          </div>
          {report.certificates.map((cert, index) => (
            <Certificate
              key={cert.certificate_id}
              cert={cert}
              bundle={bundle}
              checkpoint={checkpoint}
              first={index === 0}
            />
          ))}
        </section>
      )}
      <section className="report-section assumptions">
        <h3>Trust and scope</h3>
        <ul>
          {report.assumptions.map((assumption, i) => (
            <li key={i}>{assumption}</li>
          ))}
        </ul>
        <p>
          Checkpoint authenticity is an external assumption. This inspector does
          not verify execution, ancestry, or authenticated epoch transitions.
        </p>
        <details>
          <summary>Run identity and policy</summary>
          <dl className="facts">
            <dt>Run ID</dt>
            <dd className="mono">{report.run_id}</dd>
            <dt>Bundle digest</dt>
            <dd className="mono">{report.bundle_digest}</dd>
            <dt>Checkpoint digest</dt>
            <dd className="mono">
              {report.checkpoint_digest ?? "None selected"}
            </dd>
            <dt>Profile</dt>
            <dd className="mono">{report.profile_id}</dd>
            <dt>Policy</dt>
            <dd className="mono">{report.policy_id}</dd>
          </dl>
        </details>
      </section>
    </>
  );
}
