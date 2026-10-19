/** Display helpers for a certificate's raw bytes. The Rust decoder is the
 * authority: field ranges come from its reported layout, and byte ranges from
 * its diagnostic paths. Nothing here judges validity. */
import type {
  ByteRange,
  WireLayout,
} from "../../../packages/contracts/src/index";

export const FIELDS = [
  ["version", "Version"],
  ["kind", "Certificate tag"],
  ["slot", "Slot"],
  ["block_id", "Block ID"],
  ["signature", "Aggregate signature"],
  ["bitmap_length", "Bitmap length"],
  ["bitmap", "Signer bitmap"],
  ["shred_version", "Shred version"],
] as const;
export type FieldKey = (typeof FIELDS)[number][0];

const TAGS: Record<number, string> = {
  7: "finalization",
  8: "fast finalization",
  9: "notarization",
  12: "genesis",
};

/** `certificates[i].raw_hex` or `certificates[i].raw_hex[start..end]` for
 * certificate `index`, bounded by `length`; anything else is not a byte path. */
export function byteRange(
  path: string,
  index: number,
  length: number,
): ByteRange | null {
  const match = /^certificates\[(\d+)\]\.raw_hex(?:\[(\d+)\.\.(\d+)\])?$/.exec(
    path,
  );
  if (!match || Number(match[1]) !== index) return null;
  const start = match[2] === undefined ? 0 : Number(match[2]);
  const end = match[3] === undefined ? length : Number(match[3]);
  return start <= end && end <= length ? { start, end } : null;
}

export function hexBytes(hex: string): string[] {
  return hex.match(/../g) ?? [];
}

/** Little-endian unsigned integer as an exact decimal string. */
export function littleEndian(bytes: string[]): string {
  let value = 0n;
  for (let i = bytes.length - 1; i >= 0; i--)
    value = (value << 8n) | BigInt(Number.parseInt(bytes[i], 16));
  return value.toString();
}

export function fields(layout: WireLayout) {
  return FIELDS.flatMap(([key, label]) => {
    const range = layout[key];
    return range ? [{ key, label, range }] : [];
  });
}

export function fieldValue(key: FieldKey, bytes: string[]): string {
  switch (key) {
    case "version":
    case "shred_version":
    case "slot":
    case "bitmap_length":
      return littleEndian(bytes);
    case "kind": {
      const tag = Number.parseInt(bytes[0], 16);
      return `${tag} (${TAGS[tag] ?? "unsupported"})`;
    }
    case "signature":
      return `${bytes.length} bytes, ${bytes.slice(0, 6).join("")}…`;
    default:
      return bytes.join("");
  }
}

export function describe(range: ByteRange): string {
  const count = range.end - range.start;
  if (count === 0) return `no bytes at offset ${range.start}`;
  return count === 1
    ? `byte ${range.start}`
    : `bytes ${range.start}–${range.end - 1} (${count} bytes)`;
}
