import AxeBuilder from "@axe-core/playwright";
import { test, expect, type Page } from "@playwright/test";

const landingTitle = /Finality,\s+in full view\./;
const previewCaption = "Illustrative preview · synthetic evidence";
const previewHeadings = {
  Verify: "Every check, accounted for.",
  Tamper: "One byte. A different result.",
  Replay: "Same evidence. Fresh verification.",
};

async function openInspector(page: Page) {
  await page
    .getByRole("link", { name: "Open inspector", exact: true })
    .first()
    .click();
  await expect(page).toHaveURL(/#\/inspect$/);
  await expect(
    page.getByRole("heading", { name: "Follow the evidence." }),
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

test("landing explains its synthetic preview and links to the working inspector", async ({
  page,
}, info) => {
  await expect(
    page.getByRole("heading", { level: 1, name: landingTitle }),
  ).toBeVisible();
  await expect(page.getByText(previewCaption, { exact: true })).toBeVisible();
  await expect(
    page.getByText(
      "Synthetic examples are labeled. Recorded RPC evidence with missing context stays inconclusive. The limits stay beside the result.",
      { exact: true },
    ),
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

test("workflow link stays on the landing page and brings the workflow into view", async ({
  page,
}) => {
  await page.goto("/#/");
  await expect(
    page.getByRole("heading", { level: 1, name: landingTitle }),
  ).toBeVisible();
  await page
    .getByRole("link", { name: "Explore the workflow", exact: true })
    .click();
  await expect(page).toHaveURL(/#workflow$/);
  await expect(page.locator("#workflow")).toBeInViewport();
  await expect(
    page.getByRole("heading", { level: 1, name: landingTitle }),
  ).toHaveCount(1);
  await page.goBack();
  await expect(
    page.getByRole("heading", { level: 1, name: landingTitle }),
  ).toBeVisible();
});

test("preview modes remain illustrative and do not create evidence or select trust", async ({
  page,
}) => {
  const preview = page.getByRole("region", {
    name: "Illustrative inspector preview",
  });
  const modeControls = page.getByRole("group", {
    name: "Explore Monocle’s workflow",
  });
  await expect(
    modeControls.getByRole("button", { name: "Verify", exact: true }),
  ).toHaveAttribute("aria-pressed", "true");
  for (const mode of ["Tamper", "Replay", "Verify"] as const) {
    const button = modeControls.getByRole("button", {
      name: mode,
      exact: true,
    });
    await button.focus();
    await page.keyboard.press("Space");
    await expect(button).toHaveAttribute("aria-pressed", "true");
    await expect(
      page.getByRole("heading", {
        name: previewHeadings[mode],
        exact: true,
      }),
    ).toBeVisible();
    for (const other of ["Verify", "Tamper", "Replay"].filter(
      (name) => name !== mode,
    )) {
      await expect(
        modeControls.getByRole("button", { name: other, exact: true }),
      ).toHaveAttribute("aria-pressed", "false");
    }
    await expect(
      preview.getByText(previewCaption, { exact: true }),
    ).toBeVisible();
    if (mode === "Replay") {
      await expect(
        preview.getByText("Trust required", { exact: true }),
      ).toBeVisible();
      await expect(
        preview.getByText("Not trusted", { exact: true }),
      ).toBeVisible();
    }
    await expect(page.getByLabel("Evidence report")).toHaveCount(0);
  }
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
