/// <reference lib="webworker" />
import {
  createVerifyRequest,
  encodeVerifyRequest,
} from "../../../packages/contracts/src/index";
import { loadVerifier } from "./wasm";
import { PROFILE, type WorkerRequest, type WorkerResponse } from "./protocol";

self.onmessage = async ({ data: { id, job } }: MessageEvent<WorkerRequest>) => {
  try {
    const response = await fetch(
      new URL("../generated/monocle_wasm.wasm", import.meta.url),
    );
    if (!response.ok)
      throw new Error(
        "Verifier asset unavailable. Rebuild the app and reload.",
      );
    const call = await loadVerifier(await response.arrayBuffer());
    let value;
    if (job.kind === "bundle") value = call(1, job.bytes);
    else if (job.kind === "checkpoint") value = call(2, job.bytes);
    else if (job.kind === "example")
      value = {
        bundle: call(1, job.bundle),
        checkpoint: call(2, job.checkpoint),
      };
    else {
      if (job.selection) {
        if (!job.checkpoint)
          throw new Error("The selected checkpoint is no longer available.");
        call(2, job.checkpoint);
        const digest = call<string>(
          job.selection.basis === "synthetic" ? 3 : 4,
          new TextEncoder().encode(
            job.selection.basis === "synthetic" ? "" : job.selection.reason,
          ),
        );
        if (digest !== job.selection.digest)
          throw new Error(
            "Checkpoint bytes changed since selection. Select the checkpoint again.",
          );
      }
      const request = createVerifyRequest(
        id,
        PROFILE,
        job.bundle,
        job.selection?.digest ?? null,
      );
      value = call(5, encodeVerifyRequest(request));
    }
    self.postMessage({ id, ok: true, value } as WorkerResponse);
  } catch (error) {
    self.postMessage({
      id,
      ok: false,
      error:
        error instanceof Error ? error.message : "The verifier worker failed.",
    } satisfies WorkerResponse);
  }
};
