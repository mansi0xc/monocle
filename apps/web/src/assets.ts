import wasmUrl from "../generated/monocle_wasm.wasm?url";
let pending: Promise<ArrayBuffer> | null = null;
/** Keep immutable verifier bytes in page memory. Fresh inline workers need no network. */
export function loadVerifierAssets(): Promise<ArrayBuffer> {
  if (!pending) {
    pending = (async () => {
      const controller = new AbortController();
      const timeout = setTimeout(() => controller.abort(), 30_000);
      try {
        const response = await fetch(wasmUrl, { signal: controller.signal });
        if (!response.ok)
          throw new Error(
            "Verifier asset unavailable. Rebuild the app and reload.",
          );
        const bytes = await response.arrayBuffer();
        if (!bytes.byteLength || bytes.byteLength > 4 * 1024 * 1024)
          throw new Error("Invalid verifier asset size. Rebuild the app.");
        return bytes;
      } finally {
        clearTimeout(timeout);
      }
    })().catch((error) => {
      pending = null;
      throw error;
    });
  }
  return pending;
}
