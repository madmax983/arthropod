import { expect, test } from "@playwright/test";

test("chromium headed provides a WebGPU adapter", async ({ page, browserName }) => {
  test.skip(browserName !== "chromium", "WebGPU adapter gate only targets Chromium");

  let adapterAvailable = false;
  await expect(async () => {
    await page.goto("/", { waitUntil: "networkidle" });
    await page.waitForLoadState('networkidle');

    adapterAvailable = await page.evaluate(async () => {
      if (!("gpu" in navigator) || !navigator.gpu) {
        return false;
      }

      try {
        const adapter = await navigator.gpu.requestAdapter();
        return !!adapter;
      } catch {
        return false;
      }
    });

    expect(adapterAvailable).toBeTruthy();
  }).toPass();
});
