import { test } from "node:test";
import assert from "node:assert/strict";
import {
  byteRange,
  describe,
  fieldValue,
  hexBytes,
  littleEndian,
} from "../src/bytes.ts";

test("only this certificate's byte paths locate bytes, within bounds", () => {
  assert.deepEqual(byteRange("certificates[1].raw_hex[42..234]", 1, 248), {
    start: 42,
    end: 234,
  });
  assert.deepEqual(byteRange("certificates[0].raw_hex", 0, 248), {
    start: 0,
    end: 248,
  });
  assert.deepEqual(byteRange("certificates[0].raw_hex[248..248]", 0, 248), {
    start: 248,
    end: 248,
  });
  for (const path of [
    "certificates[1].raw_hex[42..234]", // another certificate
    "certificates[0].raw_hex[10..2]", // reversed
    "certificates[0].raw_hex[0..249]", // past the end
    "certificates[0].kind",
    "claim.slot",
    "$",
    "certificates[0].raw_hex[1..2] ",
  ])
    assert.equal(byteRange(path, 0, 248), null, path);
});

test("little-endian values stay exact beyond 2^53", () => {
  assert.equal(
    littleEndian(hexBytes("ffffffffffffffff")),
    "18446744073709551615",
  );
  assert.equal(littleEndian(hexBytes("2a00000000000000")), "42");
  assert.equal(fieldValue("shred_version", hexBytes("d204")), "1234");
  assert.equal(fieldValue("kind", ["0c"]), "12 (genesis)");
  assert.equal(fieldValue("kind", ["0a"]), "10 (unsupported)");
});

test("ranges are described for people, not offsets", () => {
  assert.equal(describe({ start: 42, end: 234 }), "bytes 42–233 (192 bytes)");
  assert.equal(describe({ start: 1, end: 2 }), "byte 1");
  assert.equal(describe({ start: 248, end: 248 }), "no bytes at offset 248");
});
