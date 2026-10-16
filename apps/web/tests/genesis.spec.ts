import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { readFileSync } from "node:fs";

// Built by tools/genesis-checkpoint from the committed synthetic RPC capture.
const built = (name: string) =>
  readFileSync(
    new URL(
      `../../../fixtures/genesis-capture/synthetic/built/${name}`,
      import.meta.url,
    ),
  );
const upload = (name: string) => ({
  name,
  mimeType: "application/json",
  buffer: built(name),
});

test.beforeEach(async ({ page }) => {
  await page.goto("/#/inspect");
});

test("a rebuilt validator set verifies the genesis certificate only under registered-key trust", async ({
  page,
}) => {
  await page
    .getByLabel("Or import a proof bundle")
    .setInputFiles(upload("bundle.json"));
  // A new job supersedes an unfinished one, so let the bundle finish first.
  await expect(page.getByRole("status")).toContainText("Bundle inspected");
  await page
    .getByLabel("Import checkpoint", { exact: true })
    .setInputFiles(upload("checkpoint.json"));
  await expect(page.getByRole("status")).toContainText("Checkpoint inspected");
  const basis = page.getByLabel("Trust basis");
  // A captured checkpoint offers no synthetic option.
  await expect(basis.locator("option")).toHaveText([
    "Operator-pinned checkpoint",
    "Operator-pinned, keys registered on chain",
  ]);
  const reason = "Rebuilt from the committed synthetic capture";
  await page.getByLabel("Why do you trust this checkpoint?").fill(reason);
  await page.getByRole("button", { name: "Use this checkpoint" }).click();
  await page.getByRole("button", { name: "Verify evidence" }).click();
  // Ordinary trust still demands proofs of possession, which vote accounts lack.
  await expect(
    page.getByRole("heading", { name: "Verification inconclusive" }),
  ).toBeVisible();
  await expect(page.getByText("MISSING_POP").first()).toBeVisible();

  await page.getByRole("button", { name: "Remove selection" }).click();
  await basis.selectOption("registered");
  await expect(page.getByText(/proofs are not re-verified/)).toBeVisible();
  const result = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
    .analyze();
  expect(result.violations).toEqual([]);
  await page.getByLabel("Why do you trust this checkpoint?").fill(reason);
  await page.getByRole("button", { name: "Use this checkpoint" }).click();
  await expect(
    page.getByText(`Operator-pinned, registered keys: ${reason}`, {
      exact: true,
    }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Verify evidence" }).click();
  await expect(
    page.getByRole("heading", { name: "Genesis anchor verified" }),
  ).toBeVisible();
  await expect(
    page.getByText("REGISTERED_KEYS_ACCEPTED").first(),
  ).toBeVisible();
  await expect(
    page.getByText("GENESIS_ANCHOR_ESTABLISHED").first(),
  ).toBeVisible();
});
