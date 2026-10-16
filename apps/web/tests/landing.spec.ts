import AxeBuilder from "@axe-core/playwright";
import { test, expect, type Page } from "@playwright/test";

const landingTitle = /See finality reach the summit\./;
const ridgeCaption =
  "The ridge is fixture data. Ranks 0, 1 and 3 sign 40 + 25 + 15 of 100 stake and reach the 80% fast-path line exactly. Synthetic keys, invented stake.";

async function openInspector(page: Page) {
  await page
    .getByRole("link", { name: "Open inspector", exact: true })
    .first()
    .click();
  await expect(page).toHaveURL(/#\/inspect$/);
  await expect(
    page.getByRole("heading", { level: 1, name: "Monocle evidence inspector" }),
  ).toBeAttached();
  await expect(
    page.getByRole("button", { name: "Load", exact: true }),
  ).toBeVisible();
}

async function loadExample(page: Page) {
  await page
    .getByLabel("Signed synthetic example")
    .selectOption("fast_finality");
  await page.getByRole("button", { name: "Load", exact: true }).click();
  await expect(page.getByRole("status")).toContainText("Example loaded");
}

async function audit(page: Page) {
  const result = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
    .analyze();
  expect(result.violations).toEqual([]);
}

test.beforeEach(async ({ page }) => {
  await page.goto("/");
});

test("landing labels its fixture data and links to the working inspector", async ({
  page,
}, info) => {
  await expect(
    page.getByRole("heading", { level: 1, name: landingTitle }),
  ).toBeVisible();
  await expect(page.getByText(ridgeCaption, { exact: true })).toBeVisible();
  await expect(
    page.getByText("Rejected evidence is not proof that a block is non-final", {
      exact: true,
    }),
  ).toBeVisible();
  const source = page
    .getByRole("navigation", { name: "Main navigation" })
    .getByRole("link", { name: "GitHub", exact: true });
  await expect(source).toHaveAttribute(
    "href",
    "https://github.com/mansi0xc/monocle",
  );
  await expect(source).toHaveAttribute("target", "_blank");
  await expect(source).toHaveAttribute("rel", /noopener/);
  await page.keyboard.press("Tab");
  await expect(
    page.getByRole("link", { name: "Skip to content" }),
  ).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("main")).toBeFocused();
  await page.keyboard.press("Tab");
  await audit(page);
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({
    path: info.outputPath("landing-desktop.png"),
    fullPage: true,
  });
  await openInspector(page);
  await expect(
    page.getByRole("button", { name: "Verify evidence" }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Remove selection" }),
  ).toHaveCount(0);
});

test("outcomes link stays on the landing page and brings the outcomes into view", async ({
  page,
}) => {
  await page.goto("/#/");
  await expect(
    page.getByRole("heading", { level: 1, name: landingTitle }),
  ).toBeVisible();
  await page
    .getByRole("link", { name: "See what a run reports", exact: true })
    .click();
  await expect(page).toHaveURL(/#outcomes$/);
  await expect(page.locator("#outcomes")).toBeInViewport();
  await expect(
    page.getByRole("heading", { level: 1, name: landingTitle }),
  ).toHaveCount(1);
  await page.goBack();
  await expect(
    page.getByRole("heading", { level: 1, name: landingTitle }),
  ).toBeVisible();
});

test("fixture outcomes are labelled and do not create evidence or select trust", async ({
  page,
}) => {
  const outcomes = page.locator("#outcomes");
  await expect(outcomes).toContainText(
    "These results come from Monocle’s signed synthetic fixtures.",
  );
  for (const code of [
    "FAST_FINALITY_ESTABLISHED",
    "SLOW_FINALITY_ESTABLISHED",
    "MISSING_FINALIZATION",
    "INVALID_SIGNATURE",
    "INSUFFICIENT_STAKE",
    "GENESIS_ANCHOR_ESTABLISHED",
    "CAPTURE_CONTEXT_MISSING",
  ])
    await expect(outcomes.getByText(code, { exact: true })).toBeVisible();
  for (const control of ["Load", "Verify evidence", "Use this checkpoint"])
    await expect(
      page.getByRole("button", { name: control, exact: true }),
    ).toHaveCount(0);
  await expect(page.locator('input[type="file"]')).toHaveCount(0);
  await expect(page.getByLabel("Evidence report")).toHaveCount(0);
  await openInspector(page);
  await expect(
    page.getByRole("button", { name: "Verify evidence" }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Use this checkpoint" }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Remove selection" }),
  ).toHaveCount(0);
});

test("ridge scenarios and the tamper demo restate recorded results without verifying", async ({
  page,
}) => {
  const scenarios = page.getByRole("group", {
    name: "Fixture certificate shown on the ridge",
  });
  const hero = page.locator(".landing-hero");
  await expect(
    scenarios.getByRole("button", { name: "Fast path", exact: true }),
  ).toHaveAttribute("aria-pressed", "true");
  for (const [name, code, caption] of [
    ["Too little stake", "INSUFFICIENT_STAKE", "65 of 100 stake"],
    ["Slow path", "SLOW_FINALITY_ESTABLISHED", "clearing 60% in both rounds"],
    ["Genesis anchor", "GENESIS_ANCHOR_ESTABLISHED", "82 of 100 stake"],
    ["Fast path", "FAST_FINALITY_ESTABLISHED", "40 + 25 + 15 of 100 stake"],
  ]) {
    const button = scenarios.getByRole("button", { name, exact: true });
    await button.focus();
    await page.keyboard.press("Enter");
    await expect(button).toHaveAttribute("aria-pressed", "true");
    await expect(hero.locator(".ridge-result")).toContainText(code);
    await expect(hero.locator(".ridge-caption")).toContainText(caption);
    await expect(hero.locator(".ridge-caption")).toContainText(
      "Synthetic keys, invented stake.",
    );
  }
  const demo = page.locator(".tamper-demo");
  await expect(demo).toContainText("Nothing is verified on this page");
  const byte = demo.getByRole("button", { name: /^Byte 42: 01\./ });
  await byte.click();
  await expect(
    demo.getByRole("button", { name: /^Byte 42: 00, changed from 01\./ }),
  ).toHaveAttribute("aria-pressed", "true");
  await expect(demo).toContainText("INVALID_SIGNATURE");
  await expect(demo).toContainText("PREREQUISITE_MISSING");
  await expect(demo).toContainText("× REJECTED");
  await demo.getByRole("button", { name: /^Byte 42/ }).click();
  await expect(demo).toContainText("✓ VERIFIED");
  await expect(page.getByLabel("Evidence report")).toHaveCount(0);
});

test("home and browser history preserve loaded evidence without granting checkpoint trust", async ({
  page,
}) => {
  await openInspector(page);
  await loadExample(page);
  await page.getByRole("link", { name: "Monocle home", exact: true }).click();
  await expect(
    page.getByRole("heading", { level: 1, name: landingTitle }),
  ).toBeVisible();
  await page.goBack();
  await expect(page).toHaveURL(/#\/inspect$/);
  await expect(
    page.getByRole("button", { name: "Verify evidence" }),
  ).toBeEnabled();
  await expect(
    page.getByRole("button", { name: "Use this checkpoint" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Remove selection" }),
  ).toHaveCount(0);
  await page.getByRole("button", { name: "Verify evidence" }).click();
  await expect(
    page.getByRole("heading", { name: "Verification inconclusive" }),
  ).toBeVisible();
  await page.goForward();
  await expect(
    page.getByRole("heading", { level: 1, name: landingTitle }),
  ).toBeVisible();
  await openInspector(page);
  await expect(
    page.getByRole("heading", { name: "Verification inconclusive" }),
  ).toBeVisible();
});

test("an explicit checkpoint decision and its verified report survive a visit home", async ({
  page,
}) => {
  await openInspector(page);
  await loadExample(page);
  const trustReason =
    "Reviewed the synthetic fixture locally before selecting this checkpoint.";
  await page.getByLabel("Trust basis").selectOption("operator");
  await page.getByLabel("Why do you trust this checkpoint?").fill(trustReason);
  await page.getByRole("button", { name: "Use this checkpoint" }).click();
  await page.getByRole("button", { name: "Verify evidence" }).click();
  await expect(
    page.getByRole("heading", { name: "Fast finality verified" }),
  ).toBeVisible();
  await page.getByRole("link", { name: "Monocle home", exact: true }).click();
  await expect(
    page.getByRole("heading", { level: 1, name: landingTitle }),
  ).toBeVisible();
  await openInspector(page);
  await expect(
    page.getByText(`Operator-pinned: ${trustReason}`, { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Fast finality verified" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Remove selection" }).click();
  await expect(
    page.getByRole("heading", { name: "Fast finality verified" }),
  ).toHaveCount(0);
  await page.getByRole("button", { name: "Verify evidence" }).click();
  await expect(
    page.getByRole("heading", { name: "Verification inconclusive" }),
  ).toBeVisible();
});

for (const width of [375, 768]) {
  test(`landing remains accessible without horizontal overflow at ${width}px`, async ({
    page,
  }, info) => {
    await page.setViewportSize({ width, height: 900 });
    await page.emulateMedia({ reducedMotion: "reduce" });
    await expect(
      page.getByRole("heading", { level: 1, name: landingTitle }),
    ).toBeVisible();
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
    await audit(page);
    if (width === 375) {
      await page.evaluate(() => document.fonts.ready);
      await page.screenshot({
        path: info.outputPath("landing-mobile.png"),
        fullPage: true,
      });
    }
    await openInspector(page);
    await expect(
      page.getByRole("button", { name: "Load", exact: true }),
    ).toBeVisible();
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
  });
}
