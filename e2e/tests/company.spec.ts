import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { gzipSync } from "node:zlib";

async function settle(page: Page) {
  await expect(page.locator("body")).toHaveAttribute("data-company-ready", "true");
  for (const image of await page.locator('img[loading="lazy"][src]').all()) {
    await image.scrollIntoViewIfNeeded();
    await image.evaluate((element: HTMLImageElement) => element.decode());
  }
  await page.evaluate(async () => {
    await document.fonts.ready;
    await Promise.all([...document.images].filter(image => image.getAttribute("src")).map(image => image.decode()));
    await new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
    scrollTo({ top: 0, behavior: "instant" });
  });
}

test("required content and local assets work without browser errors", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.goto("/");
  await settle(page);
  await expect(page.locator("h1")).toHaveText("Good ideas.\nBetter together.");
  await expect(page.locator(".company-member")).toHaveCount(2);
  await expect(page.locator("blockquote")).toContainText("understood the problem");
  await expect(page.locator(".company-partner-type")).toHaveText(["Customer", "Partner", "Affiliation"]);
  const font = await page.request.get("/fonts/plinth-company-inter.woff2");
  expect(font.ok()).toBeTruthy();
  expect(font.headers()["content-type"]).toBe("font/woff2");
  expect(errors).toEqual([]);
});

test("company pages meet WCAG AA checks", async ({ page }) => {
  for (const url of ["/", "http://127.0.0.1:1123/", "http://127.0.0.1:1123/people/", "http://127.0.0.1:1125/"]) {
    await page.goto(url);
    await settle(page);
    const result = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21aa"]).analyze();
    expect(result.violations).toEqual([]);
  }
});

test("asset-backed composed sections align with optional eyebrows", async ({ page }, info) => {
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.goto("http://127.0.0.1:1125/");
  await settle(page);
  await expect(page.locator(".company-portrait")).toBeVisible();
  await expect(page.locator(".company-partner img")).toBeVisible();
  const font = await page.request.get("http://127.0.0.1:1125/fonts/field.woff2");
  expect(font.ok()).toBeTruthy();
  expect(font.headers()["content-type"]).toBe("font/woff2");
  expect(await page.evaluate(() => document.fonts.check('16px "Field Inter"'))).toBe(true);
  await expect(page.locator("body")).toHaveAttribute("data-motion-state", "off");
  await expect(page.locator(".company-motion-toggle")).toHaveCount(0);
  expect(await page.evaluate(() => document.getAnimations().length)).toBe(0);
  await page.locator("#mission").evaluate(element => element.scrollIntoView({ behavior: "instant", block: "start" }));
  const title = await page.locator("#mission h2").boundingBox();
  const statement = await page.locator(".company-mission-body").boundingBox();
  const mobile = info.project.name === "mobile";
  expect(Math.abs(title!.x - (mobile ? 24 : 517.333))).toBeLessThanOrEqual(1);
  expect(Math.abs(title!.y - (mobile ? 72 : 96))).toBeLessThanOrEqual(2);
  expect(Math.abs(title!.width - (mobile ? 342 : 842.667))).toBeLessThanOrEqual(1);
  expect(Math.abs(title!.x - statement!.x)).toBeLessThanOrEqual(1);
  expect(title!.width).toBeGreaterThan(info.project.name === "mobile" ? 300 : 600);
  const contact = await page.locator("#contact h2").boundingBox();
  const action = await page.locator("#contact > a").boundingBox();
  expect(Math.abs(contact!.x - action!.x)).toBeLessThanOrEqual(1);
  const content = page.locator("#story");
  await expect(content).toHaveCSS("padding-top", info.project.name === "mobile" ? "48px" : "72px");
  await expect(content).toHaveCSS("border-bottom-width", "1px");
  await expect(content.locator("h2")).toHaveCSS("margin-top", "0px");
  const geometry = await content.boundingBox();
  expect(geometry!.x).toBe(info.project.name === "mobile" ? 24 : 80);
  expect(errors).toEqual([]);
});

