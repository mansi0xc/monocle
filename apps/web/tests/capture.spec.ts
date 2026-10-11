import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
const source = readFileSync(
  new URL(
    "../../../fixtures/captures/testnet-genesis-rpc.json",
    import.meta.url,
  ),
  "utf8",
);
const upload = (s: string | Buffer) => ({
  name: "rpc.json",
  mimeType: "application/json",
  buffer: Buffer.isBuffer(s) ? s : Buffer.from(s),
});
test.beforeEach(async ({ page }) => {
  await page.goto("/");
});
test("recorded capture replaces verified evidence without inheriting trust and downloads exactly", async ({
  page,
}, info) => {
  await page.getByRole("button", { name: "Load", exact: true }).click();
  await expect(page.getByRole("status")).toContainText("Example loaded");
  await page.getByRole("button", { name: "Use this checkpoint" }).click();
  await page.getByRole("button", { name: "Verify evidence" }).click();
  await expect(
    page.getByRole("heading", { name: "Fast finality verified" }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Load recorded testnet response" })
    .click();
  await expect(
    page.getByRole("heading", { name: "Genesis capture needs context" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Fast finality verified" }),
  ).toHaveCount(0);
  await expect(page.getByText("✓ Checkpoint selected")).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Verify evidence" }),
  ).toBeDisabled();
  await expect(page.getByLabel("RPC capture inspection")).toContainText(
    "444625255",
  );
  await expect(page.getByLabel("RPC capture inspection")).toContainText(
    "CAPTURE_CONTEXT_MISSING",
  );
  await expect(
    page.getByLabel("Import checkpoint", { exact: true }),
  ).toBeDisabled();
  const downloading = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "Download original response" })
    .click();
  const file = await (await downloading).path();
  const bytes = await readFile(file!);
  expect(bytes.toString()).toBe(source);
  await page
    .getByRole("heading", { name: "Genesis capture needs context" })
    .scrollIntoViewIfNeeded();
  await page.screenshot({ path: info.outputPath("capture-desktop.png") });
  await page.getByRole("button", { name: "Clear capture" }).click();
  await page
    .getByLabel("Import genesis RPC response")
    .setInputFiles(upload(bytes));
  await expect(
    page.getByRole("heading", { name: "Genesis capture needs context" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Load", exact: true }).click();
  await expect(page.getByRole("status")).toContainText("Example loaded");
  await expect(page.getByLabel("RPC capture inspection")).toHaveCount(0);
});
test("null and RPC errors are distinct, and endpoint text is never executed", async ({
  page,
}) => {
  for (const [input, title] of [
    [
      '{"jsonrpc":"2.0","id":1,"result":null}',
      "No certificate in this response",
    ],
    [
      '{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}',
      "RPC method unavailable",
    ],
    [
      JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        error: {
          code: -32000,
          message: '<img src=x onerror="window.captureExecuted=true">',
        },
      }),
      "RPC returned an error",
    ],
  ]) {
    await page
      .getByLabel("Import genesis RPC response")
      .setInputFiles(upload(input));
    await expect(page.getByRole("heading", { name: title })).toBeVisible();
    await expect(page.getByLabel("RPC capture inspection")).toContainText(
      "INCONCLUSIVE",
    );
  }
  expect(
    await page.evaluate(() => Object.hasOwn(window, "captureExecuted")),
  ).toBe(false);
  await expect(page.locator(".capture-report img")).toHaveCount(0);
});
test("malformed envelopes, duplicate keys, footers, invalid bytes and oversized files clear old inspection", async ({
  page,
}) => {
  for (const [input, error] of [
    ['{"jsonrpc":"2.0","jsonrpc":"2.0"}', "DUPLICATE_FIELD"],
    [source.replace('"id":1', '"id":1,"trusted":true'), "UNKNOWN_FIELD"],
    [
      JSON.stringify({ slot: 999, certificate: JSON.parse(source).result }),
      "UNKNOWN_FIELD",
    ],
    [Buffer.from([255]), "MALFORMED_JSON"],
    [Buffer.alloc(64 * 1024 + 1), "64 KiB"],
  ] as const) {
    await page
      .getByRole("button", { name: "Load recorded testnet response" })
      .click();
    await expect(
      page.getByRole("heading", { name: "Genesis capture needs context" }),
    ).toBeVisible();
    await page
      .getByLabel("Import genesis RPC response")
      .setInputFiles(upload(input));
    await expect(page.getByRole("alert")).toContainText(error);
    await expect(page.getByLabel("RPC capture inspection")).toHaveCount(0);
  }
});
test("RPC u64 transport and exact source identity survive the browser boundary", async ({
  page,
}) => {
  const input =
    source.replace('"slot":444625255', '"slot":18446744073709551615') + "  \n";
  await page
    .getByLabel("Import genesis RPC response")
    .setInputFiles(upload(input));
  await expect(page.getByLabel("RPC capture inspection")).toContainText(
    "18446744073709551615",
  );
  await expect(page.getByLabel("RPC capture inspection")).toContainText(
    `sha256:${createHash("sha256").update(input).digest("hex")}`,
  );
});
test("recorded capture demo, download and reimport work offline with accessible narrow layout", async ({
  page,
  context,
}, info) => {
  await expect(
    page.getByText("Ready for offline verification", { exact: false }),
  ).toBeVisible();
  const requests: string[] = [];
  page.on("request", (r) => {
    if (r.url().startsWith("http")) requests.push(r.url());
  });
  await context.setOffline(true);
  await page
    .getByRole("button", { name: "Load recorded testnet response" })
    .click();
  await expect(
    page.getByRole("heading", { name: "Genesis capture needs context" }),
  ).toBeVisible();
  const pending = page.waitForEvent("download");
  await page
    .getByRole("button", { name: "Download original response" })
    .click();
  const path = await (await pending).path();
  await page
    .getByLabel("Import genesis RPC response")
    .setInputFiles(upload(await readFile(path!)));
  await expect(
    page.getByRole("heading", { name: "Genesis capture needs context" }),
  ).toBeVisible();
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
    .getByRole("heading", { name: "Genesis capture needs context" })
    .scrollIntoViewIfNeeded();
  await page.screenshot({ path: info.outputPath("capture-mobile.png") });
  expect(requests).toEqual([]);
  await context.setOffline(false);
});
test("replacing capture during a slow file read prevents late evidence from reappearing", async ({
  page,
}) => {
  await page.evaluate(() => {
    const original = File.prototype.arrayBuffer;
    File.prototype.arrayBuffer = async function () {
      await new Promise((r) => setTimeout(r, 600));
      return original.call(this);
    };
  });
  await page
    .getByLabel("Import genesis RPC response")
    .setInputFiles(upload(source));
  await page.getByRole("button", { name: "Load", exact: true }).click();
  await expect(page.getByRole("status")).toContainText("Example loaded");
  await page.waitForTimeout(700);
  await expect(page.getByLabel("RPC capture inspection")).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Verify evidence" }),
  ).toBeEnabled();
});
