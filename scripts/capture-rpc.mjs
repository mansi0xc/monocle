#!/usr/bin/env node
// Record raw Solana JSON-RPC responses for an offline, reproducible checkpoint build.
// Network access lives only here; the verifier and the builder never fetch anything.
//
//   node scripts/capture-rpc.mjs validators <rpc-url> <out-dir>
//   node scripts/capture-rpc.mjs genesis-cert <rpc-url> <out-dir>
//
// Each response is written byte-for-byte as received, and manifest.json records the
// request, HTTP status and SHA-256 of every file. Nothing here is trusted later: the
// builder re-checks every digest and the caller still chooses trust explicitly.
import { createHash } from "node:crypto";
import { mkdir, writeFile, readdir } from "node:fs/promises";
import { join } from "node:path";

// Agave v4.3.0 `agave_feature_set::alpenglow::id()`; it differs from 4.2.x.
const ALPENGLOW_FEATURE = "A1pengvuM6JEcyNuTnMqepBKhwHE3N6PmUrdATGawhJS";
const VOTE_STATE_V4_SIZE = 3762;
const BATCH = 100; // getMultipleAccounts limit
const finalized = { commitment: "finalized" };

const [mode, endpoint, outDir] = process.argv.slice(2);
if (!["validators", "genesis-cert"].includes(mode) || !endpoint || !outDir) {
  console.error(
    "Usage: node scripts/capture-rpc.mjs <validators|genesis-cert> <rpc-url> <out-dir>",
  );
  process.exit(2);
}
await mkdir(outDir, { recursive: true });
if ((await readdir(outDir)).length) {
  console.error(`${outDir} is not empty; captures are never overwritten.`);
  process.exit(2);
}

const files = [];
let id = 0;
async function call(name, method, params) {
  const body = JSON.stringify({ jsonrpc: "2.0", id: ++id, method, params });
  for (let attempt = 1; ; attempt++) {
    const response = await fetch(endpoint, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body,
    });
    const bytes = new Uint8Array(await response.arrayBuffer());
    if ((response.status === 429 || response.status >= 500) && attempt < 6) {
      await new Promise((done) => setTimeout(done, 500 * 2 ** attempt));
      continue;
    }
    if (response.status !== 200)
      throw new Error(`${method}: HTTP ${response.status}`);
    const file = `${String(files.length).padStart(3, "0")}-${name}.json`;
    await writeFile(join(outDir, file), bytes);
    files.push({
      file,
      method,
      request: body,
      http_status: response.status,
      sha256: createHash("sha256").update(bytes).digest("hex"),
      bytes: bytes.byteLength,
      received_at: new Date().toISOString(),
    });
    // Parsed only to plan later requests; the saved bytes are what the builder reads.
    const json = JSON.parse(new TextDecoder().decode(bytes));
    if (json.error) throw new Error(`${method}: ${JSON.stringify(json.error)}`);
    await new Promise((done) => setTimeout(done, 120));
    return json.result;
  }
}

const started = await call("epoch-info", "getEpochInfo", [finalized]);
if (mode === "validators") {
  await call("epoch-schedule", "getEpochSchedule", []);
  await call("genesis-hash", "getGenesisHash", []);
  await call("cluster-nodes", "getClusterNodes", []);
  await call("rent-vote-v4", "getMinimumBalanceForRentExemption", [
    VOTE_STATE_V4_SIZE,
    finalized,
  ]);
  await call("alpenglow-feature", "getAccountInfo", [
    ALPENGLOW_FEATURE,
    { ...finalized, encoding: "base64" },
  ]);
  const votes = await call("vote-accounts", "getVoteAccounts", [finalized]);
  const keys = [...votes.current, ...votes.delinquent].map((v) => v.votePubkey);
  for (let i = 0; i < keys.length; i += BATCH)
    await call(
      `vote-state-${String(i / BATCH).padStart(3, "0")}`,
      "getMultipleAccounts",
      [keys.slice(i, i + BATCH), { ...finalized, encoding: "base64" }],
    );
} else {
  await call("genesis-cert", "getAgGenesisCert", []);
}
const ended = await call("epoch-info-end", "getEpochInfo", [finalized]);
if (ended.epoch !== started.epoch)
  console.error(
    `Warning: epoch changed during capture (${started.epoch} -> ${ended.epoch}); capture again.`,
  );

await writeFile(
  join(outDir, "manifest.json"),
  JSON.stringify(
    {
      schema_version: 1,
      format: "monocle-rpc-capture",
      mode,
      endpoint,
      epoch: started.epoch,
      first_slot: started.absoluteSlot,
      last_slot: ended.absoluteSlot,
      files,
    },
    null,
    2,
  ) + "\n",
);
console.log(
  `Captured ${files.length} responses from epoch ${started.epoch} (slots ${started.absoluteSlot}-${ended.absoluteSlot}) into ${outDir}.`,
);
