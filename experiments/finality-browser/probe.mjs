const require = (condition, message) => { if (!condition) throw new Error(message); };
try {
  const response = await fetch('/probe.wasm');
  require(response.ok, 'WASM load failed');
  const module = await WebAssembly.compile(await response.arrayBuffer());
  require(WebAssembly.Module.imports(module).length === 0, 'Unexpected WASM imports');
  const { exports } = await WebAssembly.instantiate(module, {});
  const length = exports.monocle_finality_run();
  require(length > 0 && length < 1024 * 1024, 'Probe output size');
  const bytes = Uint8Array.from({ length }, (_, index) => exports.monocle_finality_byte(index));
  require(exports.monocle_finality_byte(length) === 256, 'Out-of-range read must fail');
  const result = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
  require(result.cases.length === 35, 'Corpus size mismatch');
  for (const { name, report } of result.cases) {
    require(report.policy_id === result.policy_id, 'Policy mismatch');
    if (report.conclusion !== 'VERIFIED') {
      require(report.evidence_scope === 'unestablished' && report.finality_path === null && report.supporting_certificate_ids.length === 0, `Incomplete evidence promoted: ${name}`);
    } else {
      require(report.certificates.every(cert => cert.conclusion === 'VERIFIED'), 'Failed extra certificate was ignored');
      if (report.evidence_scope === 'block_finality') {
        require(['fast', 'slow'].includes(report.finality_path), 'Missing ordinary finality path');
        require(report.claim.block_id_hex !== null, 'Missing target block');
        require(report.checks.find(check => check.stage === 'finality').status === 'PASS', 'Composition must pass');
      }
      if (report.evidence_scope === 'genesis_migration') {
        require(report.finality_path === null, 'Genesis must not be ordinary finality');
      }
    }
  }
  const report = { result, checks: ['35 real end-to-end evidence cases', 'unestablished scope on failed/incomplete claims', 'typed fast/slow/migration distinctions', 'all supplied certificates checked', 'zero WASM imports'], runtime: navigator.userAgent };
  document.getElementById('result').textContent = JSON.stringify(report);
  document.getElementById('summary').textContent = 'PASS: direct finality and migration-anchor corpus; native comparison performed by runner.';
  document.body.dataset.state = 'passed';
} catch (error) {
  document.body.dataset.state = 'failed';
  document.getElementById('summary').textContent = 'FAIL: C4 evidence checks';
  document.getElementById('result').textContent = String(error?.stack ?? error);
}
await fetch('/__result', {
  method: 'POST', headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ state: document.body.dataset.state, report: document.getElementById('result').textContent }),
});
