#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Apple's ar/ranlib can produce an archive without a usable WASM symbol index.
# Use the LLVM archiver paired with this project's pinned Rust toolchain.
toolchain_sysroot="$(rustc --print sysroot)"
toolchain_host="$(rustc -vV | sed -n 's/^host: //p')"
llvm_ar="$toolchain_sysroot/lib/rustlib/$toolchain_host/bin/llvm-ar"
if [[ ! -x "$llvm_ar" ]]; then
  echo 'Missing LLVM tools; run: rustup component add llvm-tools' >&2
  exit 1
fi
export AR_wasm32_unknown_unknown="$llvm_ar"
export CC_wasm32_unknown_unknown="${CC_wasm32_unknown_unknown:-clang}"
cargo build --locked --workspace --release --lib --target wasm32-unknown-unknown
node --input-type=module - <<'JS'
import { readFile } from 'node:fs/promises';
for (const name of ['monocle_bls_compat', 'monocle_contracts_compat']) {
  const module = await WebAssembly.compile(await readFile(`target/wasm32-unknown-unknown/release/${name}.wasm`));
  const imports = WebAssembly.Module.imports(module);
  if (imports.length) throw new Error(`Unresolved WASM imports: ${JSON.stringify(imports)}`);
  console.log(`${name}: WASM link check passed; no unresolved runtime imports.`);
}
JS