test("composed gallery opens, closes, and restores keyboard focus", async ({ page }) => {
  await page.goto("http://127.0.0.1:1125/");
  await settle(page);
  const trigger = page.locator("#gallery [data-lightbox-image]");
  await trigger.focus();
  await page.keyboard.press("Enter");
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await expect(dialog.locator("img")).toHaveAttribute("alt", "Illustrated studio portrait");
  await expect(dialog).toContainText("Synthetic asset reference");
  await expect(page.getByRole("button", { name: "Close enlarged image" })).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();
  await trigger.click();
  await page.getByRole("button", { name: "Close enlarged image" }).click();
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();
});

test("initial reduced-motion preference prevents animation on every motif", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  for (const port of [1122, 1123, 1124]) {
    await page.goto(`http://127.0.0.1:${port}/`);
    await settle(page);
    await expect(page.locator("body")).toHaveAttribute("data-motion-state", "reduced");
    await expect(page.locator(".company-motion-toggle")).toBeHidden();
    expect(await page.evaluate(() => document.getAnimations().length)).toBe(0);
  }
});

test("every navigation destination remains reachable", async ({ page }, info) => {
  await page.goto("/");
  await settle(page);
  const openMenu = async () => {
    if (info.project.name === "mobile") {
      await page.getByRole("button", { name: "Menu", exact: true }).click();
      await expect(page.getByRole("button", { name: "Close", exact: true })).toHaveAttribute("aria-expanded", "true");
    }
  };
  for (const [label, anchor] of [["Purpose", "mission"], ["People", "team"], ["Partners", "partners"]]) {
    await openMenu();
    await page.getByRole("navigation").getByRole("link", { name: label, exact: true }).click();
    await expect(page).toHaveURL(new RegExp(`#${anchor}$`));
    await expect(page.locator(`#${anchor}`)).toBeInViewport();
  }
  if (info.project.name === "mobile") {
    const menu = page.getByRole("button", { name: "Menu", exact: true });
    await menu.click();
    await page.keyboard.press("Escape");
    await expect(menu).toBeFocused();
    await expect(menu).toHaveAttribute("aria-expanded", "false");
  }
  await page.goto("http://127.0.0.1:1123/");
  await settle(page);
  await openMenu();
  await page.getByRole("navigation").getByRole("link", { name: "Our people", exact: true }).click();
  await expect(page).toHaveURL("http://127.0.0.1:1123/people/");
  await settle(page);
  await expect(page.locator("h1")).toHaveText("Our people");
  for (const [label, anchor] of [["Our purpose", "mission"], ["Our work", "work"], ["Get in touch ↗", "contact"]]) {
    await openMenu();
    await page.getByRole("navigation").getByRole("link", { name: label, exact: true }).click();
    await expect(page).toHaveURL(`http://127.0.0.1:1123/#${anchor}`);
    await expect(page.locator(`#${anchor}`)).toBeInViewport();
  }
});

test("motion pauses and respects changing reduced-motion preferences", async ({ page }) => {
  await page.goto("/");
  await settle(page);
  await expect(page.locator("body")).toHaveAttribute("data-motion-state", "playing");
  await page.getByRole("button", { name: /Pause motion/ }).click();
  await expect(page.locator("body")).toHaveAttribute("data-motion-state", "paused");
  await expect.poll(() => page.evaluate(() => document.getAnimations()[0]?.playState)).toBe("paused");
  await page.evaluate(async () => { await Promise.all(document.getAnimations().map(animation => animation.ready)); });
  const time = await page.evaluate(() => document.getAnimations()[0].currentTime);
  await page.evaluate(() => new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))));
  expect(await page.evaluate(() => document.getAnimations()[0].currentTime)).toBe(time);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await expect(page.locator("body")).toHaveAttribute("data-motion-state", "reduced");
  await expect(page.locator(".company-motion-toggle")).toBeHidden();
  expect(await page.evaluate(() => document.getAnimations().length)).toBe(0);
  await page.emulateMedia({ reducedMotion: "no-preference" });
  await expect(page.locator("body")).toHaveAttribute("data-motion-state", "paused");
  await page.getByRole("button", { name: /Resume motion/ }).click();
  await expect(page.locator("body")).toHaveAttribute("data-motion-state", "playing");
});

