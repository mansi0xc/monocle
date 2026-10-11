import { test } from "node:test";
import assert from "node:assert/strict";
import { Runner, type WorkerPort } from "../src/runner.ts";
import type { WorkerResponse } from "../src/protocol.ts";
class FakeWorker implements WorkerPort {
  onmessage: WorkerPort["onmessage"] = null;
  onerror: WorkerPort["onerror"] = null;
  onmessageerror: WorkerPort["onmessageerror"] = null;
  sent: { id: string } | null = null;
  stopped = false;
  postMessage(message: unknown) {
    this.sent = message as { id: string };
  }
  terminate() {
    this.stopped = true;
  }
  reply(id = this.sent!.id, run = id) {
    this.onmessage?.({
      data: { id, ok: true, value: { run_id: run } } as WorkerResponse,
    } as MessageEvent<WorkerResponse>);
  }
}
const verify = {
  kind: "verify",
  bundle: new Uint8Array(),
  checkpoint: null,
  selection: null,
} as const;
function setup(timeout = 30_000) {
  const workers: FakeWorker[] = [];
  return {
    workers,
    runner: new Runner(() => {
      const worker = new FakeWorker();
      workers.push(worker);
      return worker;
    }, timeout),
  };
}
test("a new job terminates the old worker and ignores its late response", async () => {
  const { workers, runner } = setup();
  const first = runner.run(verify);
  const cancelled = assert.rejects(first, { name: "AbortError" });
  const second = runner.run(verify);
  assert.equal(workers[0].stopped, true);
  workers[0].reply();
  workers[1].reply("other-run");
  workers[1].reply();
  await cancelled;
  await second;
  assert.equal(workers[1].stopped, true);
});
test("cancel is idempotent and cannot later publish success", async () => {
  const { workers, runner } = setup();
  const promise = runner.run(verify);
  const rejected = assert.rejects(promise, { name: "AbortError" });
  runner.cancel();
  runner.cancel();
  workers[0].reply();
  await rejected;
});
test("deadline terminates an unresponsive worker", async () => {
  const { workers, runner } = setup(5);
  await assert.rejects(runner.run(verify), /exceeded/);
  assert.equal(workers[0].stopped, true);
});
test("matching envelope cannot conceal a mismatched report run ID", async () => {
  const { workers, runner } = setup();
  const promise = runner.run(verify);
  workers[0].reply(workers[0].sent!.id, "stale");
  await assert.rejects(promise, /did not match/);
});
test("worker crashes and serialization failures terminate and reject", async () => {
  const { workers, runner } = setup();
  const crash = runner.run(verify);
  workers[0].onerror?.({ preventDefault() {} } as ErrorEvent);
  await assert.rejects(crash, /worker failed/);
  const decode = runner.run(verify);
  workers[1].onmessageerror?.({} as MessageEvent);
  await assert.rejects(decode, /worker response/);
  assert.ok(workers.every((w) => w.stopped));
});
test("creation and posting failures are reported", async () => {
  await assert.rejects(
    new Runner(() => {
      throw new Error();
    }).run(verify),
    /start/,
  );
  const worker = new FakeWorker();
  worker.postMessage = () => {
    throw new Error();
  };
  await assert.rejects(new Runner(() => worker).run(verify), /send/);
  assert.equal(worker.stopped, true);
});
