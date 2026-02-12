import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "tests/visual",
  timeout: 120000,
  expect: {
    timeout: 10000
  },
  use: {
    baseURL: "http://127.0.0.1:8090",
    viewport: { width: 1280, height: 720 },
    deviceScaleFactor: 1,
    colorScheme: "dark"
  },
  webServer: {
    command:
      "trunk serve --example phase4_visual_web --features web --address 127.0.0.1 --port 8090",
    url: "http://127.0.0.1:8090",
    timeout: 180000,
    reuseExistingServer: true
  },
  projects: [
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
        launchOptions: {
          args: ["--enable-unsafe-webgpu"]
        }
      }
    },
    {
      name: "firefox",
      use: {
        ...devices["Desktop Firefox"],
        launchOptions: {
          firefoxUserPrefs: {
            "dom.webgpu.enabled": true,
            "gfx.webrender.all": true
          }
        }
      }
    }
  ]
});