test("all three configured motifs run in the browser", async ({ page }) => {
  for (const [port, motif] of [[1122, "contour"], [1123, "orbit"], [1124, "ribbon"]] as const) {
    await page.goto(`http://127.0.0.1:${port}/`);
    await settle(page);
    await expect(page.locator(`.company-art-${motif} svg`)).toBeVisible();
    await expect.poll(() => page.evaluate(() => document.getAnimations()[0]?.playState)).toBe("running");
    const initial = await page.evaluate(() => document.getAnimations()[0].currentTime as number);
    await expect.poll(() => page.evaluate(() => document.getAnimations()[0].currentTime as number)).toBeGreaterThan(initial);
  }
});

test("content and navigation work with JavaScript disabled", async ({ browser }, info) => {
  const context = await browser.newContext({ javaScriptEnabled: false, viewport: info.project.use.viewport });
  const page = await context.newPage();
  await page.goto("http://127.0.0.1:1122/");
  await expect(page.getByRole("navigation").getByRole("link", { name: "People", exact: true })).toBeVisible();
  await expect(page.locator(".company-member")).toHaveCount(2);
  await expect(page.locator(".company-motion-toggle")).toBeHidden();
  await expect(page.locator(".company-motion-layer")).toHaveCSS("animation-play-state", "paused");
  await page.getByRole("navigation").getByRole("link", { name: "People", exact: true }).click();
  await expect(page).toHaveURL(/#team$/);
  await expect(page.locator("#team")).toBeInViewport();
  await context.close();
});

test("layouts remain inside the viewport at narrow and intermediate widths", async ({ page }) => {
  for (const width of [320, 390, 768, 1024, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    for (const url of ["/", "http://127.0.0.1:1123/people/", "http://127.0.0.1:1125/"]) {
      await page.goto(url);
      await settle(page);
      const sizes = await page.evaluate(() => ({ width: innerWidth, scroll: document.documentElement.scrollWidth }));
      expect(sizes.scroll, `${url} overflows at ${width}px`).toBeLessThanOrEqual(sizes.width);
    }
  }
});

test("motion keeps layout stable within the enhancement budget", async ({ page }, info) => {
  const session = await page.context().newCDPSession(page);
  if (info.project.name === "mobile") await session.send("Emulation.setCPUThrottlingRate", { rate: 4 });
  for (const port of [1122, 1123, 1124]) {
    await page.goto(`http://127.0.0.1:${port}/`);
    await settle(page);
    const script = await page.request.get("/company.js");
    expect(gzipSync(await script.body()).length).toBeLessThanOrEqual(15 * 1024);
    const observations = await page.evaluate(async () => {
      const shifts: number[] = [], longTasks: number[] = [];
      const record = (entries: PerformanceEntry[]) => {
        for (const entry of entries) {
          if (entry.entryType === "layout-shift") shifts.push((entry as PerformanceEntry & { value: number }).value);
          if (entry.entryType === "longtask") longTasks.push(entry.duration);
        }
      };
      const observer = new PerformanceObserver(list => record(list.getEntries()));
      observer.observe({ entryTypes: ["layout-shift", "longtask"] });
      const content = document.querySelector(".company-hero-copy")!;
      const before = content.getBoundingClientRect().toJSON();
      await new Promise<void>(resolve => {
        const start = performance.now();
        const frame = () => performance.now() - start >= 1000 ? resolve() : requestAnimationFrame(frame);
        requestAnimationFrame(frame);
      });
      const after = content.getBoundingClientRect().toJSON();
      record(observer.takeRecords());
      observer.disconnect();
      return { shifts, longTasks, before, after };
    });
    expect(observations.shifts.reduce((total, value) => total + value, 0)).toBe(0);
    expect(observations.longTasks).toEqual([]);
    expect(observations.after).toEqual(observations.before);
  }
});

test("reference geometry and section reading order match the design", async ({ page }, info) => {
  await page.goto("/");
  await settle(page);
  const mobile = info.project.name === "mobile";
  const heading = await page.locator("h1").boundingBox();
  expect(heading!.x).toBe(mobile ? 24 : 80);
  expect(Math.abs(heading!.y - (mobile ? 173.5 : 226))).toBeLessThanOrEqual(2);
  expect(Math.abs(heading!.height - (mobile ? 172 : 192))).toBeLessThanOrEqual(2);
  const actions = await page.locator(".company-actions").boundingBox();
  const meta = await page.locator(".company-hero-meta").boundingBox();
  expect(meta!.y).toBeGreaterThan(actions!.y + actions!.height);
  for (const action of await page.locator(".company-actions a").all()) await expect(action).toBeVisible();
  for (const selector of ["#team .company-team-grid", "#testimonials .company-quotes", "#partners .company-partners"]) {
    const grid = page.locator(selector);
    const box = await grid.boundingBox();
    expect(box!.x).toBe(mobile ? 24 : 80);
    expect(box!.width).toBe(mobile ? 342 : 1280);
    const sectionHeading = await grid.locator("..").locator(".company-section-heading").boundingBox();
    expect(box!.y).toBeGreaterThanOrEqual(sectionHeading!.y + sectionHeading!.height + (mobile ? 31 : 47));
  }
  await page.goto("http://127.0.0.1:1123/");
  await settle(page);
  const metricsHeading = await page.locator("#impact .company-section-heading").boundingBox();
  const metrics = await page.locator("#impact dl").boundingBox();
  expect(metrics!.y).toBeGreaterThan(metricsHeading!.y + metricsHeading!.height);
  await expect(page.locator("#impact dt")).toHaveCount(3);
});

test("capture deterministic reference frames after fonts and layout settle", async ({ page }, info) => {
  const directory = resolve("../target/company-evidence/local");
  await mkdir(directory, { recursive: true });
  await page.goto("/");
  await settle(page);
  await page.evaluate(() => {
    for (const animation of document.getAnimations()) { animation.pause(); animation.currentTime = 0; }
  });
  await page.screenshot({ path: `${directory}/north-${info.project.name}.png` });
  await page.screenshot({ path: `${directory}/north-${info.project.name}-full.png`, fullPage: true });
  if (info.project.name === "mobile") {
    await page.getByRole("button", { name: "Menu", exact: true }).click();
    await page.screenshot({ path: `${directory}/menu-mobile.png` });
    await page.keyboard.press("Escape");
    await page.emulateMedia({ reducedMotion: "reduce" });
    await expect(page.locator("body")).toHaveAttribute("data-motion-state", "reduced");
    await page.screenshot({ path: `${directory}/reduced-mobile.png` });
    await page.emulateMedia({ reducedMotion: "no-preference" });
  }
  for (const name of ["mission", "team", "testimonials", "partners", "contact"]) {
    await page.locator(`#${name}`).evaluate(element => element.scrollIntoView({ behavior: "instant", block: "start" }));
    await page.screenshot({ path: `${directory}/${name}-${info.project.name}.png` });
  }
  await page.goto("http://127.0.0.1:1123/");
  await settle(page);
  await page.evaluate(() => {
    for (const animation of document.getAnimations()) { animation.pause(); animation.currentTime = 0; }
  });
  await page.screenshot({ path: `${directory}/vector-${info.project.name}.png` });
  await page.goto("http://127.0.0.1:1123/people/");
  await settle(page);
  await page.screenshot({ path: `${directory}/people-${info.project.name}.png` });
  await page.goto("http://127.0.0.1:1125/");
  await settle(page);
  await page.screenshot({ path: `${directory}/mixed-${info.project.name}-full.png`, fullPage: true });
  await page.locator("#mission").evaluate(element => element.scrollIntoView({ behavior: "instant", block: "start" }));
  await page.screenshot({ path: `${directory}/mixed-${info.project.name}.png` });
});
