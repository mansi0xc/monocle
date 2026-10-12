import AxeBuilder from "@axe-core/playwright";
import { test, expect, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
const corpus = JSON.parse(
  readFileSync(
    new URL("../../../fixtures/finality/corpus.json", import.meta.url),
    "utf8",
  ),
);
const fixture = (name: string) =>
  corpus.cases.find((c: { name: string }) => c.name === name);
const upload = (text: string | Buffer, name = "evidence.json") => ({
  name,
  mimeType: "application/json",
  buffer: Buffer.isBuffer(text) ? text : Buffer.from(text),
});
async function load(page: Page, name = "fast_finality") {
  await page.getByLabel("Signed synthetic example").selectOption(name);
  await page.getByRole("button", { name: "Load", exact: true }).click();
  await expect(page.getByRole("status")).toHaveText(
    "Example loaded. Select its checkpoint before verifying.",
  );
}
async function verify(page: Page) {
  await page.getByRole("button", { name: "Verify evidence" }).click();
}
async function select(page: Page) {
  await page.getByRole("button", { name: "Use this checkpoint" }).click();
}
test.beforeEach(async ({ page }) => {
  await page.goto("/#/inspect");
});

test("empty state and keyboard entry", async ({ page }, info) => {
  await expect(
    page.getByRole("heading", { name: "Every conclusion needs a trace." }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Verify evidence" }),
  ).toBeDisabled();
  await page.keyboard.press("Tab");
  await expect(
    page.getByRole("link", { name: "Skip to inspector" }),
  ).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/#inspector$/);
  await expect(page.getByRole("main")).toBeInViewport();
  await expect(
    page.getByRole("heading", { name: "Every conclusion needs a trace." }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Verify evidence" }),
  ).toBeDisabled();
  await page.screenshot({ path: info.outputPath("empty-desktop.png") });
});
test("explicit selection gates genuine fast finality and signer inspection", async ({
  page,
}, info) => {
  await load(page);
  await verify(page);
  await expect(
    page.getByRole("heading", { name: "Verification inconclusive" }),
  ).toBeVisible();
  await select(page);
  await expect(
    page.getByRole("heading", { name: "Verification inconclusive" }),
  ).toHaveCount(0);
  await verify(page);
  await expect(
    page.getByRole("heading", { name: "Fast finality verified" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Signer weights" }),
  ).toBeVisible();
  await page.getByLabel("Show all validators").check();
  await expect(page.locator("tbody tr")).toHaveCount(5);
  await page.evaluate(() => window.scrollTo(0, 0));
  await page.screenshot({ path: info.outputPath("fast-desktop.png") });
  await page.getByRole("button", { name: "Remove selection" }).click();
  await expect(
    page.getByRole("heading", { name: "Fast finality verified" }),
  ).toHaveCount(0);
});
test("slow path and genesis retain distinct scopes", async ({ page }) => {
  await load(page, "slow_finality");
  await select(page);
  await verify(page);
  await expect(
    page.getByRole("heading", { name: "Slow finality verified" }),
  ).toBeVisible();
  await expect(page.locator("details.certificate")).toHaveCount(2);
  await load(page, "genesis_anchor");
  await expect(page.getByText("✓ Checkpoint selected")).toHaveCount(0);
  await select(page);
  await verify(page);
  await expect(
    page.getByRole("heading", { name: "Genesis anchor verified" }),
  ).toBeVisible();
  await expect(
    page
      .getByLabel("Evidence report")
      .getByText("genesis migration", { exact: true }),
  ).toBeVisible();
});
test("invalid signature, real under-quorum signature, and missing counterpart", async ({
  page,
}) => {
  for (const [name, title] of [
    ["changed_signature", "Evidence rejected"],
    ["fast_under_quorum", "Evidence rejected"],
    ["notar_only", "Verification inconclusive"],
  ]) {
    await load(page, name);
    await select(page);
    await verify(page);
    await expect(page.getByRole("heading", { name: title })).toBeVisible();
  }
});
test("malformed, oversized, bad UTF-8, and saved-report imports fail without stale success", async ({
  page,
}) => {
  await load(page);
  await select(page);
  await verify(page);
  await expect(
    page.getByRole("heading", { name: "Fast finality verified" }),
  ).toBeVisible();
  for (const [bytes, code] of [
    ['{"schema_version":1,"schema_version":1}', "DUPLICATE_FIELD"],
    [Buffer.alloc(1024 * 1024 + 1), "1 MiB"],
    [Buffer.from([255]), "MALFORMED_JSON"],
    [
      JSON.stringify({
        ...JSON.parse(fixture("fast_finality").bundle_json),
        report: { conclusion: "VERIFIED" },
      }),
      "UNKNOWN_FIELD",
    ],
  ] as const) {
    await page
      .getByLabel("Or import a proof bundle")
      .setInputFiles(upload(bytes));
    await expect(page.getByRole("alert")).toContainText(code);
    await expect(
      page.getByRole("heading", { name: "Fast finality verified" }),
    ).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: "Verify evidence" }),
    ).toBeDisabled();
  }
});
test("import preserves exact integers and identity while rendering hostile strings as text", async ({
  page,
}) => {
  const input = JSON.parse(fixture("fast_finality").bundle_json);
  input.claim.slot = "18446744073709551615";
  input.provenance.description = '<img src=x onerror="window.pwned=true">';
  const source = JSON.stringify(input) + "\n";
  await page
    .getByLabel("Or import a proof bundle")
    .setInputFiles(upload(source));
  await expect(page.getByRole("status")).toContainText("Bundle inspected");
  await expect(page.getByText(input.claim.slot, { exact: true })).toBeVisible();
  await expect(
    page.getByText(input.provenance.description, { exact: true }),
  ).toBeVisible();
  expect(await page.evaluate(() => "pwned" in window)).toBe(false);
  await page.getByText("Input identity", { exact: true }).click();
  await expect(
    page
      .getByRole("complementary")
      .getByText(
        `sha256:${createHash("sha256").update(source).digest("hex")}`,
        { exact: true },
      ),
  ).toBeVisible();
});
test("checkpoint import grants nothing and replacement revokes selection", async ({
  page,
}) => {
  await load(page);
  await select(page);
  await page
    .getByLabel("Import checkpoint", { exact: true })
    .setInputFiles(
      upload(fixture("fast_finality").checkpoint_json, "checkpoint.json"),
    );
  await expect(page.getByRole("status")).toContainText("Checkpoint inspected");
  await expect(page.getByText("✓ Checkpoint selected")).toHaveCount(0);
  await page.getByLabel("Trust basis").selectOption("operator");
  await select(page);
  await expect(page.getByRole("alert")).toContainText("trust reason");
  await page
    .getByLabel("Why do you trust this checkpoint?")
    .fill("Reviewed the public synthetic fixture locally.");
  await select(page);
  await verify(page);
  await expect(
    page.getByRole("heading", { name: "Fast finality verified" }),
  ).toBeVisible();
  const cp = JSON.parse(fixture("fast_finality").checkpoint_json);
  cp.trusted = true;
  await page
    .getByLabel("Import checkpoint", { exact: true })
    .setInputFiles(upload(JSON.stringify(cp)));
  await expect(page.getByRole("alert")).toContainText("UNKNOWN_FIELD");
  await expect(page.getByText("✓ Checkpoint selected")).toHaveCount(0);
});
test("cancellation and replacement discard delayed worker responses", async ({
  page,
}) => {
  let release: (() => void) | undefined;
  await page.route("**/*.wasm", async (route) => {
    await new Promise<void>((resolve) => {
      release = resolve;
    });
    await route.continue().catch(() => {});
  });
  await page.reload();
  await page.getByRole("button", { name: "Load", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Cancel operation" }),
  ).toBeVisible();
  await expect.poll(() => Boolean(release)).toBe(true);
  await page.getByRole("button", { name: "Cancel operation" }).click();
  release!();
  await page.unroute("**/*.wasm");
  await expect(page.getByRole("status")).toContainText("Operation cancelled");
  await load(page, "genesis_anchor");
  await select(page);
  await verify(page);
  await expect(
    page.getByRole("heading", { name: "Genesis anchor verified" }),
  ).toBeVisible();
});
test("missing WASM is an actionable failure and a retry works", async ({
  page,
}) => {
  await page.route("**/*.wasm", (route) =>
    route.fulfill({ status: 404, body: "" }),
  );
  await page.reload();
  await page.getByRole("button", { name: "Load", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText(
    "Verifier asset unavailable",
  );
  await page.unroute("**/*.wasm");
  await load(page);
});
test("narrow viewport and reduced motion preserve controls and report", async ({
  page,
}, info) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.emulateMedia({ reducedMotion: "reduce" });
  await load(page);
  await select(page);
  await verify(page);
  await expect(
    page.getByRole("heading", { name: "Fast finality verified" }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  await page
    .getByRole("heading", { name: "Fast finality verified" })
    .scrollIntoViewIfNeeded();
  await page.screenshot({ path: info.outputPath("fast-mobile.png") });
});

test("automated accessibility checks on empty, verified, and narrow states", async ({
  page,
}) => {
  const audit = async () => {
    const result = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
      .analyze();
    expect(result.violations).toEqual([]);
  };
  await audit();
  await load(page);
  await select(page);
  await verify(page);
  await expect(
    page.getByRole("heading", { name: "Fast finality verified" }),
  ).toBeVisible();
  await audit();
  await page.setViewportSize({ width: 375, height: 812 });
  await audit();
});

test("replacing an input during an asynchronous file read ignores its eventual completion", async ({
  page,
}) => {
  await page.evaluate(() => {
    const read = File.prototype.arrayBuffer;
    File.prototype.arrayBuffer = async function () {
      if (this.name === "delayed.json")
        await new Promise((resolve) => setTimeout(resolve, 400));
      return read.call(this);
    };
  });
  await page
    .getByLabel("Or import a proof bundle")
    .setInputFiles(
      upload(fixture("fast_finality").bundle_json, "delayed.json"),
    );
  await load(page, "genesis_anchor");
  await select(page);
  await verify(page);
  await expect(
    page.getByRole("heading", { name: "Genesis anchor verified" }),
  ).toBeVisible();
  // Wait past the deliberately delayed read, not a timing assumption about crypto.
  await page.waitForTimeout(500);
  await expect(
    page.getByRole("heading", { name: "Genesis anchor verified" }),
  ).toBeVisible();
  await expect(page.getByText("delayed.json", { exact: true })).toHaveCount(0);
});

test("mismatched context and unsupported profile remain inconclusive in the inspector", async ({
  page,
}) => {
  for (const name of ["wrong_context", "unknown_profile"]) {
    await load(page);
    await select(page);
    await page
      .getByLabel("Or import a proof bundle")
      .setInputFiles(upload(fixture(name).bundle_json));
    await expect(page.getByRole("status")).toContainText("Bundle inspected");
    await verify(page);
    await expect(
      page.getByRole("heading", { name: "Verification inconclusive" }),
    ).toBeVisible();
    await expect(
      page
        .getByLabel("Evidence report")
        .getByText("unestablished", { exact: true }),
    ).toBeVisible();
  }
});
