// Execute the Rust probes in real Chrome and compare their reports with native Rust.
// Completion is explicit: DOM snapshots and virtual-time budgets can race async work.
import { createServer } from 'node:http';
import { readFile, mkdtemp, rm } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn, execFileSync } from 'node:child_process';
import assert from 'node:assert/strict';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const mode = process.argv[2] ?? 'bls';
const modes = {
  bls: ['experiments/browser', 'monocle_bls_compat', ['--bin', 'probe'], 'C1 BLS'],
  contracts: ['experiments/contracts-browser', 'monocle_contracts_compat', ['-p', 'monocle-contracts-compat'], 'C2 contracts'],
  certificates: ['experiments/certificate-browser', 'monocle_certificate_compat', ['-p', 'monocle-certificate-compat'], 'C3 certificates'],
  finality: ['experiments/finality-browser', 'monocle_finality_compat', ['-p', 'monocle-finality-compat'], 'C4 finality'],
};
if (!Object.hasOwn(modes, mode)) throw new Error('Expected no argument, contracts, certificates, or finality');
const [browserAssets, wasmName, nativeArgs, experiment] = modes[mode];
const candidates = [process.env.CHROME_BIN, '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', '/usr/bin/chromium', '/usr/bin/chromium-browser', '/usr/bin/google-chrome'].filter(Boolean);
const chrome = candidates.find(existsSync);
if (!chrome) throw new Error('Chromium/Chrome not found. Set CHROME_BIN to the browser executable.');
const routes = new Map([
  ['/', [`${browserAssets}/index.html`, 'text/html']],
  ['/probe.mjs', [`${browserAssets}/probe.mjs`, 'text/javascript']],
  ['/probe.wasm', [`target/wasm32-unknown-unknown/release/${wasmName}.wasm`, 'application/wasm']],
]);
if (mode === 'contracts') {
  routes.set('/bundle.json', ['fixtures/contracts/bundle.json', 'application/json']);
  routes.set('/contracts.js', ['target/contracts-js/index.js', 'text/javascript']);
  // index.js re-exports the Rust-generated values by this relative path.
  routes.set('/generated.js', ['target/contracts-js/generated.js', 'text/javascript']);
}
let complete;
let fail;
// Handlers are installed before navigation; immediately attach a rejection handler.
const completion = new Promise((resolve, reject) => { complete = resolve; fail = reject; });
completion.catch(() => {});
const server = createServer(async (req, res) => {
  if (req.method === 'POST' && req.url === '/__result') {
    try {
      const chunks = []; let size = 0;
      for await (const chunk of req) {
        size += chunk.length;
        if (size > 1024 * 1024) throw new Error('Browser report exceeded 1 MiB');
        chunks.push(chunk);
      }
      const result = JSON.parse(Buffer.concat(chunks).toString('utf8'));
      res.writeHead(200).end('received');
      complete(result);
    } catch (error) { res.writeHead(400).end(); fail(error); }
    return;
  }
  const entry = req.method === 'GET' && routes.get(req.url);
  if (!entry) { res.writeHead(404).end(); return; }
  try {
    const data = await readFile(join(root, entry[0]));
    res.writeHead(200, { 'Content-Type': entry[1], 'Cache-Control': 'no-store' }).end(data);
  } catch {
    res.writeHead(500).end('Build the WASM and TypeScript assets first.');
  }
});
const profile = await mkdtemp(join(tmpdir(), 'monocle-browser-'));
let child; let timer; let closed;
try {
  await new Promise((done, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', done); });
  const url = `http://127.0.0.1:${server.address().port}/`;
  const native = JSON.parse(execFileSync('cargo', ['run', '--locked', '--quiet', ...nativeArgs], { cwd: root, encoding: 'utf8', timeout: 60_000 }));
  let stderr = '';
  child = spawn(chrome, [
    '--headless', '--disable-gpu', '--no-first-run', '--no-default-browser-check',
    '--disable-background-networking', '--disable-sync', '--disable-extensions',
    '--disable-component-update', '--use-mock-keychain', '--password-store=basic',
    `--user-data-dir=${profile}`, url,
  ], { stdio: ['ignore', 'ignore', 'pipe'] });
  closed = new Promise(done => child.once('close', done));
  child.stderr.on('data', data => { stderr = (stderr + data).slice(-4000); });
  child.once('error', fail);
  child.once('close', code => fail(new Error(`Browser exited before completion (${code}): ${stderr}`)));
  timer = setTimeout(() => fail(new Error(`Browser did not report completion within 60 seconds: ${stderr}`)), 60_000);
  const delivered = await completion;
  clearTimeout(timer);
  if (delivered.state !== 'passed') throw new Error(`Browser did not pass: ${delivered.report}`);
  const browser = JSON.parse(delivered.report);
  if (mode !== 'bls') {
    assert.deepEqual(browser.result, native);
    console.log(JSON.stringify({ experiment, native_browser_match: true, browser_checks: browser.checks, cases: native.cases.map(item => ({ name: item.name, conclusion: item.report.conclusion })), runtime: browser.runtime }, null, 2));
  } else {
    assert.deepEqual(browser.cases, native.cases);
    console.log(JSON.stringify({ profile_id: native.profile_id, agave_revision: native.agave_revision, native_browser_match: true, ...browser }, null, 2));
  }
} finally {
  clearTimeout(timer);
  if (child && child.exitCode === null && child.signalCode === null) child.kill('SIGKILL');
  if (closed) await closed;
  server.closeAllConnections();
  await new Promise(resolve => server.close(resolve));
  await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
}
