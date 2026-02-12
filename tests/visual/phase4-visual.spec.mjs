import { expect, test } from "@playwright/test";

const visualCases = ["blur", "blend", "clipping"];

for (const visualCase of visualCases) {
  test(`phase4 ${visualCase} snapshot`, async ({ page }) => {
    await page.goto(`/?case=${visualCase}`);
    await page.waitForSelector("body[data-arthropod-ready='1']", {
      timeout: 30000
    });
    await expect(page.locator("body")).toHaveAttribute(
      "data-arthropod-case",
      visualCase
    );

    const canvas = page.locator("#arthropod-canvas");
    await expect(canvas).toBeVisible();

    await expect(canvas).toHaveScreenshot(`phase4-${visualCase}.png`, {
      animations: "disabled",
      caret: "hide",
      maxDiffPixelRatio: 0.01
    });
  });
}

