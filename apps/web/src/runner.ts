import type { Job, JobResult, WorkerResponse } from "./protocol";
export interface WorkerPort {
  onmessage: ((event: MessageEvent<WorkerResponse>) => void) | null;
  onerror: ((event: ErrorEvent) => void) | null;
  onmessageerror: ((event: MessageEvent) => void) | null;
  postMessage(message: unknown): void;
  terminate(): void;
}
/** A fresh worker per job bounds lifetime and discards cancelled verifier state. */
export class Runner {
  private active: {
    id: string;
    worker: WorkerPort;
    reject: (reason: Error) => void;
    timer: ReturnType<typeof setTimeout>;
  } | null = null;
  private factory: () => WorkerPort;
  private timeout: number;
  private prepare?: () => Promise<ArrayBuffer>;
  constructor(
    factory: () => WorkerPort,
    timeout = 30_000,
    prepare?: () => Promise<ArrayBuffer>,
  ) {
    this.factory = factory;
    this.timeout = timeout;
    this.prepare = prepare;
  }
  cancel() {
    const active = this.active;
    if (!active) return;
    this.active = null;
    clearTimeout(active.timer);
    active.worker.terminate();
    active.reject(new DOMException("Run cancelled.", "AbortError"));
  }
  run<T extends JobResult>(job: Job): Promise<T> {
    this.cancel();
    const id = crypto.randomUUID();
    return new Promise((resolve, reject) => {
      let worker: WorkerPort;
      try {
        worker = this.factory();
      } catch {
        reject(
          new Error(
            "Unable to start the verifier worker. Reload and try again.",
          ),
        );
        return;
      }
      const finish = (error?: Error, value?: T) => {
        if (this.active?.id !== id) return;
        clearTimeout(this.active.timer);
        this.active = null;
        worker.terminate();
        if (error) reject(error);
        else resolve(value!);
      };
      const timer = setTimeout(
        () =>
          finish(
            new Error(
              "Verification exceeded 30 seconds. The worker was stopped; try a smaller input.",
            ),
          ),
        this.timeout,
      );
      this.active = { id, worker, timer, reject };
      worker.onmessage = ({ data }) => {
        if (data.id !== id || this.active?.id !== id) return;
        if (!data.ok) {
          finish(new Error(data.error));
          return;
        }
        if (
          job.kind === "verify" &&
          (!("run_id" in data.value) || data.value.run_id !== id)
        ) {
          finish(new Error("Verifier response did not match this run."));
          return;
        }
        if (
          job.kind === "compare" &&
          (!("baseline" in data.value) ||
            !("derived" in data.value) ||
            data.value.baseline === null ||
            !("run_id" in data.value.baseline) ||
            data.value.baseline.run_id !== `${id}-baseline` ||
            data.value.derived === null ||
            !("run_id" in data.value.derived) ||
            data.value.derived.run_id !== `${id}-derived`)
        ) {
          finish(new Error("Comparison reports did not match this run."));
          return;
        }
        finish(undefined, data.value as T);
      };
      worker.onerror = (event) => {
        event.preventDefault();
        finish(new Error("Verifier worker failed. Reload and try again."));
      };
      worker.onmessageerror = () =>
        finish(new Error("Could not read the verifier worker response."));
      const send = (wasmBytes?: ArrayBuffer) => {
        if (this.active?.id !== id) return;
        try {
          worker.postMessage({ id, job, wasmBytes });
        } catch {
          finish(new Error("Could not send evidence to the verifier worker."));
        }
      };
      if (this.prepare)
        this.prepare().then(send, (error) =>
          finish(
            error instanceof Error
              ? error
              : new Error("Verifier assets unavailable."),
          ),
        );
      else send();
    });
  }
}
