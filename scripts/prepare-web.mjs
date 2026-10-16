import { mkdir, copyFile, readFile, writeFile } from "node:fs/promises";
const out = new URL("../apps/web/generated/", import.meta.url);
await mkdir(out, { recursive: true });
await copyFile(
  new URL(
    "../target/wasm32-unknown-unknown/release/monocle_wasm.wasm",
    import.meta.url,
  ),
  new URL("monocle_wasm.wasm", out),
);
const corpus = JSON.parse(
  await readFile(
    new URL("../fixtures/finality/corpus.json", import.meta.url),
    "utf8",
  ),
);
const choices = [
  [
    "fast_finality",
    "Fast finality",
    "One aggregate certificate meets the fast-path threshold.",
  ],
  [
    "slow_finality",
    "Slow finality",
    "A notarization and finalization establish the paired path.",
  ],
  [
    "genesis_anchor",
    "Genesis anchor",
    "Migration-anchor evidence with a deliberately separate scope.",
  ],
  [
    "notar_only",
    "Missing finalization",
    "A valid notarization alone leaves the block claim incomplete.",
  ],
  [
    "changed_signature",
    "Changed signer set",
    "The bitmap names different signers, so the aggregate no longer verifies.",
  ],
  [
    "fast_under_quorum",
    "Insufficient stake",
    "A genuine signature with too little signer weight.",
  ],
];
const examples = choices.map(([id, title, description]) => {
  const c = corpus.cases.find((c) => c.name === id);
  if (!c) throw new Error(`Missing example ${id}`);
  return {
    id,
    title,
    description,
    bundle_json: c.bundle_json,
    checkpoint_json: c.checkpoint_json,
  };
});
await writeFile(new URL("examples.json", out), JSON.stringify(examples));
const capture = await readFile(
  new URL("../fixtures/captures/testnet-genesis-rpc.json", import.meta.url),
  "utf8",
);
await writeFile(
  new URL("capture.json", out),
  JSON.stringify({ source_json: capture }),
);
console.log(
  "Prepared WASM, six synthetic examples and one recorded RPC response. No saved results are shipped.",
);
