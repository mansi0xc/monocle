import type {
  ContentDigest,
  EpochCheckpoint,
  EvidenceVerificationReport,
  ProofBundle,
} from "../../../packages/contracts/src/index";
export type Inspected<T> = { digest: ContentDigest; document: T };
export type Bundle = {
  bytes: Uint8Array;
  name: string;
  parsed: Inspected<ProofBundle>;
};
export type Checkpoint = {
  bytes: Uint8Array;
  name: string;
  parsed: Inspected<EpochCheckpoint>;
};
export type Selection = {
  digest: ContentDigest;
  basis: "synthetic" | "operator";
  reason: string;
};
export type Job =
  | { kind: "bundle"; bytes: Uint8Array }
  | { kind: "checkpoint"; bytes: Uint8Array }
  | { kind: "example"; bundle: Uint8Array; checkpoint: Uint8Array }
  | {
      kind: "verify";
      bundle: Uint8Array;
      checkpoint: Uint8Array | null;
      selection: Selection | null;
    };
export type JobResult =
  | Inspected<ProofBundle>
  | Inspected<EpochCheckpoint>
  | { bundle: Inspected<ProofBundle>; checkpoint: Inspected<EpochCheckpoint> }
  | EvidenceVerificationReport;
export type WorkerRequest = { id: string; job: Job };
export type WorkerResponse = { id: string } & (
  { ok: true; value: JobResult } | { ok: false; error: string }
);
export const PROFILE = "agave-98535e50-base2-v1";
export const MAX_CHECKPOINT_BYTES = 4 * 1024 * 1024;
