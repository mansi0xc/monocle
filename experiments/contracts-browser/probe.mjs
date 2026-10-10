import { createVerifyRequest, encodeVerifyRequest, decimalU64 } from '/contracts.js';
const expected = {
  context_ready: 'VERIFIER_NOT_IMPLEMENTED', missing_checkpoint: 'MISSING_CHECKPOINT',
  untrusted_checkpoint: 'CHECKPOINT_NOT_TRUSTED', duplicate_key: 'DUPLICATE_FIELD',
  numeric_slot: 'INVALID_INTEGER', injected_trust: 'UNKNOWN_FIELD', unknown_version: 'UNSUPPORTED_VERSION',
  unknown_profile: 'UNSUPPORTED_PROFILE', unknown_format: 'UNSUPPORTED_CERTIFICATE',
  wrong_epoch: 'CONTEXT_MISMATCH', wrong_reference: 'CHECKPOINT_MISMATCH',
  malformed_hex: 'INVALID_HEX', malformed_json: 'MALFORMED_JSON',
};
const require = (condition, message) => { if (!condition) throw new Error(message); };
try {
  const response = await fetch('/probe.wasm');
  require(response.ok, 'WASM load failed');
  const module = await WebAssembly.compile(await response.arrayBuffer());
  require(WebAssembly.Module.imports(module).length === 0, 'Unexpected WASM imports');
  const { exports } = await WebAssembly.instantiate(module, {});
  const length = exports.monocle_contracts_run();
  require(length > 0 && length < 1024 * 1024, 'Probe output size');
  const bytes = Uint8Array.from({ length }, (_, index) => exports.monocle_contracts_byte(index));
  require(exports.monocle_contracts_byte(length) === 256, 'Out-of-range output read must be an error');
  const result = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
  require(result.cases.length === Object.keys(expected).length, 'Corpus size mismatch');
  for (const { name, report } of result.cases) {
    require(report.conclusion !== 'VERIFIED', 'Preflight must never verify finality');
    require(report.checks.some(check => check.diagnostic.code === expected[name]), `Unexpected diagnostic: ${name}`);
    require(report.checks.slice(3).every(check => check.status === 'NOT_CHECKED'), 'Unimplemented checks must remain unperformed');
  }
  const raw = new Uint8Array(await (await fetch('/bundle.json')).arrayBuffer());
  const digest = 'sha256:' + Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', raw)), byte => byte.toString(16).padStart(2, '0')).join('');
  require(digest === result.bundle_digest, 'WebCrypto and Rust source digests differ');
  const request = createVerifyRequest('browser-transport', result.profile.id, raw, null);
  const transported = JSON.parse(new TextDecoder().decode(encodeVerifyRequest(request)));
  const restored = new TextEncoder().encode(transported.bundle_json);
  require(raw.length === restored.length && raw.every((byte, index) => byte === restored[index]), 'Browser transport changed source bytes');
  require(JSON.parse(transported.bundle_json).claim.slot === result.slot && result.slot === decimalU64(18446744073709551615n), 'u64 lost precision');
  const report = { result, checks: ['13 Rust preflight cases', 'WebCrypto/Rust SHA-256 equality', 'TypeScript request byte preservation', 'u64 max string transport', 'zero WASM imports'], runtime: navigator.userAgent };
  document.getElementById('result').textContent = JSON.stringify(report);
  document.getElementById('summary').textContent = 'PASS: native-comparable contracts corpus and browser transport checks.';
  document.body.dataset.state = 'passed';
} catch (error) {
  document.body.dataset.state = 'failed';
  document.getElementById('summary').textContent = 'FAIL: C2 boundary checks';
  document.getElementById('result').textContent = String(error?.stack ?? error);
}

// Local test harness completion channel; never sends evidence to an external service.
await fetch('/__result', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ state: document.body.dataset.state, report: document.getElementById('result').textContent }),
});
