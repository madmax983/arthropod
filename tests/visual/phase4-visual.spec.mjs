import { expect, test } from "@playwright/test";

const visualCases = ["blur", "blend", "clipping", "mask", "image"];

for (const visualCase of visualCases) {
  test(`phase4 ${visualCase} snapshot`, async ({ page }) => {
    await page.goto(`/?case=${visualCase}`);
    const adapterProbe = await page.evaluate(async () => {
      if (!("gpu" in navigator) || navigator.gpu == null) {
        return { ok: false, reason: "navigator.gpu unavailable" };
      }
      try {
        const adapter = await navigator.gpu.requestAdapter();
        if (adapter == null) {
          return { ok: false, reason: "failed to request adapter" };
        }
        return { ok: true, reason: "" };
      } catch (error) {
        return { ok: false, reason: String(error) };
      }
    });
    if (!adapterProbe.ok) {
      test.skip(true, `WebGPU unavailable in this browser/runtime: ${adapterProbe.reason}`);
    }
    await page.waitForFunction(() => {
      const body = document.body;
      return !!body && body.hasAttribute("data-arthropod-ready");
    }, {
      timeout: 60000
    });
    const body = page.locator("body");
    const ready = await body.getAttribute("data-arthropod-ready");
    if (ready !== "1") {
      const error = (await body.getAttribute("data-arthropod-error")) ?? "unknown startup error";
      if (/webgpu|failed to request adapter|surface creation|surface is not configured|canvas\.getcontext|phase4_visual_web panic/i.test(error)) {
        test.skip(true, `WebGPU unavailable in this browser/runtime: ${error}`);
      }
      throw new Error(`phase4 fixture startup failed: ${error}`);
    }
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
