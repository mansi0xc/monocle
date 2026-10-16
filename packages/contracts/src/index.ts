/** Transport types only. Rust is authoritative for validation and conclusions.
 * Enumerations and limits come from `generated.ts`, which Rust writes. */
import type {
  CaptureObservation,
  CertificateKind,
  CheckStatus,
  ClaimKind,
  Conclusion,
  DiagnosticCode,
  EvidenceScope,
  FinalityPath,
  SourceKind,
  Stage,
} from './generated.js';
import { CAPTURE_ADAPTER_ID, MAX_BUNDLE_BYTES, REPLAY_FORMAT } from './generated.js';
export * from './generated.js';
export type DecimalU64 = string;
export type Hex = string;
export type ContentDigest = `sha256:${string}`;
export interface ChainContext {
  genesis_hash_hex: Hex;
  epoch: DecimalU64;
  shred_version: number;
}
export interface Provenance {
  kind: SourceKind;
  description: string;
}
export interface CertificateEnvelope {
  schema_version: 1;
  id: string;
  kind: string;
  format: string;
  raw_hex: Hex;
}
export interface EvidenceClaim {
  kind: string;
  slot: DecimalU64;
  block_id_hex?: Hex | null;
  certificate_ids: string[];
}
export interface MutationRecord {
  schema_version: 1;
  certificate_id: string;
  operation: 'replace_bytes';
  offset: DecimalU64;
  before_hex: Hex;
  after_hex: Hex;
  parent_certificate_digest: ContentDigest;
  result_certificate_digest: ContentDigest;
}
export interface ProofBundle {
  schema_version: 1;
  profile_id: string;
  context: ChainContext;
  claim: EvidenceClaim;
  certificates: CertificateEnvelope[];
  checkpoint_reference?: ContentDigest | null;
  provenance: Provenance;
  parent_digest?: ContentDigest | null;
  mutations: MutationRecord[];
}
export interface ValidatorEntry {
  rank: number;
  identity_hex: Hex;
  bls_pubkey_hex: Hex;
  stake: DecimalU64;
  proof_of_possession_hex?: Hex | null;
}
export interface EpochCheckpoint {
  schema_version: 1;
  profile_id: string;
  context: ChainContext;
  total_stake: DecimalU64;
  validators: ValidatorEntry[];
  provenance: Provenance;
}
export interface ProtocolProfile {
  schema_version: 1;
  id: string;
  agave_revision: string;
  capability: 'contracts_only' | 'base2_certificate_verification';
  recognized_formats: readonly string[];
  recognized_certificate_kinds: readonly string[];
  recognized_claim_kinds: readonly string[];
}
export interface VerifyRequest {
  schema_version: 1;
  run_id: string;
  profile_id: string;
  bundle_json: string;
  selected_checkpoint?: ContentDigest | null;
}
export interface Diagnostic { code: DiagnosticCode; path: string; message: string }
export interface CheckResult {
  stage: Stage;
  status: CheckStatus;
  diagnostic: Diagnostic;
}
export interface VerificationReport {
  schema_version: 1;
  run_id: string | null;
  bundle_digest: ContentDigest | null;
  profile_id: string | null;
  checkpoint_digest: ContentDigest | null;
  conclusion: Conclusion;
  evidence_scope: 'preflight_only';
  checks: CheckResult[];
  assumptions: string[];
}

/** For locally constructed values; never round-trip a u64 through Number. */
export function decimalU64(value: string | bigint): DecimalU64 {
  if (typeof value !== 'string' && typeof value !== 'bigint') throw new TypeError('Use a string or bigint, never Number');
  const text = String(value);
  if (!/^(0|[1-9][0-9]{0,19})$/.test(text) || BigInt(text) > 18446744073709551615n) {
    throw new RangeError('Expected a canonical unsigned 64-bit integer');
  }
  return text;
}
/** Preserve the input text, including duplicate keys, until Rust validates it.
 * This does not parse evidence, select trust, or produce a verification result. */
