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
// Built the same way from the capture recorded on mainnet during epoch 1063.
const mainnet = (name: string) => ({
  name,
  mimeType: "application/json",
  buffer: readFileSync(
    new URL(
      `../../../fixtures/genesis-capture/mainnet/built/${name}`,
      import.meta.url,
    ),
  ),
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

test("the recorded mainnet genesis certificate verifies in the browser under registered-key trust", async ({
  page,
}, info) => {
  await page
    .getByLabel("Or import a proof bundle")
    .setInputFiles(mainnet("bundle.json"));
  await expect(page.getByRole("status")).toContainText("Bundle inspected");
  await page
    .getByLabel("Import checkpoint", { exact: true })
    .setInputFiles(mainnet("checkpoint.json"));
  await expect(page.getByRole("status")).toContainText("Checkpoint inspected");
  await page.getByLabel("Trust basis").selectOption("registered");
  const reason = "Rebuilt from the recorded epoch-1063 mainnet capture";
  await page.getByLabel("Why do you trust this checkpoint?").fill(reason);
  await page.getByRole("button", { name: "Use this checkpoint" }).click();
  await page.getByRole("button", { name: "Verify evidence" }).click();
  await expect(
    page.getByRole("heading", { name: "Genesis anchor verified" }),
  ).toBeVisible();
  await expect(
    page.getByText(
      "Signers hold 360285044710320374 of 438565601666629812 stake",
      { exact: false },
    ),
  ).toBeVisible();
  const genesis = page.locator("details.certificate").first();
  await genesis
    .getByRole("button", {
      name: "Show bytes 42–233 (192 bytes) for signature",
    })
    .click();
  const table = genesis.getByRole("table", {
    name: "Wire fields reported by the decoder",
  });
  await expect(
    table.getByRole("row", { name: /^Certificate tag/ }),
  ).toContainText("12 (genesis)");
  await expect(table.getByRole("row", { name: /^Slot/ })).toContainText(
    "459652999",
  );
  await expect(
    table.getByRole("row", { name: /^Shred version/ }),
  ).toContainText("50093");
  await page.screenshot({
    path: info.outputPath("mainnet-genesis.png"),
    fullPage: true,
  });
});
