import { test, expect } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const root = resolve(import.meta.dirname, "../..");
const out = resolve(root, "target/progress-e2e/index.html");

test.beforeAll(() => {
  test.setTimeout(600_000); // the hook compiles plinth-progress
  mkdirSync(resolve(out, ".."), { recursive: true });
  execFileSync(
    "cargo",
    ["run", "--locked", "-q", "-p", "plinth-progress", "--", "build",
      "crates/progress/tests/fixtures/render/long-label.toml", "--out", out],
    { cwd: root, stdio: "inherit" },
  );
});

test("long milestone labels wrap inside the status column at 390px", async ({ page }) => {
  await page.goto(pathToFileURL(out).href);
  const layout = await page.evaluate(() => {
    const wrap = document.querySelector(".table-wrap")!;
    const words = [...document.querySelectorAll(".status-word")];
    return {
      page: document.documentElement.scrollWidth - document.documentElement.clientWidth,
      table: wrap.scrollWidth - wrap.clientWidth,
      wrappedPills: [...document.querySelectorAll(".status")].filter(pill => pill.getClientRects()[0].height > 30).length,
      brokenWords: words.filter(word => word.getClientRects().length !== 1).length,
    };
  });
  expect(layout.page).toBeLessThanOrEqual(0);
  expect(layout.table).toBeLessThanOrEqual(0);
  expect(layout.wrappedPills).toBeGreaterThan(0); // the fixture really wraps
  expect(layout.brokenWords).toBe(0);
});