export function createVerifyRequest(
  runId: string,
  profileId: string,
  bundleBytes: Uint8Array,
  selectedCheckpoint: ContentDigest | null,
): VerifyRequest {
  if (bundleBytes.byteLength > MAX_BUNDLE_BYTES) throw new RangeError('Bundle exceeds size limit');
  // Retain a BOM if present so the authoritative parser can reject it, not silently rewrite it.
  const bundleJson = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(bundleBytes);
  return { schema_version: 1, run_id: runId, profile_id: profileId, bundle_json: bundleJson, selected_checkpoint: selectedCheckpoint };
}
export function encodeVerifyRequest(request: VerifyRequest): Uint8Array {
  return new TextEncoder().encode(JSON.stringify(request));
}

export interface CertificateReport {
  certificate_id: string;
  raw_digest: ContentDigest;
  conclusion: Conclusion;
  kind: CertificateKind | null;
  slot: DecimalU64 | null;
  block_id_hex: Hex | null;
  signed_payload_hex: Hex | null;
  signer_ranks: number[];
  /** Selected checkpoint weight; signature acceptance is a separate check. */
  selected_stake: DecimalU64 | null;
  total_stake: DecimalU64 | null;
  threshold_percent: number | null;
  checks: CheckResult[];
}
/** C3 only establishes individual certificate validity under the selected checkpoint. */
export interface CertificateVerificationReport {
  schema_version: 1;
  run_id: string | null;
  bundle_digest: ContentDigest | null;
  profile_id: string | null;
  checkpoint_digest: ContentDigest | null;
  conclusion: Conclusion;
  evidence_scope: 'individual_certificates';
  checks: CheckResult[];
  certificates: CertificateReport[];
  assumptions: string[];
}

export interface EvaluatedClaim {
  kind: ClaimKind;
  slot: DecimalU64;
  block_id_hex: Hex | null;
}
/** C4: scope/path/witnesses describe established evidence only. The claim is the requested target. */
export interface EvidenceVerificationReport {
  schema_version: 1;
  policy_id: string;
  run_id: string | null;
  bundle_digest: ContentDigest | null;
  profile_id: string | null;
  checkpoint_digest: ContentDigest | null;
  conclusion: Conclusion;
  evidence_scope: EvidenceScope;
  claim: EvaluatedClaim | null;
  finality_path: FinalityPath | null;
  supporting_certificate_ids: string[];
  checks: CheckResult[];
  certificates: CertificateReport[];
  assumptions: string[];
}

/** C6 keeps original bytes and one independently derived replacement. */
export type MutationAction = { kind: 'signature_bit' } | { kind: 'replace_bytes'; offset: DecimalU64; after_hex: Hex };
export interface MutationRequest { schema_version: 1; baseline_json: string; certificate_id: string; action: MutationAction }
export interface DerivedEvidence { bundle_json: string; digest: ContentDigest; document: ProofBundle }
/** Saved reports are deliberately unknown/untrusted on import. They never enter verification. */
export interface ReplayArchive {
  schema_version: 1;
  format: typeof REPLAY_FORMAT;
  profile_id: string;
  policy_id: string;
  baseline_json: string;
  derived_json: string | null;
  checkpoint_json: string | null;
  checkpoint_digest: ContentDigest | null;
  saved_reports: { baseline: unknown | null; derived: unknown | null } | null;
}

export interface CaptureReport {
  schema_version: 1;
  adapter_id: typeof CAPTURE_ADAPTER_ID;
  source_digest: ContentDigest;
  source_bytes: number;
  observation: CaptureObservation;
  conclusion: 'INCONCLUSIVE';
  evidence_scope: 'capture_inspection';
  certificate: { slot: DecimalU64; block_id_hex: Hex; signature_hex: Hex; bitmap_hex: Hex } | null;
  rpc_error_code: string | null;
  rpc_error_message: string | null;
  checks: CheckResult[];
  requirements: string[];
  assumptions: string[];
}
