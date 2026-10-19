import AxeBuilder from "@axe-core/playwright";
import { test, expect, type Page } from "@playwright/test";

async function verifyExample(page: Page, name: string) {
  await page.getByLabel("Signed synthetic example").selectOption(name);
  await page.getByRole("button", { name: "Load", exact: true }).click();
  await expect(page.getByRole("status")).toContainText("Example loaded");
  await page.getByRole("button", { name: "Use this checkpoint" }).click();
}
test.beforeEach(async ({ page }) => {
  await page.goto("/#/inspect");
});

test("a failing check reveals and labels the exact bytes behind it", async ({
  page,
}, info) => {
  await verifyExample(page, "changed_signature");
  await page.getByRole("button", { name: "Verify evidence" }).click();
  await expect(
    page.getByRole("heading", { name: "Evidence rejected" }),
  ).toBeVisible();
  const notar = page.locator("details.certificate").first();
  await notar
    .getByRole("button", {
      name: "Show bytes 42–233 (192 bytes) for signature",
    })
    .click();
  const view = notar.locator(".hex-view");
  await expect(view).toBeVisible();
  await expect(view.locator(".hex-focus")).toHaveText(
    "Highlighted: bytes 42–233 (192 bytes), Aggregate signature.",
  );
  const table = view.getByRole("table", {
    name: "Wire fields reported by the decoder",
  });
  await expect(table.getByRole("row")).toHaveCount(9); // header + 8 fields
  await expect(table.getByRole("row", { name: /^Slot/ })).toContainText("42");
  await expect(
    table.getByRole("row", { name: /^Certificate tag/ }),
  ).toContainText("9 (notarization)");
  await table.getByRole("button", { name: "Signer bitmap" }).click();
  await expect(view.locator(".hex-focus")).toHaveText(
    "Highlighted: bytes 242–245 (4 bytes), Signer bitmap.",
  );
  await expect(
    table.getByRole("button", { name: "Signer bitmap" }),
  ).toHaveAttribute("aria-pressed", "true");
  await expect(view.locator(".hex-byte[data-lit]")).toHaveCount(4);
  await view.scrollIntoViewIfNeeded();
  await view.screenshot({ path: info.outputPath("bytes-desktop.png") });
  const result = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
    .analyze();
  expect(result.violations).toEqual([]);
});

test("an experiment marks the byte it changed, and phones never scroll sideways", async ({
  page,
}) => {
  await page.setViewportSize({ width: 375, height: 800 });
  await verifyExample(page, "fast_finality");
  await page.locator(".tamper-lab > summary").click();
  await page
    .getByRole("button", { name: "Create experiment", exact: true })
    .click();
  await expect(page.getByRole("status")).toContainText("Experiment created");
  await page.getByRole("button", { name: "Verify comparison" }).click();
  await expect(page.getByRole("status")).toContainText("Comparison complete");
  await page
    .getByRole("button", {
      name: "Show bytes 42–233 (192 bytes) for signature",
    })
    .first()
    .click();
  const view = page.locator(".hex-view").first();
  await expect(view.locator(".hex-changes")).toHaveText(
    "Changed by the experiment: byte 42, 01 → 00.",
  );
  await expect(view.locator(".hex-byte[data-changed]")).toHaveText("00");
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(0);
});
