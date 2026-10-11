import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { readFile } from "node:fs/promises";
import { readFileSync } from "node:fs";
const corpus = JSON.parse(
  readFileSync(
    new URL("../../../fixtures/finality/corpus.json", import.meta.url),
    "utf8",
  ),
);
const fixture = corpus.cases.find(
  (c: { name: string }) => c.name === "fast_finality",
);
const upload = (s: string | Buffer) => ({
  name: "replay.json",
  mimeType: "application/json",
  buffer: Buffer.isBuffer(s) ? s : Buffer.from(s),
});
async function load(page: Page) {
  await page.getByRole("button", { name: "Load", exact: true }).click();
  await expect(page.getByRole("status")).toContainText("Example loaded");
  await page.getByRole("button", { name: "Use this checkpoint" }).click();
}
async function mutate(page: Page) {
  await page.locator(".tamper-lab > summary").click();
  await page
    .getByRole("button", { name: "Create experiment", exact: true })
    .click();
  await expect(page.getByRole("status")).toContainText("Experiment created");
}
async function compare(page: Page) {
  await page.getByRole("button", { name: "Verify comparison" }).click();
  await expect(page.getByRole("status")).toContainText("Comparison complete");
}
async function exported(page: Page) {
  const wait = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export replay" }).click();
  const download = await wait;
  const path = await download.path();
  return JSON.parse(await readFile(path!, "utf8"));
}
test.beforeEach(async ({ page }) => {
  await page.goto("/");
});
test("immutable original, meaningful diagnostic comparison, export and fresh replay", async ({
  page,
}, info) => {
  await load(page);
  await mutate(page);
  await compare(page);
  await expect(
    page.getByRole("heading", { name: "What changed?" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Evidence rejected" }),
  ).toBeVisible();
  await expect(page.getByLabel("Fresh comparison")).toContainText("VERIFIED");
  await expect(page.getByLabel("Fresh comparison")).toContainText("REJECTED");
  const saved = await exported(page);
  expect(saved.baseline_json).toBe(fixture.bundle_json);
  expect(saved.checkpoint_json).toBe(fixture.checkpoint_json);
  const derived = JSON.parse(saved.derived_json);
  expect(derived.mutations).toHaveLength(1);
  expect(derived.mutations[0].offset).toBe("42");
  expect(saved.saved_reports.baseline.conclusion).toBe("VERIFIED");
  expect(saved.saved_reports.derived.conclusion).toBe("REJECTED");
  await page
    .getByRole("heading", { name: "What changed?" })
    .scrollIntoViewIfNeeded();
  await page.screenshot({ path: info.outputPath("comparison-desktop.png") });
  await page
    .getByLabel("Import replay file")
    .setInputFiles(upload(JSON.stringify(saved)));
  await expect(page.getByRole("status")).toContainText(
    "Replay imported without trust",
  );
  await expect(page.getByLabel("Fresh comparison")).toHaveCount(0);
  await expect(page.getByText("✓ Checkpoint selected")).toHaveCount(0);
  await page.getByRole("button", { name: "Use this checkpoint" }).click();
  await compare(page);
  const again = await exported(page);
  const normalized = (r: Record<string, unknown>) => ({ ...r, run_id: null });
  expect(normalized(again.saved_reports.baseline)).toEqual(
    normalized(saved.saved_reports.baseline),
  );
  expect(normalized(again.saved_reports.derived)).toEqual(
    normalized(saved.saved_reports.derived),
  );
  expect(again.baseline_json).toBe(saved.baseline_json);
  expect(again.derived_json).toBe(saved.derived_json);
  await page.getByRole("button", { name: "Discard experiment" }).click();
  await expect(
    page.getByRole("button", { name: "Verify evidence" }),
  ).toBeEnabled();
  const original = await exported(page);
  expect(original.baseline_json).toBe(fixture.bundle_json);
  expect(original.derived_json).toBeNull();
});
test("forged saved successes cannot influence freshly recomputed reports", async ({
  page,
}) => {
  await load(page);
  await mutate(page);
  await compare(page);
  const saved = await exported(page);
  saved.saved_reports = {
    baseline: { conclusion: "REJECTED" },
    derived: { conclusion: "VERIFIED", trusted: true, pop_verified: true },
  };
  await page
    .getByLabel("Import replay file")
    .setInputFiles(upload(JSON.stringify(saved)));
  await expect(page.getByRole("status")).toContainText("Replay imported");
  await compare(page);
  await expect(page.getByLabel("Fresh comparison")).toContainText(
    "INCONCLUSIVE",
  );
  await page.getByRole("button", { name: "Use this checkpoint" }).click();
  await compare(page);
  await expect(
    page.getByRole("heading", { name: "Evidence rejected" }),
  ).toBeVisible();
});
test("offline production replay has no HTTP requests after assets are ready", async ({
  page,
  context,
}) => {
  await expect(
    page.getByText("Ready for offline verification", { exact: false }),
  ).toBeVisible();
  const requests: string[] = [];
  page.on("request", (r) => {
    if (r.url().startsWith("http")) requests.push(r.url());
  });
  await context.setOffline(true);
  await load(page);
  await mutate(page);
  await compare(page);
  const saved = await exported(page);
  await page
    .getByLabel("Import replay file")
    .setInputFiles(upload(JSON.stringify(saved)));
  await expect(page.getByRole("status")).toContainText("Replay imported");
  await page.getByRole("button", { name: "Use this checkpoint" }).click();
  await compare(page);
  await expect(
    page.getByRole("heading", { name: "Evidence rejected" }),
  ).toBeVisible();
  expect(requests).toEqual([]);
  await context.setOffline(false);
});
test("omitted checkpoint remains explicit and requires the exact referenced bytes", async ({
  page,
}) => {
  await load(page);
  await mutate(page);
  await page.getByLabel("Include checkpoint bytes").uncheck();
  const saved = await exported(page);
  expect(saved.checkpoint_json).toBeNull();
  expect(saved.checkpoint_digest).toMatch(/^sha256:/);
  await page
    .getByLabel("Import replay file")
    .setInputFiles(upload(JSON.stringify(saved)));
  await expect(page.getByRole("status")).toContainText("Replay imported");
  await compare(page);
  await expect(
    page.getByRole("heading", { name: "Verification inconclusive" }),
  ).toBeVisible();
  await page
    .getByLabel("Import checkpoint", { exact: true })
    .setInputFiles(upload(fixture.checkpoint_json + "\n"));
  await expect(page.getByRole("status")).toContainText("Checkpoint inspected");
  await page.getByRole("button", { name: "Use this checkpoint" }).click();
  await page.getByRole("button", { name: "Verify comparison" }).click();
  await expect(page.getByRole("alert")).toContainText(
    "different checkpoint digest",
  );
  await page
    .getByLabel("Import checkpoint", { exact: true })
    .setInputFiles(upload(fixture.checkpoint_json));
  await expect(page.getByRole("status")).toContainText("Checkpoint inspected");
  await page.getByRole("button", { name: "Use this checkpoint" }).click();
  await compare(page);
  await expect(
    page.getByRole("heading", { name: "Evidence rejected" }),
  ).toBeVisible();
});
test("corrupt lineage, unknown policy, duplicate keys and oversized replay fail without old results", async ({
  page,
}) => {
  await load(page);
  await mutate(page);
  await compare(page);
  const saved = await exported(page);
  const bad = JSON.parse(saved.derived_json);
  bad.mutations[0].before_hex = "ff";
  for (const [input, code] of [
    [
      JSON.stringify({ ...saved, derived_json: JSON.stringify(bad) }),
      "INVALID_MUTATION",
    ],
    [
      JSON.stringify({ ...saved, policy_id: "unknown-policy" }),
      "UNSUPPORTED_PROFILE",
    ],
    ['{"schema_version":1,"schema_version":1}', "DUPLICATE_FIELD"],
    [Buffer.alloc(16 * 1024 * 1024 + 1), "16 MiB"],
  ] as const) {
    await page.getByLabel("Import replay file").setInputFiles(upload(input));
    await expect(page.getByRole("alert")).toContainText(code);
    await expect(page.getByLabel("Fresh comparison")).toHaveCount(0);
  }
});
test("raw signed-field mutation and invalid range have clear outcomes", async ({
  page,
}) => {
  await load(page);
  await page.locator(".tamper-lab > summary").click();
  await page
    .getByLabel("Experiment", { exact: true })
    .selectOption("replace_bytes");
  await page.getByLabel("Byte offset").fill("18446744073709551615");
  await page.getByLabel("Replacement hex").fill("ff");
  await page
    .getByRole("button", { name: "Create experiment", exact: true })
    .click();
  await expect(page.getByRole("alert")).toContainText("INVALID_MUTATION");
  await page.getByLabel("Byte offset").fill("2");
  await page.getByLabel("Replacement hex").fill("2b");
  await page
    .getByRole("button", { name: "Create experiment", exact: true })
    .click();
  await expect(page.getByRole("status")).toContainText("Experiment created");
  await compare(page);
  await expect(page.getByLabel("Fresh comparison")).toContainText(
    "CLAIM_MISMATCH",
  );
});
test("comparison accessibility, mobile width, and context change invalidate both reports", async ({
  page,
}, info) => {
  await load(page);
  await mutate(page);
  await compare(page);
  await page.setViewportSize({ width: 375, height: 812 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  expect(
    (
      await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
        .analyze()
    ).violations,
  ).toEqual([]);
  await page
    .getByRole("heading", { name: "What changed?" })
    .scrollIntoViewIfNeeded();
  await page.screenshot({ path: info.outputPath("comparison-mobile.png") });
  await page.getByRole("button", { name: "Remove selection" }).click();
  await expect(page.getByLabel("Fresh comparison")).toHaveCount(0);
  await expect(
    page.getByRole("heading", { name: "Evidence rejected" }),
  ).toHaveCount(0);
});
