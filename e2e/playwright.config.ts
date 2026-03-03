import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  timeout: 30_000,
  retries: 1,
  use: {
    baseURL: process.env.BASE_URL || "http://localhost:3000",
    headless: true,
  },
  projects: [
    {
      name: "chromium",
      use: {
        // On NixOS, use system Chromium via CHROMIUM_PATH env var
        launchOptions: {
          executablePath: process.env.CHROMIUM_PATH || undefined,
        },
      },
    },
  ],
});
