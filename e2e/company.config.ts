import { defineConfig } from "@playwright/test";

const command = (fixture: string, port: number) =>
  `cargo run --locked --manifest-path ../Cargo.toml -p plinth-project --bin plinth-company -- dev --config ../website/company-examples/${fixture}.toml --out ../target/company-e2e/${fixture} --port ${port} --no-open --no-watch`;

export default defineConfig({
  testDir: "./tests",
  testMatch: "company*.spec.ts",
  timeout: 30_000,
  retries: 0,
  workers: 2,
  use: {
    baseURL: "http://127.0.0.1:1122",
    headless: true,
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
    launchOptions: { executablePath: process.env.CHROMIUM_PATH || undefined },
  },
  projects: [
    { name: "desktop", testMatch: "company.spec.ts", use: { viewport: { width: 1440, height: 900 } } },
    { name: "mobile", testMatch: "company.spec.ts", use: { viewport: { width: 390, height: 844 } } },
    {
      name: "visibility",
      testMatch: "company-visibility.spec.ts",
      use: {
        headless: false,
        viewport: { width: 1440, height: 900 },
        launchOptions: {
          executablePath: process.env.CHROMIUM_PATH || undefined,
          ignoreDefaultArgs: ["--disable-backgrounding-occluded-windows", "--disable-renderer-backgrounding", "--disable-background-timer-throttling"],
        },
      },
    },
  ],
  webServer: [
    { command: command("north", 1122), url: "http://127.0.0.1:1122", timeout: 120_000 },
    { command: command("vector", 1123), url: "http://127.0.0.1:1123", timeout: 120_000 },
    { command: command("ribbon", 1124), url: "http://127.0.0.1:1124", timeout: 120_000 },
    { command: command("mixed", 1125), url: "http://127.0.0.1:1125", timeout: 120_000 },
  ],
});
