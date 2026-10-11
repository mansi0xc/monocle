import { readFile } from "node:fs/promises";
import { execFileSync } from "node:child_process";
import assert from "node:assert/strict";
import { loadVerifier } from "../apps/web/src/wasm.ts";
const bytes = await readFile(
  "target/wasm32-unknown-unknown/release/monocle_wasm.wasm",
);
const corpus = JSON.parse(
  await readFile("fixtures/finality/corpus.json", "utf8"),
);
const native = JSON.parse(
  execFileSync(
    "cargo",
    ["run", "--locked", "--quiet", "-p", "monocle-finality-compat"],
    { encoding: "utf8", maxBuffer: 4 * 1024 * 1024 },
  ),
);
const encoder = new TextEncoder();
for (const c of corpus.cases) {
  const call = await loadVerifier(bytes);
  const checkpoint = call(2, encoder.encode(c.checkpoint_json));
  if (c.select_checkpoint) assert.equal(call(3), checkpoint.digest);
  const bundle = JSON.parse(c.bundle_json);
  const result = call(
    5,
    encoder.encode(
      JSON.stringify({
        schema_version: 1,
        run_id: c.name,
        profile_id: bundle.profile_id ?? "agave-98535e50-base2-v1",
        bundle_json: c.bundle_json,
        selected_checkpoint: c.select_checkpoint ? checkpoint.digest : null,
      }),
    ),
  );
  assert.deepEqual(
    result,
    native.cases.find((n) => n.name === c.name).report,
    c.name,
  );
}
const { instance } = await WebAssembly.instantiate(bytes, {});
const api = instance.exports;
assert.equal(api.monocle_execute(1), 0, "execute without allocation fails");
assert.equal(
  api.monocle_input_alloc(16 * 1024 * 1024 + 1),
  0,
  "oversized allocation fails",
);
assert.equal(
  api.monocle_execute(5),
  0,
  "failed allocation cannot reuse prior input",
);
const call = await loadVerifier(bytes);
assert.throws(() => call(1, new Uint8Array([255])), /MALFORMED_JSON/);
assert.throws(
  () => call(1, encoder.encode('{"schema_version":1,"schema_version":1}')),
  /DUPLICATE_FIELD/,
);
const base = JSON.parse(corpus.cases[0].bundle_json);
base.claim.slot = "18446744073709551615";
assert.equal(
  call(1, encoder.encode(JSON.stringify(base))).document.claim.slot,
  base.claim.slot,
);
// Exercise successive memory growth and then a small request using fresh views.
assert.throws(
  () => call(2, encoder.encode(" ".repeat(4 * 1024 * 1024))),
  /MALFORMED_JSON/,
);
assert.equal(
  call(1, encoder.encode(JSON.stringify(base))).document.claim.slot,
  base.claim.slot,
);
console.log(
  `Bridge: ${corpus.cases.length} full WASM/native report comparisons; ABI bounds, UTF-8, duplicates, u64, and memory growth passed.`,
);

const replayCases = JSON.parse(
  execFileSync(
    "cargo",
    [
      "run",
      "--locked",
      "--quiet",
      "-p",
      "monocle-wasm",
      "--example",
      "replay_probe",
    ],
    { encoding: "utf8", maxBuffer: 4 * 1024 * 1024 },
  ),
);
for (const c of replayCases) {
  const bridge = await loadVerifier(bytes);
  if (c.response.ok)
    assert.deepEqual(
      bridge(c.operation, encoder.encode(c.input_json)),
      c.response.value,
    );
  else
    assert.throws(() => bridge(c.operation, encoder.encode(c.input_json)), {
      message: `${c.response.error.code} · ${c.response.error.path}: ${c.response.error.message}`,
    });
}
console.log(
  `Replay bridge: ${replayCases.length} native/WASM mutation, archive, lineage and boundary comparisons passed.`,
);

// Raw RPC integers enter Rust unchanged; JS never parses transport slots for the product.
const captureSource = await readFile(
  "fixtures/captures/testnet-genesis-rpc.json",
  "utf8",
);
const captureCases = [
  captureSource,
  captureSource.replace('"slot":444625255', '"slot":18446744073709551615'),
  '{"jsonrpc":"2.0","id":1,"result":null}',
  '{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}',
  '{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"Temporarily unavailable"}}',
  '{"jsonrpc":"2.0","jsonrpc":"2.0"}',
  captureSource.replace('"id":1', '"id":1,"trusted":true'),
  " ".repeat(64 * 1024 + 1),
];
for (const input of captureCases) {
  let expected;
  try {
    expected = JSON.parse(
      execFileSync(
        "cargo",
        [
          "run",
          "--locked",
          "--quiet",
          "-p",
          "monocle-core",
          "--bin",
          "monocle-capture",
          "--",
          "-",
        ],
        {
          input,
          encoding: "utf8",
          maxBuffer: 1024 * 1024,
          stdio: ["pipe", "pipe", "pipe"],
        },
      ),
    );
  } catch (error) {
    assert.equal(error.status, 2);
    const diagnostic = JSON.parse(error.stderr);
    assert.throws(() => call(9, encoder.encode(input)), {
      message: `${diagnostic.code} · ${diagnostic.path}: ${diagnostic.message}`,
    });
    continue;
  }
  assert.deepEqual(call(9, encoder.encode(input)), expected);
  assert.equal(expected.conclusion, "INCONCLUSIVE");
}
console.log(
  `Capture bridge: ${captureCases.length} complete native/WASM RPC, integer and rejection comparisons passed.`,
);
