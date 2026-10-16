/** Deliberately small ABI: Rust owns every buffer; JS never supplies an address. */
import {
  ABI_VERSION,
  MAX_BRIDGE_INPUT_BYTES,
  type Operation,
} from "../../../packages/contracts/src/generated.ts"; // Node also loads this file directly.
interface Exports extends WebAssembly.Exports {
  memory: WebAssembly.Memory;
  monocle_abi_version: () => number;
  monocle_input_alloc: (length: number) => number;
  monocle_execute: (operation: number) => number;
  monocle_output_ptr: () => number;
  monocle_output_len: () => number;
}
export async function loadVerifier(bytes: BufferSource) {
  const { instance } = await WebAssembly.instantiate(bytes, {});
  const api = instance.exports as Exports;
  if (api.monocle_abi_version() !== ABI_VERSION)
    throw new Error("Unsupported verifier ABI. Rebuild the app assets.");
  return function call<T>(
    operation: Operation,
    input: Uint8Array = new Uint8Array(),
  ): T {
    if (input.byteLength > MAX_BRIDGE_INPUT_BYTES)
      throw new Error("Verifier input exceeds its size limit.");
    const ptr = api.monocle_input_alloc(input.byteLength);
    if (!ptr)
      throw new Error("The verifier could not allocate its input buffer.");
    new Uint8Array(api.memory.buffer, ptr, input.byteLength).set(input);
    if (api.monocle_execute(operation) !== 1)
      throw new Error("The verifier could not complete this operation.");
    const size = api.monocle_output_len();
    if (!size || size > 32 * 1024 * 1024)
      throw new Error("The verifier returned an invalid response size.");
    // Execution can grow memory. Never reuse the pre-execution view.
    const output = new Uint8Array(
      api.memory.buffer,
      api.monocle_output_ptr(),
      size,
    );
    const response = JSON.parse(
      new TextDecoder("utf-8", { fatal: true }).decode(output),
    );
    if (!response.ok)
      throw new Error(
        `${response.error.code} · ${response.error.path}: ${response.error.message}`,
      );
    return response.value as T;
  };
}
