const require = (condition, message) => { if (!condition) throw new Error(message); };
try {
  const response = await fetch('/probe.wasm');
  require(response.ok, 'WASM load failed');
  const module = await WebAssembly.compile(await response.arrayBuffer());
  require(WebAssembly.Module.imports(module).length === 0, 'Unexpected WASM imports');
  const { exports } = await WebAssembly.instantiate(module, {});
  const length = exports.monocle_certificates_run();
  require(length > 0 && length < 1024 * 1024, 'Probe output size');
  const bytes = Uint8Array.from({ length }, (_, index) => exports.monocle_certificates_byte(index));
  require(exports.monocle_certificates_byte(length) === 256, 'Out-of-range output read must be an error');
  const result = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
  require(result.cases.length === 32, 'Corpus size mismatch');
  for (const { name, report } of result.cases) {
    require(report.evidence_scope === 'individual_certificates', 'Wrong evidence scope');
    const finality = report.checks.find(check => check.stage === 'finality');
    require(finality.status === 'NOT_CHECKED' && finality.diagnostic.code === 'FINALITY_NOT_EVALUATED', 'Finality must remain unevaluated');
    if (name.includes('under_quorum')) {
      const checks = report.certificates[0].checks;
      require(checks.find(check => check.stage === 'signature').status === 'PASS', 'Under-quorum fixture must have a valid signature');
      require(checks.find(check => check.stage === 'quorum').status === 'FAIL', 'Under-quorum stake must fail');
    }
    if (name.endsWith('claim_deferred')) {
      require(report.conclusion === 'INCONCLUSIVE' && report.certificates[0].conclusion === 'VERIFIED', 'Certificate success must not establish finality');
    }
  }
  const report = { result, checks: ['32 real Rust certificate cases', 'under-quorum signatures pass', 'finality claims remain inconclusive', 'zero WASM imports'], runtime: navigator.userAgent };
  document.getElementById('result').textContent = JSON.stringify(report);
  document.getElementById('summary').textContent = 'PASS: signed certificate corpus; native comparison performed by the runner.';
  document.body.dataset.state = 'passed';
} catch (error) {
  document.body.dataset.state = 'failed';
  document.getElementById('summary').textContent = 'FAIL: C3 certificate checks';
  document.getElementById('result').textContent = String(error?.stack ?? error);
}
await fetch('/__result', {
  method: 'POST', headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ state: document.body.dataset.state, report: document.getElementById('result').textContent }),
});
