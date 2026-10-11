import type {
  ContentDigest,
  EpochCheckpoint,
  EvidenceVerificationReport,
  ProofBundle,
  MutationAction,
  CaptureReport,
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
export type Comparison = {
  baseline: EvidenceVerificationReport;
  derived: EvidenceVerificationReport;
};
export type ReplayImport = {
  baseline: Bundle;
  derived: Bundle | null;
  checkpoint: Checkpoint | null;
  requiredCheckpoint: ContentDigest | null;
  profileId: string;
  policyId: string;
  hasSavedReports: boolean;
};
export type ExportedReplay = { bytes: Uint8Array; filename: string };
export type Job =
  | { kind: "capture"; bytes: Uint8Array }
  | {
      kind: "mutate";
      baseline: Uint8Array;
      certificateId: string;
      action: MutationAction;
    }
  | { kind: "replay"; bytes: Uint8Array }
  | {
      kind: "export";
      baseline: Uint8Array;
      derived: Uint8Array | null;
      checkpoint: Uint8Array | null;
      checkpointDigest: ContentDigest | null;
      reports:
        | Comparison
        | {
            baseline: EvidenceVerificationReport | null;
            derived: EvidenceVerificationReport | null;
          };
    }
  | {
      kind: "compare";
      baseline: Uint8Array;
      derived: Uint8Array;
      checkpoint: Uint8Array | null;
      selection: Selection | null;
      requiredCheckpoint: ContentDigest | null;
    }
  | { kind: "bundle"; bytes: Uint8Array }
  | { kind: "checkpoint"; bytes: Uint8Array }
  | { kind: "example"; bundle: Uint8Array; checkpoint: Uint8Array }
  | {
      kind: "verify";
      bundle: Uint8Array;
      requiredCheckpoint?: ContentDigest | null;
      checkpoint: Uint8Array | null;
      selection: Selection | null;
    };
export type JobResult =
  | CaptureReport
  | Bundle
  | Comparison
  | ReplayImport
  | ExportedReplay
  | Inspected<ProofBundle>
  | Inspected<EpochCheckpoint>
  | { bundle: Inspected<ProofBundle>; checkpoint: Inspected<EpochCheckpoint> }
  | EvidenceVerificationReport;
export type WorkerRequest = { id: string; job: Job; wasmBytes: ArrayBuffer };
export type WorkerResponse = { id: string } & (
  { ok: true; value: JobResult } | { ok: false; error: string }
);
export const PROFILE = "agave-98535e50-base2-v1";
export const MAX_CHECKPOINT_BYTES = 4 * 1024 * 1024;

export const POLICY = "agave-98535e50-direct-finality-v1";
