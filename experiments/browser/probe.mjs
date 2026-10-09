const names = [
  'valid_aggregate', 'changed_block_id', 'changed_shred_version',
  'missing_signer', 'invalid_pop', 'invalid_signature',
];
try {
  const response = await fetch('/probe.wasm');
  if (!response.ok) throw new Error(`WASM load failed: ${response.status}`);
  const module = await WebAssembly.compile(await response.arrayBuffer());
  if (WebAssembly.Module.imports(module).length !== 0) throw new Error('Unexpected runtime imports');
  const instance = await WebAssembly.instantiate(module, {});
  const count = instance.exports.monocle_case_count();
  if (count !== names.length) throw new Error('Probe case count mismatch');
  const cases = names.map((name, index) => ({ name, result: instance.exports.monocle_case_result(index) }));
  if (cases.some((item, index) => item.result !== (index === 0 ? 1 : 0))) {
    throw new Error(`Unexpected verification results: ${JSON.stringify(cases)}`);
  }
  if (instance.exports.monocle_case_result(999) !== 2) throw new Error('Unknown case must fail as a probe error');
  const report = { cases, runtime: navigator.userAgent, wasm_imports: 0 };
  document.getElementById('result').textContent = JSON.stringify(report, null, 2);
  document.getElementById('summary').textContent = 'PASS: genuine aggregate accepted; all five negative cases rejected in browser WASM.';
  document.body.dataset.state = 'passed';
} catch (error) {
  document.body.dataset.state = 'failed';
  document.getElementById('summary').textContent = 'FAIL: compatibility experiment did not pass.';
  document.getElementById('result').textContent = String(error?.stack ?? error);
}
