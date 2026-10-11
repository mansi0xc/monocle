import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "./tests",
  testMatch: "**/*.spec.ts",
  workers: 1,
  timeout: 30_000,
  fullyParallel: false,
  outputDir: "../../test-results",
  use: {
    baseURL: "http://127.0.0.1:4177",
    viewport: { width: 1440, height: 1000 },
    launchOptions: {
      executablePath:
        process.env.CHROME_BIN ||
        (process.platform === "darwin"
          ? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
          : "/usr/bin/chromium"),
    },
    trace: "retain-on-failure",
  },
  webServer: {
    command: "npm run preview -- --port 4177 --strictPort",
    cwd: "../..",
    url: "http://127.0.0.1:4177",
    reuseExistingServer: false,
  },
});
