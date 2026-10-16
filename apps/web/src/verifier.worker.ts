/// <reference lib="webworker" />
import {
  createVerifyRequest,
  encodeVerifyRequest,
  MAX_REPLAY_BYTES,
  Operation,
  REPLAY_FORMAT,
} from "../../../packages/contracts/src/index";
import type {
  DerivedEvidence,
  ProofBundle,
  EpochCheckpoint,
  ReplayArchive,
  ContentDigest,
  EvidenceVerificationReport,
} from "../../../packages/contracts/src/index";
import { loadVerifier } from "./wasm";
import {
  PROFILE,
  POLICY,
  type WorkerRequest,
  type WorkerResponse,
  type Inspected,
  type Bundle,
  type Checkpoint,
  type JobResult,
} from "./protocol";
const encoder = new TextEncoder();
const text = (bytes: Uint8Array) =>
  new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(bytes);
const json = (value: unknown) => encoder.encode(JSON.stringify(value));
self.onmessage = async ({
  data: { id, job, wasmBytes },
}: MessageEvent<WorkerRequest>) => {
  try {
    if (!wasmBytes)
      throw new Error("Verifier assets are not loaded. Reconnect and retry.");
    const call = await loadVerifier(wasmBytes);
    const inspect = (bytes: Uint8Array, name: string): Bundle => ({
      bytes,
      name,
      parsed: call<Inspected<ProofBundle>>(Operation.InspectBundle, bytes),
    });
    const checkpoint = (bytes: Uint8Array): Checkpoint => ({
      bytes,
      name: "Replay checkpoint",
      parsed: call<Inspected<EpochCheckpoint>>(
        Operation.InspectCheckpoint,
        bytes,
      ),
    });
    function archive(
      baseline: Uint8Array,
      derived: Uint8Array | null,
      cp: Uint8Array | null,
      digest: ContentDigest | null,
      reports: ReplayArchive["saved_reports"] = null,
    ): Uint8Array {
      const original = call<Inspected<ProofBundle>>(
        Operation.InspectBundle,
        baseline,
      );
      const bytes = json({
        schema_version: 1,
        format: REPLAY_FORMAT,
        profile_id: original.document.profile_id,
        policy_id: POLICY,
        baseline_json: text(baseline),
        derived_json: derived ? text(derived) : null,
        checkpoint_json: cp ? text(cp) : null,
        checkpoint_digest:
          digest ?? original.document.checkpoint_reference ?? null,
        saved_reports: reports,
      } satisfies ReplayArchive);
      if (bytes.byteLength > MAX_REPLAY_BYTES)
        throw new Error("Replay file exceeds the 16 MiB archive limit.");
      call(Operation.InspectReplay, bytes); // The same strict lineage/identity checks govern import and export.
      return bytes;
    }
    let value: JobResult;
    if (job.kind === "capture")
      value = call(Operation.InspectCapture, job.bytes);
    else if (job.kind === "bundle")
      value = call(Operation.InspectBundle, job.bytes);
    else if (job.kind === "checkpoint")
      value = call(Operation.InspectCheckpoint, job.bytes);
    else if (job.kind === "example")
      value = {
        bundle: call<Inspected<ProofBundle>>(
          Operation.InspectBundle,
          job.bundle,
        ),
        checkpoint: call<Inspected<EpochCheckpoint>>(
          Operation.InspectCheckpoint,
          job.checkpoint,
        ),
      };
    else if (job.kind === "mutate") {
      const result = call<DerivedEvidence>(
        Operation.Mutate,
        json({
          schema_version: 1,
          baseline_json: text(job.baseline),
          certificate_id: job.certificateId,
          action: job.action,
        }),
      );
      value = {
        bytes: encoder.encode(result.bundle_json),
        name: "Derived experiment",
        parsed: { digest: result.digest, document: result.document },
      };
    } else if (job.kind === "replay") {
      const replay = call<ReplayArchive>(Operation.InspectReplay, job.bytes);
      value = {
        baseline: inspect(
          encoder.encode(replay.baseline_json),
          "Replay baseline",
        ),
        derived: replay.derived_json
          ? inspect(encoder.encode(replay.derived_json), "Derived experiment")
          : null,
        checkpoint: replay.checkpoint_json
          ? checkpoint(encoder.encode(replay.checkpoint_json))
          : null,
        requiredCheckpoint: replay.checkpoint_digest,
        profileId: replay.profile_id,
        policyId: replay.policy_id,
        hasSavedReports: Boolean(
          replay.saved_reports?.baseline || replay.saved_reports?.derived,
        ),
      };
      // Saved reports never leave the worker or become a current result.
    } else if (job.kind === "export") {
      value = {
        bytes: archive(
          job.baseline,
          job.derived,
          job.checkpoint,
          job.checkpointDigest,
          job.reports,
        ),
        filename: "monocle-replay.json",
      };
    } else {
      if (
        job.requiredCheckpoint &&
        job.selection &&
        job.requiredCheckpoint !== job.selection.digest
      )
        throw new Error(
          "Replay requires a different checkpoint digest. Import and select the exact referenced checkpoint.",
        );
      if (job.kind === "compare")
        archive(job.baseline, job.derived, null, job.requiredCheckpoint);
      if (job.selection) {
        if (!job.checkpoint)
          throw new Error("The selected checkpoint is no longer available.");
        call(Operation.InspectCheckpoint, job.checkpoint);
        const digest = call<string>(
          job.selection.basis === "synthetic"
            ? Operation.SelectSynthetic
            : Operation.SelectOperator,
          encoder.encode(
            job.selection.basis === "synthetic" ? "" : job.selection.reason,
          ),
        );
        if (digest !== job.selection.digest)
          throw new Error(
            "Checkpoint bytes changed since selection. Select the checkpoint again.",
          );
      }
      const verify = (bytes: Uint8Array, run: string) =>
        call<EvidenceVerificationReport>(
          Operation.Verify,
          encodeVerifyRequest(
            createVerifyRequest(
              run,
              PROFILE,
              bytes,
              job.selection?.digest ?? null,
            ),
          ),
        );
      value =
        job.kind === "compare"
          ? {
              baseline: verify(job.baseline, `${id}-baseline`),
              derived: verify(job.derived, `${id}-derived`),
            }
          : verify(job.bundle, id);
    }
    self.postMessage({ id, ok: true, value } satisfies WorkerResponse);
  } catch (error) {
    self.postMessage({
      id,
      ok: false,
      error:
        error instanceof Error ? error.message : "The verifier worker failed.",
    } satisfies WorkerResponse);
  }
};
