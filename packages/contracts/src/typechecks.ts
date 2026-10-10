import { decimalU64, type ProofBundle, type EpochCheckpoint, type VerifyRequest } from './index.js';
// Compile-only regression checks: JSON-facing u64 values must never be Number.
const slot: ProofBundle['claim']['slot'] = decimalU64(18446744073709551615n);
const stake: EpochCheckpoint['total_stake'] = '9007199254740993';
// @ts-expect-error Numeric u64 transport loses precision.
const invalidSlot: ProofBundle['claim']['slot'] = 9007199254740993;
// @ts-expect-error Trust is not an imported request field.
const importedTrust: VerifyRequest = { schema_version: 1, run_id: 'test', profile_id: 'test', bundle_json: '{}', trusted: true };
// @ts-expect-error The helper refuses Number even for small values.
decimalU64(42);
void [slot, stake, invalidSlot, importedTrust];
