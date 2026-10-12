import type { CaptureReport } from "../../../packages/contracts/src/index";
import { words } from "./Report";
export function CaptureView({
  report,
  name,
  download,
  clear,
}: {
  report: CaptureReport;
  name: string;
  download: () => void;
  clear: () => void;
}) {
  const titles = {
    certificate_present: "Genesis capture needs context",
    no_certificate: "No certificate in this response",
    method_unavailable: "RPC method unavailable",
    rpc_error: "RPC returned an error",
  };
  return (
    <section className="capture-report" aria-label="RPC capture inspection">
      <div className="result-header" data-status="INCONCLUSIVE">
        <p className="result-label">
          <strong className="outcome-badge" data-status="INCONCLUSIVE">
            INCONCLUSIVE
          </strong>
          <span>capture inspection</span>
        </p>
        <h2>{titles[report.observation]}</h2>
        <p>
          Recorded RPC data · source unverified. This inspection does not verify
          a signature, validator stake, or network finality.
        </p>
      </div>
      <dl className="facts">
        <dt>File</dt>
        <dd>{name}</dd>
        <dt>Response</dt>
        <dd>{words(report.observation)}</dd>
        <dt>Source bytes</dt>
        <dd className="mono">{report.source_bytes}</dd>
        <dt>Source SHA-256</dt>
        <dd className="mono">{report.source_digest}</dd>
        <dt>Adapter</dt>
        <dd className="mono">{report.adapter_id}</dd>
        {report.certificate && (
          <>
            <dt>Certificate slot</dt>
            <dd className="mono">{report.certificate.slot}</dd>
            <dt>Block ID</dt>
            <dd className="mono">{report.certificate.block_id_hex}</dd>
          </>
        )}
      </dl>
      {report.rpc_error_message !== null && (
        <div className="capture-note">
          <strong>Endpoint error {report.rpc_error_code}</strong>
          <p>{report.rpc_error_message}</p>
          <p className="small muted">
            This is a recorded endpoint message, not evidence about the
            cluster's consensus state.
          </p>
        </div>
      )}
      {report.requirements.length > 0 && (
        <section>
          <h3>What is missing?</h3>
          <p className="small muted">
            These fields are absent from this RPC response. Monocle does not
            infer them from today's validator set or copy them from an unrelated
            selected checkpoint.
          </p>
          <ul className="capture-requirements">
            {report.requirements.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </section>
      )}
      <section>
        <h3>Inspection trace</h3>
        <ol className="capture-checks">
          {report.checks.map((c) => (
            <li key={c.stage} data-status={c.status}>
              <div>
                <strong>{words(c.stage)}</strong>
                <span className="check-status">{words(c.status)}</span>
              </div>
              <p>{c.diagnostic.message}</p>
              <code>{c.diagnostic.code}</code>
            </li>
          ))}
        </ol>
      </section>
      {report.certificate && (
        <details className="bytes">
          <summary>Preserved signature and bitmap bytes</summary>
          <p className="small muted">
            Byte extraction only. Array length does not establish point
            validity, signer identity, or sufficient stake.
          </p>
          <dl className="facts">
            <dt>Signature · 192 bytes</dt>
            <dd className="mono">{report.certificate.signature_hex}</dd>
            <dt>Bitmap · {report.certificate.bitmap_hex.length / 2} bytes</dt>
            <dd className="mono">{report.certificate.bitmap_hex}</dd>
          </dl>
        </details>
      )}
      <details className="assumptions">
        <summary>Scope and assumptions</summary>
        <ul>
          {report.assumptions.map((a) => (
            <li key={a}>{a}</li>
          ))}
        </ul>
      </details>
      <div className="capture-actions">
        <button onClick={download}>Download original response</button>
        <button className="text-button" onClick={clear}>
          Clear capture
        </button>
      </div>
      <p className="small muted">
        The downloaded JSON is the exact source file. Import it again to repeat
        inspection. It is not a proof bundle or a saved verification result.
      </p>
    </section>
  );
}
