// Run the compiled Rust module in an actual Chromium browser, then compare it
// with the native executable. No npm runtime or browser automation dependency.
import { createServer } from 'node:http';
import { readFile, mkdtemp, rm } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn, execFileSync } from 'node:child_process';
import assert from 'node:assert/strict';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const candidates = [
  process.env.CHROME_BIN,
  '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
  '/usr/bin/chromium', '/usr/bin/chromium-browser', '/usr/bin/google-chrome',
].filter(Boolean);
const chrome = candidates.find(existsSync);
if (!chrome) throw new Error('Chromium/Chrome not found. Set CHROME_BIN to the browser executable.');
const routes = new Map([
  ['/', ['experiments/browser/index.html', 'text/html']],
  ['/probe.mjs', ['experiments/browser/probe.mjs', 'text/javascript']],
  ['/probe.wasm', ['target/wasm32-unknown-unknown/release/monocle_bls_compat.wasm', 'application/wasm']],
]);
const server = createServer(async (req, res) => {
  const entry = routes.get(req.url);
  if (!entry) { res.writeHead(404).end(); return; }
  try {
    const data = await readFile(join(root, entry[0]));
    res.writeHead(200, { 'Content-Type': entry[1], 'Cache-Control': 'no-store' }).end(data);
  } catch {
    res.writeHead(500).end('Build the WASM library first.');
  }
});
const profile = await mkdtemp(join(tmpdir(), 'monocle-browser-'));
let child;
try {
  await new Promise((done, fail) => { server.once('error', fail); server.listen(0, '127.0.0.1', done); });
  const url = `http://127.0.0.1:${server.address().port}/`;
  const native = JSON.parse(execFileSync('cargo', ['run', '--locked', '--quiet', '--bin', 'probe'], { cwd: root, encoding: 'utf8' }));
  const dom = await new Promise((done, fail) => {
    child = spawn(chrome, [
      '--headless', '--disable-gpu', '--no-first-run', '--no-default-browser-check',
      '--disable-background-networking', '--disable-sync', '--disable-extensions',
      '--disable-component-update', '--use-mock-keychain', '--password-store=basic',
      `--user-data-dir=${profile}`, '--dump-dom', '--virtual-time-budget=15000', url,
    ], { stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = ''; let stderr = '';
    const timer = setTimeout(() => { child.kill('SIGKILL'); fail(new Error('Browser probe timed out after 60 seconds.')); }, 60_000);
    child.stdout.on('data', data => { stdout += data; });
    child.stderr.on('data', data => { stderr += data; });
    child.once('error', error => { clearTimeout(timer); fail(error); });
    child.once('close', code => {
      clearTimeout(timer);
      if (code === 0) done(stdout);
      else fail(new Error(`Browser exited ${code}: ${stderr.slice(-4000)}`));
    });
  });
  if (!dom.includes('data-state="passed"')) throw new Error(`Browser did not pass:\n${dom}`);
  const match = dom.match(/<pre id="result">([\s\S]*?)<\/pre>/);
  if (!match) throw new Error('Browser report missing.');
  const browser = JSON.parse(match[1]);
  assert.deepEqual(browser.cases, native.cases);
  console.log(JSON.stringify({ profile_id: native.profile_id, agave_revision: native.agave_revision, native_browser_match: true, ...browser }, null, 2));
} finally {
  if (child && child.exitCode === null) child.kill();
  await new Promise(resolve => server.close(resolve));
  await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
}
