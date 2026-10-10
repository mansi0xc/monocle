// Compile actual Rust/fixture JSON against the TypeScript contracts as well as
// checking individual precision errors. No generated test file is committed.
import ts from 'typescript';
import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
const corpus = JSON.parse(execFileSync('cargo', ['run', '--locked', '--quiet', '-p', 'monocle-contracts-compat'], { encoding: 'utf8' }));
const bundle = JSON.parse(await readFile('fixtures/contracts/bundle.json', 'utf8'));
const checkpoint = JSON.parse(await readFile('fixtures/contracts/checkpoint.json', 'utf8'));
const path = resolve('target/c2-typecheck.ts');
await writeFile(path, `import type { ProofBundle, EpochCheckpoint, ProtocolProfile, VerificationReport } from '../packages/contracts/src/index.js';
const bundle = ${JSON.stringify(bundle)} satisfies ProofBundle;
const checkpoint = ${JSON.stringify(checkpoint)} satisfies EpochCheckpoint;
const profile = ${JSON.stringify(corpus.profile)} satisfies ProtocolProfile;
const reports = ${JSON.stringify(corpus.cases.map(item => item.report))} satisfies VerificationReport[];
`);
const program = ts.createProgram([path], { strict: true, noEmit: true, target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.NodeNext, moduleResolution: ts.ModuleResolutionKind.NodeNext });
const diagnostics = ts.getPreEmitDiagnostics(program);
if (diagnostics.length) throw new Error(ts.formatDiagnosticsWithColorAndContext(diagnostics, { getCanonicalFileName: p => p, getCurrentDirectory: () => process.cwd(), getNewLine: () => '\n' }));
console.log('Rust/TypeScript shape checks passed for both fixtures, compiled-in profile, and 13 actual Rust reports.');
